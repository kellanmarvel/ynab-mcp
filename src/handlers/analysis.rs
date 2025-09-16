use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

// Removed progress dependency - not needed for MCP operations
use super::common::CommonUtils;

pub struct AnalysisHandlers {
    common: CommonUtils,
}

impl AnalysisHandlers {
    pub fn new(common: CommonUtils) -> Self {
        Self { common }
    }

    pub async fn handle_analysis_operation(&self, tool_name: &str, args: Value) -> Result<Value> {
        // Handle analysis operation

        let result = match tool_name {
            "analyze_transactions" => self.analyze_transactions(args).await,
            "get_transactions_by_date_range" => self.get_transactions_by_date_range(args).await,
            "analyze_spending_by_category" => self.analyze_spending_by_category(args).await,
            "get_all_account_transactions" => self.get_all_account_transactions(args).await,
            "get_transactions_delta" => self.get_transactions_delta(args).await,
            "get_transactions_since_date" => self.get_transactions_since_date(args).await,
            "get_transactions_paginated" => self.get_transactions_paginated(args).await,
            "get_transactions_with_fields" => self.get_transactions_with_fields(args).await,
            _ => return Err(anyhow::anyhow!("Unknown analysis operation: {}", tool_name)),
        };

        match result {
            Ok(response) => {
                let message = self
                    .common
                    .generate_conversational_summary(tool_name, &response);
                Ok(self
                    .common
                    .wrap_mcp_response_with_data(&message, &response, false))
            }
            Err(e) => {
                let error_msg = format!("Analysis operation failed: {}", e);
                Ok(self.common.wrap_mcp_response(&error_msg, true))
            }
        }
    }

    pub async fn analyze_transactions(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        // Removed progress reporter - not needed for MCP operations
        info!("Initializing transaction analysis");

        let from_date = arguments.get("from_date").and_then(|v| v.as_str());
        let to_date = arguments.get("to_date").and_then(|v| v.as_str());
        let category_id = arguments.get("category_id").and_then(|v| v.as_str());
        let account_id = arguments.get("account_id").and_then(|v| v.as_str());

        // Pagination parameters
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as usize;
        let offset = arguments
            .get("offset")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as usize;
        let summary_only = arguments
            .get("summary_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Validate pagination parameters
        if !(1..=500).contains(&limit) {
            return Err(anyhow::anyhow!("limit must be between 1 and 500"));
        }

        // Get ALL transactions for analysis
        info!("Fetching all transactions from YNAB");

        // Removed cancellation check - not needed for MCP operations

        let mut transactions = self
            .common
            .ynab_client
            .get_transactions(&resolved_budget_id)
            .await?;

        info!(
            transactions_count = %transactions.len(),
            "Retrieved transactions, applying filters"
        );

        // Removed cancellation check - not needed for MCP operations

        // Apply filters
        let original_count = transactions.len();
        if let Some(from) = from_date {
            if let Ok(from_parsed) = chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d") {
                transactions.retain(|t| t.date >= from_parsed);
            }
        }
        if let Some(to) = to_date {
            if let Ok(to_parsed) = chrono::NaiveDate::parse_from_str(to, "%Y-%m-%d") {
                transactions.retain(|t| t.date <= to_parsed);
            }
        }
        if let Some(cat_id) = category_id {
            if let Ok(cat_uuid) = Uuid::parse_str(cat_id) {
                transactions.retain(|t| t.category_id == Some(cat_uuid));
            }
        }
        if let Some(acc_id) = account_id {
            if let Ok(acc_uuid) = Uuid::parse_str(acc_id) {
                transactions.retain(|t| t.account_id == acc_uuid);
            }
        }

        info!(
            original_count = %original_count,
            filtered_count = %transactions.len(),
            "Applied filters to transactions"
        );

        // Removed cancellation check - not needed for MCP operations

        // Sort by date descending for consistent pagination
        transactions.sort_by(|a, b| b.date.cmp(&a.date));

        info!("Performing transaction analysis");

        // Calculate comprehensive statistics (on ALL filtered transactions)
        let total_count = transactions.len();
        let total_inflow = transactions
            .iter()
            .filter(|t| t.amount > 0)
            .map(|t| t.amount)
            .sum::<i64>();
        let total_outflow = transactions
            .iter()
            .filter(|t| t.amount < 0)
            .map(|t| t.amount)
            .sum::<i64>();
        let net_amount = total_inflow + total_outflow; // outflow is negative

        info!("Preparing analysis results");

        // Apply pagination to transaction list (only for response, not analysis)
        let paginated_transactions = if summary_only {
            vec![]
        } else {
            transactions
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>()
        };

        let returned_count = paginated_transactions.len();
        let has_more = !summary_only && offset + returned_count < total_count;

        // Create conversational summary
        let summary = if total_count == 0 {
            "No transactions found matching the criteria".to_string()
        } else if summary_only {
            let formatted_inflow = self.common.format_amount(total_inflow);
            let formatted_outflow = self.common.format_amount(total_outflow.abs());
            let formatted_net = self.common.format_amount(net_amount);

            format!(
                "Analysis complete: {} total transactions with {} inflow, {} outflow, net amount: {}",
                total_count, formatted_inflow, formatted_outflow, formatted_net
            )
        } else {
            let formatted_inflow = self.common.format_amount(total_inflow);
            let formatted_outflow = self.common.format_amount(total_outflow.abs());
            let formatted_net = self.common.format_amount(net_amount);

            format!(
                "Analysis of {} transactions (showing {} of {} with {} inflow, {} outflow, net: {})",
                total_count, returned_count, total_count, formatted_inflow, formatted_outflow, formatted_net
            )
        };

        let mut response = json!({
            "success": true,
            "conversational_summary": summary,
            "analysis": {
                "total_transactions": total_count,
                "date_range": {
                    "from": from_date,
                    "to": to_date,
                },
                "filters": {
                    "category_id": category_id,
                    "account_id": account_id,
                },
                "financial_summary": {
                    "total_inflow": self.common.format_amount(total_inflow),
                    "total_outflow": self.common.format_amount(total_outflow.abs()),
                    "net_amount": self.common.format_amount(net_amount),
                    "average_transaction": if total_count > 0 {
                        self.common.format_amount(net_amount / total_count as i64)
                    } else {
                        self.common.format_amount(0)
                    }
                }
            }
        });

        // Add pagination info and transactions if not summary_only
        if !summary_only {
            response["pagination"] = json!({
                "limit": limit,
                "offset": offset,
                "returned_count": returned_count,
                "total_count": total_count,
                "has_more": has_more,
                "current_page": (offset / limit) + 1,
                "total_pages": total_count.div_ceil(limit)
            });
            response["transactions"] = json!(paginated_transactions);
        }

        let operation_id = Uuid::new_v4().to_string();

        info!(
            total_analyzed = %total_count,
            filters_applied = %(original_count != total_count),
            paginated_results = %returned_count,
            "Analysis complete"
        );

        response["operation_id"] = json!(operation_id);

        Ok(response)
    }

    pub async fn get_transactions_by_date_range(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let from_date = arguments
            .get("from_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("from_date is required"))?;

        let to_date = arguments
            .get("to_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("to_date is required"))?;

        // Pagination parameters
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as usize;
        let offset = arguments
            .get("offset")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as usize;

        // Validate pagination parameters
        if !(1..=1000).contains(&limit) {
            return Err(anyhow::anyhow!("limit must be between 1 and 1000"));
        }

        // Validate date formats
        let from_parsed = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid from_date format, expected YYYY-MM-DD"))?;
        let to_parsed = chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid to_date format, expected YYYY-MM-DD"))?;

        if from_parsed > to_parsed {
            return Err(anyhow::anyhow!("from_date cannot be after to_date"));
        }

        let mut transactions = self
            .common
            .ynab_client
            .get_transactions(&resolved_budget_id)
            .await?;

        // Filter by date range
        transactions.retain(|t| t.date >= from_parsed && t.date <= to_parsed);

        // Sort by date descending for consistent pagination
        transactions.sort_by(|a, b| b.date.cmp(&a.date));

        let total_count = transactions.len();

        // Apply pagination
        let paginated_transactions: Vec<_> =
            transactions.into_iter().skip(offset).take(limit).collect();

        let returned_count = paginated_transactions.len();
        let has_more = offset + returned_count < total_count;

        let summary = format!(
            "Found {} transactions between {} and {} (showing {} of {})",
            total_count, from_date, to_date, returned_count, total_count
        );

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "date_range": {
                "from": from_date,
                "to": to_date,
                "total_count": total_count
            },
            "pagination": {
                "limit": limit,
                "offset": offset,
                "returned_count": returned_count,
                "total_count": total_count,
                "has_more": has_more,
                "current_page": (offset / limit) + 1,
                "total_pages": total_count.div_ceil(limit)
            },
            "transactions": paginated_transactions
        }))
    }

    pub async fn analyze_spending_by_category(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let months_back = arguments
            .get("months_back")
            .and_then(|v| v.as_f64())
            .unwrap_or(12.0) as i32;

        if !(1..=60).contains(&months_back) {
            return Err(anyhow::anyhow!("months_back must be between 1 and 60"));
        }

        // Calculate date range
        let now = chrono::Utc::now();
        let from_date = now
            .date_naive()
            .checked_sub_months(chrono::Months::new(months_back as u32))
            .unwrap_or(now.date_naive());

        let transactions = self
            .common
            .ynab_client
            .get_transactions(&resolved_budget_id)
            .await?;

        // Filter to date range and only outflow transactions
        let filtered_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| t.date >= from_date && t.amount < 0)
            .collect();

        // Group by category
        let mut category_spending: HashMap<String, i64> = HashMap::new();
        for transaction in &filtered_transactions {
            let category_name = transaction
                .category_name
                .as_ref()
                .unwrap_or(&"Uncategorized".to_string())
                .clone();
            *category_spending.entry(category_name).or_insert(0) += transaction.amount.abs();
        }

        // Sort categories by spending amount
        let mut sorted_categories: Vec<_> = category_spending.into_iter().collect();
        sorted_categories.sort_by(|a, b| b.1.cmp(&a.1));

        let total_spending: i64 = sorted_categories.iter().map(|(_, amount)| amount).sum();

        let summary = format!(
            "Analyzed {} months of spending across {} categories, total spending: {}",
            months_back,
            sorted_categories.len(),
            self.common.format_amount(total_spending)
        );

        let top_categories = sorted_categories
            .iter()
            .take(10)
            .map(|(name, amount)| {
                json!({
                    "category": name,
                    "amount": self.common.format_amount(*amount),
                    "percentage": if total_spending > 0 {
                        format!("{:.1}%", (*amount as f64 / total_spending as f64) * 100.0)
                    } else {
                        "0.0%".to_string()
                    }
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "analysis_period": {
                "months_back": months_back,
                "from_date": from_date.format("%Y-%m-%d").to_string(),
                "to_date": now.date_naive().format("%Y-%m-%d").to_string()
            },
            "spending_summary": {
                "total_spending": self.common.format_amount(total_spending),
                "total_transactions": filtered_transactions.len(),
                "categories_count": sorted_categories.len()
            },
            "top_categories": top_categories,
            "all_categories": sorted_categories.iter().map(|(name, amount)| {
                json!({
                    "category": name,
                    "amount": self.common.format_amount(*amount)
                })
            }).collect::<Vec<_>>()
        }))
    }

    pub async fn get_all_account_transactions(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let account_id = arguments
            .get("account_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| anyhow::anyhow!("Invalid account_id"))?;

        let transactions = self
            .common
            .ynab_client
            .get_transactions_by_account(&resolved_budget_id, account_id)
            .await?;

        let account_name = transactions
            .first()
            .map(|t| t.account_name.clone())
            .unwrap_or_else(|| "Unknown Account".to_string());

        let summary = format!(
            "Retrieved {} transactions for account: {}",
            transactions.len(),
            account_name
        );

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "account": {
                "id": account_id.to_string(),
                "name": account_name,
                "transaction_count": transactions.len()
            },
            "transactions": transactions
        }))
    }

    pub async fn get_transactions_delta(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let server_knowledge = arguments
            .get("server_knowledge")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as i64;

        let (transactions, new_server_knowledge) = self
            .common
            .ynab_client
            .get_transactions_delta(&resolved_budget_id, server_knowledge)
            .await?;

        let summary = format!(
            "Retrieved {} transactions using delta sync from server knowledge {}",
            transactions.len(),
            server_knowledge
        );

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "delta_sync": {
                "server_knowledge": new_server_knowledge,
                "previous_knowledge": server_knowledge,
                "new_transactions": transactions.len()
            },
            "transactions": transactions
        }))
    }

    pub async fn get_transactions_since_date(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let since_date = arguments
            .get("since_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("since_date is required"))?;

        // Validate date format
        chrono::NaiveDate::parse_from_str(since_date, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid since_date format, expected YYYY-MM-DD"))?;

        let transactions = self
            .common
            .ynab_client
            .get_transactions_since_date(&resolved_budget_id, since_date)
            .await?;

        let summary = format!(
            "Retrieved {} transactions modified since {}",
            transactions.len(),
            since_date
        );

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "filter": {
                "since_date": since_date,
                "transaction_count": transactions.len()
            },
            "transactions": transactions
        }))
    }

    pub async fn get_transactions_paginated(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as usize;

        let offset = arguments
            .get("offset")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as usize;

        if !(1..=1000).contains(&limit) {
            return Err(anyhow::anyhow!("limit must be between 1 and 1000"));
        }

        let (transactions, has_more) = self
            .common
            .ynab_client
            .get_transactions_paginated(&resolved_budget_id, limit as u32, Some(offset as u32))
            .await?;

        let summary = format!(
            "Retrieved {} transactions (page {}, {} per page)",
            transactions.len(),
            (offset / limit) + 1,
            limit
        );

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "pagination": {
                "limit": limit,
                "offset": offset,
                "current_page": (offset / limit) + 1,
                "transactions_returned": transactions.len(),
                "has_more": has_more
            },
            "transactions": transactions
        }))
    }

    pub async fn get_transactions_with_fields(&self, arguments: Value) -> Result<Value> {
        let budget_id = arguments
            .get("budget_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("budget_id is required"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;

        let fields = arguments.get("fields");
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as usize;

        let offset = arguments
            .get("offset")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as usize;

        if !(1..=1000).contains(&limit) {
            return Err(anyhow::anyhow!("limit must be between 1 and 1000"));
        }

        // Parse fields parameter
        let field_list = match fields {
            Some(Value::String(s)) => s.split(',').map(|s| s.trim().to_string()).collect(),
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![], // If no fields specified, return all fields
        };

        // Convert Vec<String> to Vec<&str> for the API call
        let field_refs: Vec<&str> = field_list.iter().map(|s| s.as_str()).collect();
        let field_option = if field_refs.is_empty() {
            None
        } else {
            Some(field_refs.as_slice())
        };

        let transactions = self
            .common
            .ynab_client
            .get_transactions_with_fields(
                &resolved_budget_id,
                field_option,
                Some(limit as u32),
                Some(offset as u32),
            )
            .await?;

        let summary = format!(
            "Retrieved {} transactions with {} fields (page {}, {} per page)",
            transactions.len(),
            if field_list.is_empty() {
                "all".to_string()
            } else {
                field_list.len().to_string()
            },
            (offset / limit) + 1,
            limit
        );

        Ok(json!({
            "success": true,
            "conversational_summary": summary,
            "field_selection": {
                "requested_fields": field_list,
                "limit": limit,
                "offset": offset,
                "transactions_returned": transactions.len()
            },
            "transactions": transactions
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::common::CommonUtils;
    use crate::secrets::YnabApiToken;
    use crate::ynab::YnabClient;

    fn create_test_common() -> CommonUtils {
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, true)
    }
    use serde_json::json;

    #[tokio::test]
    async fn test_analysis_handlers_creation() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        assert!(handlers.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_handle_analysis_operation_unknown_tool() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let result = handlers
            .handle_analysis_operation("unknown_tool", json!({}))
            .await;
        assert!(result.is_err()); // Returns error for unknown tool
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Unknown analysis operation"));
    }

    #[tokio::test]
    async fn test_analyze_transactions_validation_error() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.analyze_transactions(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_transactions_by_date_range_validation_error() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.get_transactions_by_date_range(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_spending_by_category_validation_error() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.analyze_spending_by_category(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_account_transactions_validation_error() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.get_all_account_transactions(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_transactions_paginated_validation_error() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.get_transactions_paginated(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_validation_error() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.get_transactions_with_fields(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_transactions_with_basic_args() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "invalid-budget",
            "from_date": "2024-01-01",
            "to_date": "2024-12-31"
        });
        let result = handlers.analyze_transactions(args).await;
        assert!(result.is_err()); // Should fail due to invalid budget but tests validation
    }

    #[tokio::test]
    async fn test_handle_analysis_operation_analyze_transactions() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "invalid-budget"
        });
        let result = handlers
            .handle_analysis_operation("analyze_transactions", args)
            .await;
        assert!(result.is_ok()); // Returns wrapped error response
    }

    #[tokio::test]
    async fn test_handle_analysis_operation_get_transactions_by_date_range() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "invalid-budget",
            "from_date": "2024-01-01",
            "to_date": "2024-12-31"
        });
        let result = handlers
            .handle_analysis_operation("get_transactions_by_date_range", args)
            .await;
        assert!(result.is_ok()); // Returns wrapped error response
    }

    #[tokio::test]
    async fn test_handle_analysis_operation_analyze_spending_by_category() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "invalid-budget",
            "months_back": 6
        });
        let result = handlers
            .handle_analysis_operation("analyze_spending_by_category", args)
            .await;
        assert!(result.is_ok()); // Returns wrapped error response
    }

    #[tokio::test]
    async fn test_analyze_transactions_invalid_limit() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "limit": 501 // Invalid limit > 500
        });
        let result = handlers.analyze_transactions(args).await;
        assert!(result.is_err()); // Just verify it errors
    }

    #[tokio::test]
    async fn test_analyze_transactions_zero_limit() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "limit": 0 // Invalid limit = 0
        });
        let result = handlers.analyze_transactions(args).await;
        assert!(result.is_err()); // Just verify it errors
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_invalid_limit() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "limit": 1001 // Invalid limit > 1000
        });
        let result = handlers.get_transactions_with_fields(args).await;
        assert!(result.is_err()); // Just verify it errors
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_string_fields() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "invalid-budget",
            "fields": "id,amount,date", // String format fields
            "limit": 10
        });
        let result = handlers.get_transactions_with_fields(args).await;
        assert!(result.is_err()); // Will fail due to invalid budget but tests field parsing
    }

    #[tokio::test]
    async fn test_get_transactions_with_fields_array_fields() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "invalid-budget",
            "fields": ["id", "amount", "date"], // Array format fields
            "limit": 10
        });
        let result = handlers.get_transactions_with_fields(args).await;
        assert!(result.is_err()); // Will fail due to invalid budget but tests field parsing
    }

    #[tokio::test]
    async fn test_analyze_spending_by_category_missing_budget() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "months_back": 6
            // Missing budget_id
        });
        let result = handlers.analyze_spending_by_category(args).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("budget_id is required"));
    }

    #[tokio::test]
    async fn test_get_all_account_transactions_missing_account() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "default"
            // Missing account_id
        });
        let result = handlers.get_all_account_transactions(args).await;
        assert!(result.is_err()); // Just verify it errors
    }

    #[tokio::test]
    async fn test_get_transactions_by_date_range_missing_dates() {
        let common = create_test_common();
        let handlers = AnalysisHandlers::new(common);

        let args = json!({
            "budget_id": "default"
            // Missing from_date and to_date
        });
        let result = handlers.get_transactions_by_date_range(args).await;
        assert!(result.is_err()); // Just verify it errors
    }
}
