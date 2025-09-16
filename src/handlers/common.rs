use anyhow::Result;
use chrono::Datelike;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::handlers::validation::InputValidator;
use crate::ynab::YnabClient;

/// Common functionality shared across all tool handlers
#[derive(Clone)]
pub struct CommonUtils {
    pub ynab_client: YnabClient,
    pub read_only_mode: bool,
}

impl CommonUtils {
    pub fn new(ynab_client: YnabClient, read_only_mode: bool) -> Self {
        Self {
            ynab_client,
            read_only_mode,
        }
    }

    /// Resolve "default" budget ID to the first available budget
    pub async fn resolve_budget_id(&self, budget_id: &str) -> Result<String> {
        if budget_id != "default" {
            return Ok(budget_id.to_string());
        }

        // Get budgets and return the first one as the default
        let budgets = self.ynab_client.get_budgets().await?;
        let first_budget = budgets
            .first()
            .ok_or_else(|| anyhow::anyhow!("No budgets found in your YNAB account"))?;

        Ok(first_budget.id.to_string())
    }

    /// Resolve "current" month to the actual current month in YYYY-MM-DD format
    pub fn resolve_month(&self, month: &str) -> String {
        if month != "current" {
            return month.to_string();
        }

        // Get the current month in YYYY-MM-DD format (first day of month)
        let now = chrono::Utc::now();
        format!("{}-{:02}-01", now.year(), now.month())
    }

    /// Format amount from milliunits (YNAB's internal format) to human-readable currency
    /// Deprecated: Use MoneyUtils::format_milliunits directly
    pub fn format_amount(&self, milliunits: i64) -> String {
        crate::utils::MoneyUtils::format_milliunits(milliunits)
    }

    /// Wrap tool responses in proper MCP content format
    pub fn wrap_mcp_response(&self, message: &str, is_error: bool) -> Value {
        json!({
            "content": [
                {
                    "type": "text",
                    "text": message
                }
            ],
            "isError": is_error
        })
    }

    /// Wrap tool responses with both text summary and structured data
    pub fn wrap_mcp_response_with_data(
        &self,
        message: &str,
        data: &Value,
        is_error: bool,
    ) -> Value {
        // Create a structured response with conversational text and separate data
        json!({
            "content": [
                {
                    "type": "text",
                    "text": message
                },
                {
                    "type": "resource",
                    "resource": {
                        "uri": "ynab://response/data",
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(data).unwrap_or_default()
                    }
                }
            ],
            "isError": is_error
        })
    }

    /// Generate a conversational summary from response data
    pub fn generate_conversational_summary(&self, tool_name: &str, data: &Value) -> String {
        // Check if the response already has a conversational_summary field
        if let Some(summary) = data.get("conversational_summary").and_then(|v| v.as_str()) {
            return summary.to_string();
        }

        // Check if the response has a message field
        if let Some(message) = data.get("message").and_then(|v| v.as_str()) {
            return message.to_string();
        }

        // Generate tool-specific summaries based on the data
        match tool_name {
            "analyze_transactions" => {
                if let Some(summary) = data.get("summary") {
                    format!(
                        "Transaction analysis completed with {} items analyzed",
                        summary
                            .get("total_transactions")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                    )
                } else {
                    "Transaction analysis completed successfully".to_string()
                }
            }
            "get_transactions_by_date_range" => {
                if let Some(transactions) = data.get("transactions").and_then(|v| v.as_array()) {
                    format!(
                        "Retrieved {} transactions for the specified date range",
                        transactions.len()
                    )
                } else {
                    "Date range query completed successfully".to_string()
                }
            }
            "analyze_spending_by_category" => {
                if let Some(categories) = data.get("categories").and_then(|v| v.as_array()) {
                    format!(
                        "Spending analysis completed across {} categories",
                        categories.len()
                    )
                } else {
                    "Category spending analysis completed successfully".to_string()
                }
            }
            "health_check" => {
                if let Some(status) = data.get("status").and_then(|v| v.as_str()) {
                    if status == "healthy" {
                        "✅ YNAB API connection is healthy and responding normally".to_string()
                    } else {
                        "⚠️ YNAB API connection check failed - service may be unavailable"
                            .to_string()
                    }
                } else {
                    "Health check completed".to_string()
                }
            }
            _ => format!("{} completed successfully", tool_name.replace('_', " ")),
        }
    }

    /// Check if an operation is a write operation that should be blocked in read-only mode
    pub fn is_write_operation(&self, tool_name: &str) -> bool {
        matches!(
            tool_name,
            "create_transaction"
                | "create_transactions_bulk"
                | "update_transaction"
                | "create_transfer"
                | "approve_transaction"
                | "clear_transaction"
                | "assign_category_budget"
        )
    }

    /// Create an MCP resource response format
    pub fn create_mcp_response(&self, uri: &str, name: &str, title: &str, data: Value) -> Value {
        json!({
            "contents": [
                {
                    "uri": uri,
                    "name": name,
                    "title": title,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string())
                }
            ]
        })
    }

    /// Standard validation pipeline for transaction operations
    pub async fn validate_transaction_input(
        &self,
        args: &Value,
    ) -> Result<ValidatedTransactionInput> {
        let budget_id = InputValidator::validate_budget_id(args)?;
        let resolved_budget_id = self.resolve_budget_id(&budget_id).await?;
        let transaction_id = InputValidator::validate_uuid(args, "transaction_id")?;

        Ok(ValidatedTransactionInput {
            resolved_budget_id,
            transaction_id,
        })
    }

    /// Create standardized transaction response
    pub fn create_transaction_response(
        &self,
        action: &str,
        transaction: &crate::ynab::models::Transaction,
    ) -> Result<Value> {
        let account_name = &transaction.account_name;
        let payee_name = transaction.payee_name.as_deref().unwrap_or("Unknown Payee");
        let amount = self.format_amount(transaction.amount);
        let date = transaction.date.format("%Y-%m-%d");

        let conversational_summary = format!(
            "✅ Successfully {} transaction: {} for {} from {} on {}",
            action, payee_name, amount, account_name, date
        );

        Ok(json!({
            "success": true,
            "conversational_summary": conversational_summary,
            "message": format!("💰 {} transaction successfully", action),
            "transaction": transaction
        }))
    }
}

#[derive(Debug)]
pub struct ValidatedTransactionInput {
    pub resolved_budget_id: String,
    pub transaction_id: Uuid,
}
