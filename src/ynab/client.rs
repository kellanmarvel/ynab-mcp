use crate::error::YnabMcpError;
use crate::secrets::YnabApiToken;
use anyhow::{Context, Result};
use chrono;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::cache::{CacheStats, YnabCache};
use super::health::HealthStatus;
use super::models::*;

/// Truncate `s` to at most `max_bytes` bytes without splitting a UTF-8 character.
///
/// Slicing a `&str` at an arbitrary byte offset panics on a multi-byte character
/// (e.g. an accented payee name), and with `panic = "abort"` that kills the server.
fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[derive(Clone)]
pub struct YnabClient {
    client: Arc<Client>,
    base_url: String,
    #[allow(dead_code)] // Stored for potential future use (debugging, token refresh, etc.)
    api_token: YnabApiToken,
    cache: YnabCache,
}

impl YnabClient {
    /// Helper method for tests - creates client with default configuration
    #[cfg(test)]
    pub fn new_for_test(base_url: String, api_token: YnabApiToken) -> Result<Self> {
        Self::new_with_config(base_url, api_token, 1000, 30, 10)
    }

    pub fn new_with_config(
        base_url: String,
        api_token: YnabApiToken,
        cache_capacity: usize,
        request_timeout_secs: u64,
        connect_timeout_secs: u64,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let auth_value = HeaderValue::from_str(&format!("Bearer {}", api_token.expose_secret()))
            .context("Invalid API token format")?;
        headers.insert(AUTHORIZATION, auth_value);

        let client = Client::builder()
            // Request timeout - how long to wait for a complete request/response cycle
            .timeout(Duration::from_secs(request_timeout_secs))
            // Connection timeout - how long to wait to establish a connection
            .connect_timeout(Duration::from_secs(connect_timeout_secs))
            // Read timeout - how long to wait between bytes when reading response
            .read_timeout(Duration::from_secs(
                request_timeout_secs.saturating_sub(5).max(5),
            ))
            // Connection pooling - reuse connections for better performance
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            // TCP keepalive to maintain long-lived connections
            .tcp_keepalive(Duration::from_secs(60))
            // Set user agent for better API compatibility
            .user_agent("ynab-mcp/0.1.0")
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        info!(
            http_config.request_timeout_secs = request_timeout_secs,
            http_config.connect_timeout_secs = connect_timeout_secs,
            http_config.pool_max_idle = 10,
            http_config.keepalive_secs = 60,
            "HTTP client configured successfully"
        );

        Ok(Self {
            client: Arc::new(client),
            base_url,
            api_token,
            cache: YnabCache::new(cache_capacity),
        })
    }

    /// Check cache for a GET request before making HTTP call
    fn get_cached<T: DeserializeOwned>(&self, cache_key: &str) -> Option<T> {
        if let Some(cached_data) = self.cache.get(cache_key) {
            if let Ok(result) = serde_json::from_value(cached_data) {
                return Some(result);
            } else {
                warn!(
                    cache_key = cache_key,
                    "Cache hit but deserialization failed"
                );
            }
        }
        None
    }

    /// Store response in cache with TTL
    fn set_cache<T: serde::Serialize>(&self, cache_key: &str, data: &T, ttl_minutes: u64) {
        if let Ok(serialized) = serde_json::to_value(data) {
            let ttl = Duration::from_secs(ttl_minutes * 60);
            self.cache.set(cache_key.to_string(), serialized, ttl);
        }
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&self) {
        self.cache.cleanup_expired();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats()
    }

    /// Clear all cache entries
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    async fn get<T: DeserializeOwned + serde::Serialize>(&self, endpoint: &str) -> Result<T> {
        // Check cache first for read operations
        let cache_key = format!("GET:{}", endpoint);
        if let Some(cached) = self.get_cached::<T>(&cache_key) {
            return Ok(cached);
        }
        let start_time = std::time::Instant::now();

        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        let max_retries = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;

            // Only log retries, not first attempt
            if attempt > 1 {
                info!(
                    http.operation = "retry",
                    http.attempt = attempt,
                    http.max_retries = max_retries,
                    http.endpoint = endpoint,
                    "Retrying GET request"
                );
            }

            let response = self
                .client
                .get(&url)
                .send()
                .await
                .context("Failed to send request")?;

            let status = response.status();

            // Handle rate limiting with retry logic
            if status.as_u16() == 429 {
                if attempt >= max_retries {
                    error!(
                        "YNAB API rate limit exceeded after {} attempts",
                        max_retries
                    );
                    let now = chrono::Utc::now();
                    let next_available =
                        (now + chrono::Duration::minutes(15)).format("%I:%M %p UTC");
                    return Err(YnabMcpError::rate_limit(
                        &format!("YNAB API rate limit exceeded after {} retries", max_retries),
                        &format!("Next request available at: {}", next_available),
                        "YNAB enforces 200 requests per hour. Consider using cache_management tools to optimize requests, or wait 15+ minutes before retrying. Check rate limit status with health_check tool."
                    ).into());
                }

                // Exponential backoff: 1s, 2s, 4s (capped at 5s for MCP appropriateness)
                let delay_secs = std::cmp::min(1u64 << (attempt - 1), 5);
                warn!(
                    "Rate limited, retrying in {} seconds (attempt {}/{})",
                    delay_secs, attempt, max_retries
                );

                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                continue;
            }

            if !status.is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                error!("API request failed with status {}: {}", status, error_text);
                return Err(anyhow::anyhow!(
                    "API request failed: {} - {}",
                    status,
                    error_text
                ));
            }

            // Check content length and warn about very large responses
            let content_length = response.content_length();
            if let Some(size) = content_length {
                if size > 5_000_000 {
                    // 5MB
                    warn!(
                        "Large response received ({} bytes), parsing may be slow",
                        size
                    );
                }
            }

            let response_text = response
                .text()
                .await
                .context("Failed to read response body")?;

            // No need to log response size for normal operation

            // Try to parse with better error handling for large responses
            let parsed_result: Result<T> =
                serde_json::from_str(&response_text).with_context(|| {
                    // Only include first 500 chars in error to avoid memory issues
                    let truncated = if response_text.len() > 500 {
                        format!(
                            "{}... (truncated, original length: {} chars)",
                            truncate_at_char_boundary(&response_text, 500),
                            response_text.len()
                        )
                    } else {
                        response_text.clone()
                    };
                    format!("Failed to parse response: {}", truncated)
                });

            // Cache successful responses with appropriate TTL
            if let Ok(ref data) = parsed_result {
                let ttl_minutes =
                    if endpoint.contains("/budgets") && !endpoint.contains("/transactions") {
                        30 // Budget structure changes infrequently
                    } else if (endpoint.contains("/accounts") || endpoint.contains("/categories"))
                        && !endpoint.contains("/transactions")
                    {
                        15 // Account/category info changes less frequently
                    } else if endpoint.contains("/transactions") {
                        2 // Transaction data changes more frequently
                    } else {
                        5 // Default TTL for other endpoints
                    };

                self.set_cache(&cache_key, data, ttl_minutes);
            }

            // Simple logging instead of complex metrics
            let elapsed = start_time.elapsed();
            match &parsed_result {
                Ok(_) => tracing::debug!(
                    endpoint,
                    elapsed_ms = elapsed.as_millis(),
                    "API GET request completed successfully"
                ),
                Err(e) => {
                    tracing::warn!(endpoint, elapsed_ms = elapsed.as_millis(), error = %e, "API GET request failed")
                }
            }

            return parsed_result;
        }
    }

    async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        // Removed metrics - using simple tracing
        let start_time = std::time::Instant::now();

        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        // A write can change any cached read (lists, balances, the transaction itself),
        // so drop the whole cache rather than serve stale data afterwards.
        self.cache.clear();

        // POST request to YNAB API
        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 429 {
                error!("YNAB API rate limit exceeded");
                return Err(anyhow::anyhow!(
                    "YNAB API rate limit exceeded. Please wait before making more requests. The YNAB API has strict rate limits that may last for extended periods."
                ));
            }

            error!("API request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "API request failed: {} - {}",
                status,
                error_text
            ));
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Response received successfully

        let result = serde_json::from_str(&response_text).with_context(|| {
            // Only include first 500 chars in error to avoid memory issues
            let truncated = if response_text.len() > 500 {
                format!(
                    "{}... (truncated, original length: {} chars)",
                    truncate_at_char_boundary(&response_text, 500),
                    response_text.len()
                )
            } else {
                response_text.clone()
            };
            format!("Failed to parse response: {}", truncated)
        });

        // Simple logging instead of complex metrics
        let elapsed = start_time.elapsed();
        match &result {
            Ok(_) => debug!(
                endpoint,
                elapsed_ms = elapsed.as_millis(),
                "API request completed successfully"
            ),
            Err(e) => {
                warn!(endpoint, elapsed_ms = elapsed.as_millis(), error = %e, "API request failed")
            }
        }

        result
    }

    async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        // Removed metrics - using simple tracing
        let start_time = std::time::Instant::now();

        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        // A write can change any cached read, so drop the whole cache (see post()).
        self.cache.clear();

        // PATCH request to YNAB API
        let response = self
            .client
            .patch(&url)
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 429 {
                error!("YNAB API rate limit exceeded");
                return Err(anyhow::anyhow!(
                    "YNAB API rate limit exceeded. Please wait before making more requests. The YNAB API has strict rate limits that may last for extended periods."
                ));
            }

            error!("API request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "API request failed: {} - {}",
                status,
                error_text
            ));
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Response received successfully

        let result = serde_json::from_str(&response_text).with_context(|| {
            // Only include first 500 chars in error to avoid memory issues
            let truncated = if response_text.len() > 500 {
                format!(
                    "{}... (truncated, original length: {} chars)",
                    truncate_at_char_boundary(&response_text, 500),
                    response_text.len()
                )
            } else {
                response_text.clone()
            };
            format!("Failed to parse response: {}", truncated)
        });

        // Simple logging instead of complex metrics
        let elapsed = start_time.elapsed();
        match &result {
            Ok(_) => debug!(
                endpoint,
                elapsed_ms = elapsed.as_millis(),
                "API request completed successfully"
            ),
            Err(e) => {
                warn!(endpoint, elapsed_ms = elapsed.as_millis(), error = %e, "API request failed")
            }
        }

        result
    }

    // User methods
    pub async fn get_user(&self) -> Result<User> {
        let response: ApiResponse<UserResponse> = self.get("user").await?;
        Ok(response.data.user)
    }

    // Budget methods
    pub async fn get_budgets(&self) -> Result<Vec<Budget>> {
        let response: ApiResponse<BudgetsResponse> = self.get("budgets").await?;
        Ok(response.data.budgets)
    }

    // Account methods
    pub async fn get_accounts(&self, budget_id: &str) -> Result<Vec<Account>> {
        let endpoint = format!("budgets/{}/accounts", budget_id);
        let response: ApiResponse<AccountsResponse> = self.get(&endpoint).await?;
        Ok(response.data.accounts)
    }

    pub async fn get_account(&self, budget_id: &str, account_id: Uuid) -> Result<Account> {
        let endpoint = format!("budgets/{}/accounts/{}", budget_id, account_id);
        let response: ApiResponse<AccountResponse> = self.get(&endpoint).await?;
        Ok(response.data.account)
    }

    // Category methods
    pub async fn get_categories(&self, budget_id: &str) -> Result<Vec<CategoryGroup>> {
        let endpoint = format!("budgets/{}/categories", budget_id);
        let response: ApiResponse<CategoriesResponse> = self.get(&endpoint).await?;
        Ok(response.data.category_groups)
    }

    pub async fn get_category(&self, budget_id: &str, category_id: Uuid) -> Result<Category> {
        let endpoint = format!("budgets/{}/categories/{}", budget_id, category_id);
        let response: ApiResponse<CategoryResponse> = self.get(&endpoint).await?;
        Ok(response.data.category)
    }

    // Payee methods
    pub async fn get_payees(&self, budget_id: &str) -> Result<Vec<Payee>> {
        let endpoint = format!("budgets/{}/payees", budget_id);
        let response: ApiResponse<PayeesResponse> = self.get(&endpoint).await?;
        Ok(response.data.payees)
    }

    pub async fn get_payee(&self, budget_id: &str, payee_id: Uuid) -> Result<Payee> {
        let endpoint = format!("budgets/{}/payees/{}", budget_id, payee_id);
        let response: ApiResponse<PayeeResponse> = self.get(&endpoint).await?;
        Ok(response.data.payee)
    }

    // Transaction methods
    pub async fn get_transactions(&self, budget_id: &str) -> Result<Vec<Transaction>> {
        let endpoint = format!("budgets/{}/transactions", budget_id);
        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;
        let transactions = response.data.transactions;

        Ok(transactions)
    }

    pub async fn get_transactions_delta(
        &self,
        budget_id: &str,
        server_knowledge: i64,
    ) -> Result<(Vec<Transaction>, i64)> {
        let endpoint = format!(
            "budgets/{}/transactions?last_knowledge_of_server={}",
            budget_id, server_knowledge
        );
        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;
        let transactions = response.data.transactions;

        Ok((transactions, response.data.server_knowledge))
    }

    pub async fn get_transactions_since_date(
        &self,
        budget_id: &str,
        since_date: &str,
    ) -> Result<Vec<Transaction>> {
        let endpoint = format!(
            "budgets/{}/transactions?since_date={}",
            budget_id, since_date
        );
        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;
        let transactions = response.data.transactions;

        Ok(transactions)
    }

    pub async fn get_transactions_paginated(
        &self,
        budget_id: &str,
        limit: u32,
        offset: Option<u32>,
    ) -> Result<(Vec<Transaction>, bool)> {
        let mut query_params = vec![format!("limit={}", limit)];

        if let Some(offset_val) = offset {
            query_params.push(format!("offset={}", offset_val));
        }

        let endpoint = format!(
            "budgets/{}/transactions?{}",
            budget_id,
            query_params.join("&")
        );

        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;

        // Check if we have more data (if we got exactly the limit requested)
        let has_more_data = response.data.transactions.len() as u32 >= limit;

        Ok((response.data.transactions, has_more_data))
    }

    pub async fn get_transactions_with_fields(
        &self,
        budget_id: &str,
        fields: Option<&[&str]>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut query_params = Vec::new();

        if let Some(limit_val) = limit {
            query_params.push(format!("limit={}", limit_val));
        }

        if let Some(offset_val) = offset {
            query_params.push(format!("offset={}", offset_val));
        }

        let endpoint = if query_params.is_empty() {
            format!("budgets/{}/transactions", budget_id)
        } else {
            format!(
                "budgets/{}/transactions?{}",
                budget_id,
                query_params.join("&")
            )
        };

        // Get full transactions first (YNAB API doesn't support field selection server-side)
        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;

        // Apply client-side field filtering if fields are specified
        if let Some(field_list) = fields {
            let filtered_transactions: Vec<serde_json::Value> = response
                .data
                .transactions
                .into_iter()
                .map(|transaction| {
                    // Convert transaction to JSON value first
                    let mut transaction_json =
                        serde_json::to_value(&transaction).unwrap_or(serde_json::Value::Null);

                    if let serde_json::Value::Object(ref mut map) = transaction_json {
                        // Keep only requested fields
                        let field_set: std::collections::HashSet<&str> =
                            field_list.iter().copied().collect();
                        map.retain(|key, _| field_set.contains(key.as_str()));

                        // Always include id for reference
                        if !field_set.contains("id") {
                            if let Ok(original_transaction) = serde_json::to_value(&transaction) {
                                if let Some(id) = original_transaction.get("id") {
                                    map.insert("id".to_string(), id.clone());
                                }
                            }
                        }
                    }

                    transaction_json
                })
                .collect();

            Ok(filtered_transactions)
        } else {
            // Return all fields as JSON values
            let all_transactions: Vec<serde_json::Value> = response
                .data
                .transactions
                .into_iter()
                .map(|transaction| {
                    serde_json::to_value(&transaction).unwrap_or(serde_json::Value::Null)
                })
                .collect();

            Ok(all_transactions)
        }
    }

    pub async fn get_transactions_by_account(
        &self,
        budget_id: &str,
        account_id: Uuid,
    ) -> Result<Vec<Transaction>> {
        let endpoint = format!("budgets/{}/accounts/{}/transactions", budget_id, account_id);
        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;
        let transactions = response.data.transactions;

        Ok(transactions)
    }

    pub async fn get_transactions_by_category(
        &self,
        budget_id: &str,
        category_id: Uuid,
    ) -> Result<Vec<Transaction>> {
        let endpoint = format!(
            "budgets/{}/categories/{}/transactions",
            budget_id, category_id
        );
        let response: ApiResponse<TransactionsResponse> = self.get(&endpoint).await?;
        let transactions = response.data.transactions;

        Ok(transactions)
    }

    pub async fn get_transaction(
        &self,
        budget_id: &str,
        transaction_id: Uuid,
    ) -> Result<Transaction> {
        let endpoint = format!("budgets/{}/transactions/{}", budget_id, transaction_id);
        // Always fetch fresh: approve/clear/update rebuild the full transaction from
        // this result and send it back, so a cached copy could overwrite edits made
        // in the YNAB app since it was cached.
        self.cache.remove(&format!("GET:{}", endpoint));
        let response: ApiResponse<TransactionResponse> = self.get(&endpoint).await?;
        Ok(response.data.transaction)
    }

    pub async fn create_transaction(
        &self,
        budget_id: &str,
        transaction: NewTransaction,
    ) -> Result<Vec<Transaction>> {
        let endpoint = format!("budgets/{}/transactions", budget_id);
        let response: ApiResponse<TransactionsResponse> =
            self.post(&endpoint, &transaction).await?;
        Ok(response.data.transactions)
    }

    pub async fn update_transaction(
        &self,
        budget_id: &str,
        transaction_id: Uuid,
        transaction: UpdateTransaction,
    ) -> Result<Transaction> {
        let endpoint = format!("budgets/{}/transactions/{}", budget_id, transaction_id);
        let response: ApiResponse<TransactionResponse> =
            self.patch(&endpoint, &transaction).await?;
        Ok(response.data.transaction)
    }

    // Scheduled transaction methods
    pub async fn get_scheduled_transactions(
        &self,
        budget_id: &str,
    ) -> Result<Vec<ScheduledTransaction>> {
        let endpoint = format!("budgets/{}/scheduled_transactions", budget_id);
        let response: ApiResponse<ScheduledTransactionsResponse> = self.get(&endpoint).await?;
        Ok(response.data.scheduled_transactions)
    }

    pub async fn get_scheduled_transaction(
        &self,
        budget_id: &str,
        scheduled_transaction_id: Uuid,
    ) -> Result<ScheduledTransaction> {
        let endpoint = format!(
            "budgets/{}/scheduled_transactions/{}",
            budget_id, scheduled_transaction_id
        );
        let response: ApiResponse<ScheduledTransactionResponse> = self.get(&endpoint).await?;
        Ok(response.data.scheduled_transaction)
    }

    // Month methods (budget data for specific months)
    pub async fn get_months(&self, budget_id: &str) -> Result<Vec<Month>> {
        let endpoint = format!("budgets/{}/months", budget_id);
        let response: ApiResponse<MonthsResponse> = self.get(&endpoint).await?;
        Ok(response.data.months)
    }

    pub async fn get_month(&self, budget_id: &str, month: &str) -> Result<Month> {
        let endpoint = format!("budgets/{}/months/{}", budget_id, month);
        let response: ApiResponse<MonthResponse> = self.get(&endpoint).await?;
        Ok(response.data.month)
    }

    pub async fn update_month_category(
        &self,
        budget_id: &str,
        month: &str,
        category_id: uuid::Uuid,
        budgeted_amount: i64,
    ) -> Result<crate::ynab::models::Category> {
        let endpoint = format!(
            "budgets/{}/months/{}/categories/{}",
            budget_id, month, category_id
        );
        let request_body = serde_json::json!({
            "category": {
                "budgeted": budgeted_amount
            }
        });

        #[derive(serde::Deserialize, serde::Serialize)]
        struct CategoryResponse {
            category: crate::ynab::models::Category,
        }

        let response: ApiResponse<CategoryResponse> = self.patch(&endpoint, &request_body).await?;
        Ok(response.data.category)
    }

    /// Perform a health check against the YNAB API
    /// Returns health status including response time and any errors
    pub async fn health_check(&self) -> HealthStatus {
        let start_time = Instant::now();

        match self.get_user().await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                info!("YNAB API health check passed ({}ms)", response_time);

                HealthStatus {
                    healthy: true,
                    response_time_ms: response_time,
                    last_check: Instant::now(),
                    error_message: None,
                }
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                let error_msg = e.to_string();
                warn!(
                    "YNAB API health check failed ({}ms): {}",
                    response_time, error_msg
                );

                HealthStatus {
                    healthy: false,
                    response_time_ms: response_time,
                    last_check: Instant::now(),
                    error_message: Some(error_msg),
                }
            }
        }
    }

    /// Perform a lightweight connection test using the /user endpoint
    /// This is faster than a full health check as it doesn't validate all functionality
    pub async fn connection_test(&self) -> Result<Duration> {
        let start_time = Instant::now();

        let url = format!("{}/user", self.base_url.trim_end_matches('/'));

        // Testing YNAB API connection

        let response = self
            .client
            .head(&url)
            .send()
            .await
            .context("Failed to connect to YNAB API")?;

        let elapsed = start_time.elapsed();

        if response.status().is_success() {
            info!("YNAB API connection verified ({} ms)", elapsed.as_millis());
            Ok(elapsed)
        } else {
            error!("Connection test failed with status: {}", response.status());
            Err(anyhow::anyhow!(
                "Connection test failed: {}",
                response.status()
            ))
        }
    }

    /// Check if the API is experiencing rate limiting
    /// Returns true if we're being rate limited, false otherwise
    pub async fn check_rate_limit(&self) -> Result<bool> {
        // Make a minimal request to check rate limit headers
        let url = format!("{}/user", self.base_url.trim_end_matches('/'));

        let response = self
            .client
            .head(&url)
            .send()
            .await
            .context("Failed to check rate limit status")?;

        // Check for rate limiting (429 status or rate limit headers)
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            warn!("YNAB API rate limit exceeded");
            return Ok(true);
        }

        // Check rate limit headers if present
        if let Some(remaining) = response.headers().get("X-Rate-Limit-Remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_count) = remaining_str.parse::<u32>() {
                    if remaining_count <= 5 {
                        warn!(
                            "YNAB API approaching rate limit: {} requests remaining",
                            remaining_count
                        );
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

impl std::fmt::Debug for YnabClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YnabClient")
            .field("base_url", &self.base_url)
            .field("api_token", &"[REDACTED]")
            .field("client", &"[HTTP Client]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_at_char_boundary() {
        assert_eq!(truncate_at_char_boundary("short", 500), "short");
        assert_eq!(truncate_at_char_boundary("abcdef", 3), "abc");
        // "é" is two bytes; cutting at byte 2 would split it.
        let s = "aé"; // bytes: 'a' (1) + 'é' (2)
        assert_eq!(truncate_at_char_boundary(s, 2), "a");
        let long = "é".repeat(400); // 800 bytes, every odd offset is mid-char
        assert_eq!(truncate_at_char_boundary(&long, 501).len(), 500);
    }
    use crate::ynab::models::{AccountType, ClearState, TransactionDetail};
    use serde_json::json;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_client_creation() {
        let client = YnabClient::new_for_test(
            "https://api.ynab.com/v1".to_string(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        assert_eq!(client.base_url, "https://api.ynab.com/v1");
        // Note: api_token is private for security, so we can't test it directly
    }

    #[tokio::test]
    async fn test_client_creation_with_invalid_token() {
        let result = YnabClient::new_for_test(
            "https://api.ynab.com/v1".to_string(),
            YnabApiToken::new("invalid\ntoken".to_string()),
        );

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_user() {
        let mock_server = MockServer::start().await;
        let user_response = json!({
            "data": {
                "user": {
                    "id": "123e4567-e89b-12d3-a456-426614174000"
                }
            }
        });

        Mock::given(method("GET"))
            .and(path("/user"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&user_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let user = client.get_user().await.unwrap();
        assert_eq!(
            user.id,
            Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_budgets() {
        let mock_server = MockServer::start().await;
        let budgets_response = json!({
            "data": {
                "budgets": [{
                    "id": "123e4567-e89b-12d3-a456-426614174000",
                    "name": "Test Budget",
                    "last_modified_on": "2023-01-01T00:00:00Z",
                    "first_month": "2023-01-01",
                    "last_month": "2023-12-01",
                    "date_format": {
                        "format": "MM/DD/YYYY"
                    },
                    "currency_format": {
                        "iso_code": "USD",
                        "example_format": "$123.45",
                        "decimal_digits": 2,
                        "decimal_separator": ".",
                        "symbol_first": true,
                        "group_separator": ",",
                        "currency_symbol": "$",
                        "display_symbol": true
                    }
                }],
                "default_budget": null
            }
        });

        Mock::given(method("GET"))
            .and(path("/budgets"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&budgets_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let budgets = client.get_budgets().await.unwrap();
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].name, "Test Budget");
    }

    #[tokio::test]
    async fn test_get_accounts() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";
        let accounts_response = json!({
            "data": {
                "accounts": [{
                    "id": "456e7890-e89b-12d3-a456-426614174000",
                    "name": "Checking Account",
                    "type": "checking",
                    "on_budget": true,
                    "closed": false,
                    "note": null,
                    "balance": 100000,
                    "cleared_balance": 95000,
                    "uncleared_balance": 5000,
                    "transfer_payee_id": "00000000-0000-0000-0000-000000000000",
                    "direct_import_linked": false,
                    "direct_import_in_error": false,
                    "last_reconciled_at": null,
                    "debt_original_balance": null,
                    "debt_interest_rates": {},
                    "debt_minimum_payments": {},
                    "debt_escrow_amounts": {},
                    "deleted": false
                }],
                "server_knowledge": 1
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/accounts", budget_id)))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&accounts_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let accounts = client.get_accounts(budget_id).await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Checking Account");
        assert_eq!(accounts[0].account_type, AccountType::Checking);
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let result = client.get_user().await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("401"));
    }

    #[tokio::test]
    async fn test_create_transaction() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";
        let transaction_response = json!({
            "data": {
                "transactions": [{
                    "id": "789e0123-e89b-12d3-a456-426614174000",
                    "date": "2023-01-01",
                    "amount": -5000,
                    "memo": "Test transaction",
                    "cleared": "uncleared",
                    "approved": true,
                    "flag_color": null,
                    "flag_name": null,
                    "account_id": "456e7890-e89b-12d3-a456-426614174000",
                    "account_name": "Checking Account",
                    "payee_id": "abc12345-e89b-12d3-a456-426614174000",
                    "payee_name": "Test Payee",
                    "category_id": "def67890-e89b-12d3-a456-426614174000",
                    "category_name": "Test Category",
                    "transfer_account_id": null,
                    "transfer_transaction_id": null,
                    "matched_transaction_id": null,
                    "import_id": null,
                    "import_payee_name": null,
                    "import_payee_name_original": null,
                    "debt_transaction_type": null,
                    "deleted": false,
                    "transaction_type": null,
                    "parent_transaction_id": null,
                    "subtransactions": []
                }],
                "server_knowledge": 1
            }
        });

        Mock::given(method("POST"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(201).set_body_json(&transaction_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let transaction_detail = TransactionDetail {
            account_id: Uuid::parse_str("456e7890-e89b-12d3-a456-426614174000").unwrap(),
            payee_id: Some(Uuid::parse_str("abc12345-e89b-12d3-a456-426614174000").unwrap()),
            category_id: Some(Uuid::parse_str("def67890-e89b-12d3-a456-426614174000").unwrap()),
            memo: Some("Test transaction".to_string()),
            date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            amount: -5000,
            cleared: Some(ClearState::Uncleared),
            approved: Some(true),
            flag_color: None,
            flag_name: None,
            import_id: None,
        };

        let new_transaction = NewTransaction {
            transactions: vec![transaction_detail],
        };

        let transactions = client
            .create_transaction(budget_id, new_transaction)
            .await
            .unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].memo, Some("Test transaction".to_string()));
    }

    #[tokio::test]
    async fn test_get_transactions_paginated_first_page() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        // Mock response with 3 transactions (first page of limit=2)
        let transactions_response = json!({
            "data": {
                "transactions": [
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174001",
                        "date": "2023-01-03",
                        "amount": -1000,
                        "memo": "Transaction 3",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174001",
                        "payee_name": "Payee 3",
                        "category_id": "def67890-e89b-12d3-a456-426614174001",
                        "category_name": "Category 3",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    },
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174002",
                        "date": "2023-01-02",
                        "amount": -2000,
                        "memo": "Transaction 2",
                        "cleared": "uncleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174002",
                        "payee_name": "Payee 2",
                        "category_id": "def67890-e89b-12d3-a456-426614174002",
                        "category_name": "Category 2",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    }
                ],
                "server_knowledge": 5
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(query_param("limit", "2"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let (transactions, has_more) = client
            .get_transactions_paginated(budget_id, 2, None)
            .await
            .unwrap();

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].memo, Some("Transaction 3".to_string()));
        assert_eq!(transactions[1].memo, Some("Transaction 2".to_string()));
        assert!(has_more); // Exactly 2 transactions returned (matches limit), indicating more might be available
    }

    #[tokio::test]
    async fn test_get_transactions_paginated_with_offset() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        // Mock response for page 2 (offset=2, limit=2)
        let transactions_response = json!({
            "data": {
                "transactions": [
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174003",
                        "date": "2023-01-01",
                        "amount": -3000,
                        "memo": "Transaction 1",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174003",
                        "payee_name": "Payee 1",
                        "category_id": "def67890-e89b-12d3-a456-426614174003",
                        "category_name": "Category 1",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    }
                ],
                "server_knowledge": 5
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(query_param("limit", "2"))
            .and(query_param("offset", "2"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let (transactions, has_more) = client
            .get_transactions_paginated(budget_id, 2, Some(2))
            .await
            .unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].memo, Some("Transaction 1".to_string()));
        assert!(!has_more); // Only 1 transaction returned (less than limit), so no more
    }

    #[tokio::test]
    async fn test_get_transactions_paginated_has_more_pages() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        // Mock response with exactly the limit (indicating more pages available)
        let transactions_response = json!({
            "data": {
                "transactions": [
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174001",
                        "date": "2023-01-05",
                        "amount": -1000,
                        "memo": "Transaction 5",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174001",
                        "payee_name": "Payee 5",
                        "category_id": "def67890-e89b-12d3-a456-426614174001",
                        "category_name": "Category 5",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    },
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174002",
                        "date": "2023-01-04",
                        "amount": -2000,
                        "memo": "Transaction 4",
                        "cleared": "uncleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174002",
                        "payee_name": "Payee 4",
                        "category_id": "def67890-e89b-12d3-a456-426614174002",
                        "category_name": "Category 4",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    },
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174003",
                        "date": "2023-01-03",
                        "amount": -3000,
                        "memo": "Transaction 3",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174003",
                        "payee_name": "Payee 3",
                        "category_id": "def67890-e89b-12d3-a456-426614174003",
                        "category_name": "Category 3",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    }
                ],
                "server_knowledge": 10
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(query_param("limit", "3"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let (transactions, has_more) = client
            .get_transactions_paginated(budget_id, 3, None)
            .await
            .unwrap();

        assert_eq!(transactions.len(), 3);
        assert_eq!(transactions[0].memo, Some("Transaction 5".to_string()));
        assert_eq!(transactions[1].memo, Some("Transaction 4".to_string()));
        assert_eq!(transactions[2].memo, Some("Transaction 3".to_string()));
        assert!(has_more); // Exactly 3 transactions returned (matches limit), indicating more available
    }

    #[tokio::test]
    async fn test_get_transactions_paginated_empty_result() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        // Mock empty response
        let transactions_response = json!({
            "data": {
                "transactions": [],
                "server_knowledge": 1
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(query_param("limit", "10"))
            .and(query_param("offset", "100"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        let (transactions, has_more) = client
            .get_transactions_paginated(budget_id, 10, Some(100))
            .await
            .unwrap();

        assert_eq!(transactions.len(), 0);
        assert!(!has_more); // No transactions returned, so no more pages
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_basic_filtering() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        let transactions_response = json!({
            "data": {
                "transactions": [
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174001",
                        "date": "2023-01-03",
                        "amount": -1500,
                        "memo": "Test Transaction",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": "red",
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174001",
                        "payee_name": "Test Payee",
                        "category_id": "def67890-e89b-12d3-a456-426614174001",
                        "category_name": "Test Category",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    }
                ],
                "server_knowledge": 50
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        // Test with specific fields
        let fields = ["id", "date", "amount", "memo"];
        let transactions = client
            .get_transactions_with_fields(budget_id, Some(&fields), None, None)
            .await
            .unwrap();

        assert_eq!(transactions.len(), 1);

        // Check that only requested fields (plus id) are present
        let transaction = &transactions[0];
        assert!(transaction.get("id").is_some());
        assert!(transaction.get("date").is_some());
        assert!(transaction.get("amount").is_some());
        assert!(transaction.get("memo").is_some());

        // These fields should not be present (filtered out)
        assert!(transaction.get("account_name").is_none());
        assert!(transaction.get("payee_name").is_none());
        assert!(transaction.get("flag_color").is_none());
        assert!(transaction.get("cleared").is_none());
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_no_filtering() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        let transactions_response = json!({
            "data": {
                "transactions": [
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174001",
                        "date": "2023-01-03",
                        "amount": -1500,
                        "memo": "Test Transaction",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174001",
                        "payee_name": "Test Payee",
                        "category_id": "def67890-e89b-12d3-a456-426614174001",
                        "category_name": "Test Category",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    }
                ],
                "server_knowledge": 50
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        // Test without field filtering (should return all fields)
        let transactions = client
            .get_transactions_with_fields(budget_id, None, None, None)
            .await
            .unwrap();

        assert_eq!(transactions.len(), 1);

        // All major fields should be present
        let transaction = &transactions[0];
        assert!(transaction.get("id").is_some());
        assert!(transaction.get("date").is_some());
        assert!(transaction.get("amount").is_some());
        assert!(transaction.get("memo").is_some());
        assert!(transaction.get("account_name").is_some());
        assert!(transaction.get("payee_name").is_some());
        assert!(transaction.get("category_name").is_some());
        assert!(transaction.get("cleared").is_some());
        assert!(transaction.get("approved").is_some());
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_with_pagination() {
        let mock_server = MockServer::start().await;
        let budget_id = "123e4567-e89b-12d3-a456-426614174000";

        let transactions_response = json!({
            "data": {
                "transactions": [
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174001",
                        "date": "2023-01-03",
                        "amount": -1500,
                        "memo": "Transaction 1",
                        "cleared": "cleared",
                        "approved": true,
                        "flag_color": null,
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174001",
                        "payee_name": "Test Payee 1",
                        "category_id": "def67890-e89b-12d3-a456-426614174001",
                        "category_name": "Test Category 1",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    },
                    {
                        "id": "789e0123-e89b-12d3-a456-426614174002",
                        "date": "2023-01-02",
                        "amount": -2000,
                        "memo": "Transaction 2",
                        "cleared": "uncleared",
                        "approved": true,
                        "flag_color": "blue",
                        "flag_name": null,
                        "account_id": "456e7890-e89b-12d3-a456-426614174000",
                        "account_name": "Checking Account",
                        "payee_id": "abc12345-e89b-12d3-a456-426614174002",
                        "payee_name": "Test Payee 2",
                        "category_id": "def67890-e89b-12d3-a456-426614174002",
                        "category_name": "Test Category 2",
                        "transfer_account_id": null,
                        "transfer_transaction_id": null,
                        "matched_transaction_id": null,
                        "import_id": null,
                        "import_payee_name": null,
                        "import_payee_name_original": null,
                        "debt_transaction_type": null,
                        "deleted": false,
                        "transaction_type": null,
                        "parent_transaction_id": null,
                        "subtransactions": []
                    }
                ],
                "server_knowledge": 50
            }
        });

        Mock::given(method("GET"))
            .and(path(format!("/budgets/{}/transactions", budget_id)))
            .and(query_param("limit", "2"))
            .and(query_param("offset", "5"))
            .and(header("authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&transactions_response))
            .mount(&mock_server)
            .await;

        let client = YnabClient::new_for_test(
            mock_server.uri(),
            YnabApiToken::new("test_token".to_string()),
        )
        .unwrap();

        // Test with field filtering AND pagination
        let fields = ["id", "date", "amount", "memo", "payee_name"];
        let transactions = client
            .get_transactions_with_fields(budget_id, Some(&fields), Some(2), Some(5))
            .await
            .unwrap();

        assert_eq!(transactions.len(), 2);

        // Check first transaction has only requested fields
        let transaction1 = &transactions[0];
        assert!(transaction1.get("id").is_some());
        assert!(transaction1.get("date").is_some());
        assert!(transaction1.get("amount").is_some());
        assert!(transaction1.get("memo").is_some());
        assert!(transaction1.get("payee_name").is_some());

        // These should be filtered out
        assert!(transaction1.get("account_name").is_none());
        assert!(transaction1.get("flag_color").is_none());
        assert!(transaction1.get("cleared").is_none());

        // Check second transaction structure
        let transaction2 = &transactions[1];
        assert_eq!(
            transaction2.get("memo").unwrap().as_str().unwrap(),
            "Transaction 2"
        );
        assert_eq!(
            transaction2.get("payee_name").unwrap().as_str().unwrap(),
            "Test Payee 2"
        );
    }
}
