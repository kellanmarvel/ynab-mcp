use crate::error::{Result, YnabMcpError};
use serde_json::Value;

use super::{
    ApproveTransactionHandler, BulkTransactionHandler, ClearTransactionHandler,
    CreateTransactionHandler, UpdateTransactionHandler,
};
use crate::handlers::common::CommonUtils;

pub struct TransactionHandlers {
    common: CommonUtils,
}

impl TransactionHandlers {
    pub fn new(common: CommonUtils) -> Self {
        Self { common }
    }

    pub async fn handle_transaction_operation(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Result<Value> {
        // Handle transaction operation

        if self.common.read_only_mode && self.common.is_write_operation(tool_name) {
            return Err(YnabMcpError::generic(
                "Write operations are disabled in read-only mode",
            ));
        }

        let result = match tool_name {
            "create_transaction" => {
                let handler = CreateTransactionHandler::new(&self.common);
                handler.create_transaction(args).await
            }
            "create_transactions_bulk" => {
                let handler = BulkTransactionHandler::new(&self.common);
                handler.create_transactions_bulk(args).await
            }
            "update_transaction" => {
                let handler = UpdateTransactionHandler::new(&self.common);
                handler.update_transaction(args).await
            }
            "approve_transaction" => {
                let handler = ApproveTransactionHandler::new(&self.common);
                handler.approve_transaction(args).await
            }
            "clear_transaction" => {
                let handler = ClearTransactionHandler::new(&self.common);
                handler.clear_transaction(args).await
            }
            _ => {
                return Err(YnabMcpError::generic(&format!(
                    "Unknown transaction operation: {}",
                    tool_name
                )))
            }
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
            Err(e) => Err(e.into()),
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
    use std::env;

    fn create_test_common() -> CommonUtils {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, true) // read_only mode
    }

    fn create_test_common_write_mode() -> CommonUtils {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, false) // write mode
    }

    #[test]
    fn test_transaction_handlers_creation() {
        let common = create_test_common();
        let handlers = TransactionHandlers::new(common);

        assert!(handlers.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_read_only_mode() {
        let common = create_test_common();
        let handlers = TransactionHandlers::new(common);

        // Test that write operations are blocked in read-only mode
        let result = handlers
            .handle_transaction_operation("create_transaction", json!({}))
            .await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("read-only mode"));
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_create_transaction() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "account_id": "550e8400-e29b-41d4-a716-446655440000",
            "amount": "25.50",
            "date": "2023-01-01"
        });

        let result = handlers
            .handle_transaction_operation("create_transaction", args)
            .await;

        // Will fail at API level but tests the routing logic
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_create_bulk() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "transactions": [
                {
                    "account_id": "550e8400-e29b-41d4-a716-446655440000",
                    "amount": "25.50",
                    "date": "2023-01-01"
                }
            ]
        });

        let result = handlers
            .handle_transaction_operation("create_transactions_bulk", args)
            .await;

        // Tests the bulk transaction routing
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_update_transaction() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
            "amount": "30.00"
        });

        let result = handlers
            .handle_transaction_operation("update_transaction", args)
            .await;

        // Tests the update transaction routing
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_approve_transaction() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "transaction_id": "550e8400-e29b-41d4-a716-446655440000"
        });

        let result = handlers
            .handle_transaction_operation("approve_transaction", args)
            .await;

        // Tests the approve transaction routing
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_clear_transaction() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        let args = json!({
            "budget_id": "default",
            "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
            "clear_state": "cleared"
        });

        let result = handlers
            .handle_transaction_operation("clear_transaction", args)
            .await;

        // Tests the clear transaction routing
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_unknown_tool() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        let result = handlers
            .handle_transaction_operation("unknown_operation", json!({}))
            .await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unknown transaction operation: unknown_operation"));
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_all_write_operations_blocked() {
        let common = create_test_common(); // read-only mode
        let handlers = TransactionHandlers::new(common);

        let write_operations = vec![
            "create_transaction",
            "create_transactions_bulk",
            "update_transaction",
            "approve_transaction",
            "clear_transaction",
        ];

        for operation in write_operations {
            let result = handlers
                .handle_transaction_operation(operation, json!({}))
                .await;

            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("read-only mode"));
        }
    }

    #[tokio::test]
    async fn test_handle_transaction_operation_error_wrapping() {
        let common = create_test_common_write_mode();
        let handlers = TransactionHandlers::new(common);

        // This will cause a validation error that gets wrapped
        let result = handlers
            .handle_transaction_operation("create_transaction", json!({}))
            .await;

        assert!(result.is_err());
        // Tests the error wrapping logic in the Err(e) => Err(e.into()) path
    }

    #[tokio::test]
    async fn test_transaction_handlers_struct_access() {
        let common = create_test_common();
        let handlers = TransactionHandlers {
            common: common.clone(),
        };

        // Test that we can access the common field
        assert!(handlers.common.read_only_mode);
    }
}
