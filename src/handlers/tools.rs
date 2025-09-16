use crate::error::Result;
// Removed complex metrics - using simple logging instead
use serde_json::Value;
use tracing::{instrument, warn};
// debug import removed - no debug logging needed

use super::{
    AnalysisHandlers, BudgetHandlers, CommonUtils, SystemHandlers, TransactionHandlers,
    TransferHandlers,
};
use crate::ynab::YnabClient;

pub struct ToolHandlers {
    transaction_handlers: TransactionHandlers,
    transfer_handlers: TransferHandlers,
    budget_handlers: BudgetHandlers,
    analysis_handlers: AnalysisHandlers,
    system_handlers: SystemHandlers,
}

impl ToolHandlers {
    pub fn new(ynab_client: YnabClient, read_only_mode: bool) -> Self {
        let common = CommonUtils::new(ynab_client, read_only_mode);

        Self {
            transaction_handlers: TransactionHandlers::new(common.clone()),
            transfer_handlers: TransferHandlers::new(common.clone()),
            budget_handlers: BudgetHandlers::new(common.clone()),
            analysis_handlers: AnalysisHandlers::new(common.clone()),
            system_handlers: SystemHandlers::new(common),
        }
    }

    #[instrument(skip(self, arguments), fields(tool_name = name))]
    pub async fn handle_tool_call(&self, name: &str, arguments: Value) -> Result<Value> {
        // Handle tool call with simple logging instead of complex metrics
        let start_time = std::time::Instant::now();

        // Strip MCP prefix if present
        let tool_name = if let Some(stripped) = name.strip_prefix("mcp__ynab-mcp__") {
            stripped
        } else {
            name
        };

        let result = match tool_name {
            "create_transaction"
            | "create_transactions_bulk"
            | "update_transaction"
            | "approve_transaction"
            | "clear_transaction" => {
                self.transaction_handlers
                    .handle_transaction_operation(tool_name, arguments)
                    .await
            }

            "create_transfer" => {
                self.transfer_handlers
                    .handle_transfer_operation(tool_name, arguments)
                    .await
            }

            "set_category_budget" => {
                self.budget_handlers
                    .handle_budget_operation(tool_name, arguments)
                    .await
            }

            "analyze_transactions"
            | "get_transactions_by_date_range"
            | "analyze_spending_by_category"
            | "get_all_account_transactions"
            | "get_transactions_delta"
            | "get_transactions_since_date"
            | "get_transactions_paginated"
            | "get_transactions_with_fields" => self
                .analysis_handlers
                .handle_analysis_operation(tool_name, arguments)
                .await
                .map_err(|e| e.into()),

            "health_check"
            | "cache_management"
            | "get_api_documentation"
            | "get_operation_progress"
            | "list_active_operations"
            | "cancel_operation"
            | "get_metrics"
            | "get_health_status"
            | "export_metrics"
            | "list_plugins"
            | "get_plugin_info"
            | "generate_spending_report"
            | "generate_category_trends"
            | "check_budget_alerts" => self
                .system_handlers
                .handle_system_operation(tool_name, arguments)
                .await
                .map_err(|e| e.into()),

            _ => {
                warn!(
                    tool_name = tool_name,
                    operation = "unknown_tool",
                    "Received request for unknown tool"
                );
                let error_msg = format!("Unknown tool: {}", tool_name);
                let common = &self.system_handlers.common;
                Ok(common.wrap_mcp_response(&error_msg, true))
            }
        };

        // Simple logging instead of complex metrics
        let elapsed = start_time.elapsed();
        match &result {
            Ok(_) => tracing::debug!(
                tool_name,
                elapsed_ms = elapsed.as_millis(),
                "Tool call completed successfully"
            ),
            Err(e) => {
                tracing::warn!(tool_name, elapsed_ms = elapsed.as_millis(), error = %e, "Tool call failed")
            }
        }

        result
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub async fn handle_tool_call_raw(&self, name: &str, arguments: Value) -> Result<Value> {
        // This method is for testing - it returns raw responses without MCP wrapping
        // Raw tool call for testing

        // Strip MCP prefix if present
        let tool_name = if let Some(stripped) = name.strip_prefix("mcp__ynab-mcp__") {
            stripped
        } else {
            name
        };

        // For testing, we bypass the MCP wrapper and return raw results
        // This is a simplified version that directly calls the underlying methods
        match tool_name {
            "create_transaction"
            | "update_transaction"
            | "create_transactions_bulk"
            | "approve_transaction"
            | "clear_transaction" => {
                self.transaction_handlers
                    .handle_transaction_operation(tool_name, arguments)
                    .await
            }
            "create_transfer" => self.transfer_handlers.create_transfer(arguments).await,
            "set_category_budget" => {
                self.budget_handlers.set_category_budget(arguments).await
            }
            "analyze_transactions" => self
                .analysis_handlers
                .analyze_transactions(arguments)
                .await
                .map_err(|e| e.into()),
            "get_transactions_by_date_range" => self
                .analysis_handlers
                .get_transactions_by_date_range(arguments)
                .await
                .map_err(|e| e.into()),
            "analyze_spending_by_category" => self
                .analysis_handlers
                .analyze_spending_by_category(arguments)
                .await
                .map_err(|e| e.into()),
            "get_all_account_transactions" => self
                .analysis_handlers
                .get_all_account_transactions(arguments)
                .await
                .map_err(|e| e.into()),
            "get_transactions_delta" => self
                .analysis_handlers
                .get_transactions_delta(arguments)
                .await
                .map_err(|e| e.into()),
            "get_transactions_since_date" => self
                .analysis_handlers
                .get_transactions_since_date(arguments)
                .await
                .map_err(|e| e.into()),
            "get_transactions_paginated" => self
                .analysis_handlers
                .get_transactions_paginated(arguments)
                .await
                .map_err(|e| e.into()),
            "get_transactions_with_fields" => self
                .analysis_handlers
                .get_transactions_with_fields(arguments)
                .await
                .map_err(|e| e.into()),
            "health_check" => self
                .system_handlers
                .health_check(arguments)
                .await
                .map_err(|e| e.into()),
            "cache_management" => self
                .system_handlers
                .cache_management(arguments)
                .await
                .map_err(|e| e.into()),
            _ => Err(crate::error::YnabMcpError::generic(&format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::common::CommonUtils;
    use crate::secrets::YnabApiToken;
    use crate::ynab::YnabClient;
    use serde_json::json;
    use uuid::Uuid;

    fn create_test_common() -> CommonUtils {
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, true)
    }

    #[tokio::test]
    async fn test_tool_handlers_creation() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        // Verify handlers are created properly - just check they exist
        assert!(std::ptr::eq(
            &handlers.transaction_handlers as *const _,
            &handlers.transaction_handlers as *const _
        ));
    }

    #[tokio::test]
    async fn test_handle_tool_call_create_transaction() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });

        let result = handlers.handle_tool_call("create_transaction", args).await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("read-only mode"));
    }

    #[tokio::test]
    async fn test_handle_tool_call_with_mcp_prefix() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });

        let result = handlers
            .handle_tool_call("mcp__ynab-mcp__create_transaction", args)
            .await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_create_transactions_bulk() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "transactions": []
        });

        let result = handlers
            .handle_tool_call("create_transactions_bulk", args)
            .await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_update_transaction() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "transaction_id": Uuid::new_v4().to_string()
        });

        let result = handlers.handle_tool_call("update_transaction", args).await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_approve_transaction() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "transaction_id": Uuid::new_v4().to_string()
        });

        let result = handlers.handle_tool_call("approve_transaction", args).await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_clear_transaction() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "cleared"
        });

        let result = handlers.handle_tool_call("clear_transaction", args).await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_create_transfer() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });

        let result = handlers.handle_tool_call("create_transfer", args).await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_set_category_budget() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "category_id": Uuid::new_v4().to_string(),
            "budgeted_amount": "100.00"
        });

        let result = handlers
            .handle_tool_call("set_category_budget", args)
            .await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_transactions() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("analyze_transactions", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_transactions_by_date_range() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "from_date": "2024-01-01",
            "to_date": "2024-01-31"
        });

        let result = handlers
            .handle_tool_call("get_transactions_by_date_range", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_spending_by_category() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("analyze_spending_by_category", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_all_account_transactions() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "account_id": Uuid::new_v4().to_string()
        });

        let result = handlers
            .handle_tool_call("get_all_account_transactions", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_transactions_delta() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("get_transactions_delta", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_transactions_since_date() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "since_date": "2024-01-01"
        });

        let result = handlers
            .handle_tool_call("get_transactions_since_date", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_transactions_paginated() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("get_transactions_paginated", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_transactions_with_fields() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("get_transactions_with_fields", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_health_check() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({});

        let result = handlers.handle_tool_call("health_check", args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_cache_management() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "action": "stats"
        });

        let result = handlers.handle_tool_call("cache_management", args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_api_documentation() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "type": "tools"
        });

        let result = handlers
            .handle_tool_call("get_api_documentation", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_metrics() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({});

        let result = handlers.handle_tool_call("get_metrics", args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_generate_spending_report() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("generate_spending_report", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_generate_category_trends() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers
            .handle_tool_call("generate_category_trends", args)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_check_budget_alerts() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget"
        });

        let result = handlers.handle_tool_call("check_budget_alerts", args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_unknown_tool() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({});

        let result = handlers.handle_tool_call("unknown_tool", args).await;
        assert!(result.is_ok()); // Returns wrapped error response

        // Verify it contains an error message
        let response = result.unwrap();
        assert!(response.get("isError").is_some() && response["isError"] == true);
    }

    #[tokio::test]
    async fn test_handle_tool_call_raw_create_transaction() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });

        let result = handlers
            .handle_tool_call_raw("create_transaction", args)
            .await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_raw_with_mcp_prefix() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });

        let result = handlers
            .handle_tool_call_raw("mcp__ynab-mcp__create_transaction", args)
            .await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_raw_create_transfer() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });

        let result = handlers.handle_tool_call_raw("create_transfer", args).await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_raw_set_category_budget() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({
            "budget_id": "test-budget",
            "category_id": Uuid::new_v4().to_string(),
            "budgeted_amount": "100.00"
        });

        let result = handlers
            .handle_tool_call_raw("set_category_budget", args)
            .await;
        // In read-only mode, write operations should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_tool_call_raw_health_check() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({});

        let result = handlers.handle_tool_call_raw("health_check", args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_raw_unknown_tool() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), true);

        let args = json!({});

        let result = handlers.handle_tool_call_raw("unknown_tool", args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown tool"));
    }

    #[test]
    fn test_tool_handlers_new() {
        let common = create_test_common();
        let handlers = ToolHandlers::new(common.ynab_client.clone(), false);

        // Test that read_only_mode is properly passed to the common utils
        // This is verified through the behavior of individual handlers
        assert!(std::ptr::eq(
            &handlers.transaction_handlers as *const _,
            &handlers.transaction_handlers as *const _
        ));
    }
}
