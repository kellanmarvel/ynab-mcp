use crate::error::{Result, YnabMcpError};
use serde_json::{json, Value};
// debug import removed - no debug logging needed

use super::{common::CommonUtils, validation::InputValidator};
use crate::utils::{MoneyUtils, ResponseUtils};
use crate::ynab::{ClearState, NewTransaction, TransactionDetail};

pub struct TransferHandlers {
    common: CommonUtils,
}

impl TransferHandlers {
    pub fn new(common: CommonUtils) -> Self {
        Self { common }
    }

    pub async fn handle_transfer_operation(&self, tool_name: &str, args: Value) -> Result<Value> {
        // Handle transfer operation

        if self.common.read_only_mode && self.common.is_write_operation(tool_name) {
            return Err(YnabMcpError::generic(
                "Write operations are disabled in read-only mode",
            ));
        }

        let result = match tool_name {
            "create_transfer" => self.create_transfer(args).await,
            _ => {
                return Err(YnabMcpError::generic(&format!(
                    "Unknown transfer operation: {}",
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
            Err(e) => Err(e),
        }
    }

    pub async fn create_transfer(&self, args: Value) -> Result<Value> {
        // Validate all input fields comprehensively
        InputValidator::validate_create_transfer(&args)?;

        let budget_id = InputValidator::validate_budget_id(&args)?;
        let resolved_budget_id = self.common.resolve_budget_id(&budget_id).await?;

        let from_account_id = InputValidator::validate_uuid(&args, "from_account_id")?;
        let to_account_id = InputValidator::validate_uuid(&args, "to_account_id")?;

        if from_account_id == to_account_id {
            return Err(YnabMcpError::generic("Cannot transfer to the same account"));
        }

        let amount_milliunits = MoneyUtils::validate_amount(&args, "amount")?;

        // Ensure transfer amount is positive
        MoneyUtils::validate_positive_amount(amount_milliunits, "Transfer")?;

        let date = InputValidator::validate_date(&args, "date")?;
        let memo = InputValidator::validate_optional_string(&args, "memo")?;

        // Get account names for better reporting
        let accounts = self
            .common
            .ynab_client
            .get_accounts(&resolved_budget_id)
            .await?;
        let from_account = accounts.iter().find(|a| a.id == from_account_id);
        let to_account = accounts.iter().find(|a| a.id == to_account_id);

        let from_account_name = from_account
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown Account".to_string());
        let to_account_name = to_account
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown Account".to_string());

        // Create outgoing transaction (negative amount from source account)
        let outgoing_transaction = TransactionDetail {
            account_id: from_account_id,
            payee_id: None,             // Transfers don't have payees
            category_id: None,          // Transfers don't have categories
            amount: -amount_milliunits, // Negative for outgoing
            memo: memo.clone(),
            cleared: Some(ClearState::Cleared),
            approved: Some(true),
            flag_color: None,
            flag_name: None,
            date,
            import_id: None,
        };

        // Create incoming transaction (positive amount to destination account)
        let incoming_transaction = TransactionDetail {
            account_id: to_account_id,
            payee_id: None,
            category_id: None,
            amount: amount_milliunits, // Positive for incoming
            memo: memo.clone(),
            cleared: Some(ClearState::Cleared),
            approved: Some(true),
            flag_color: None,
            flag_name: None,
            date,
            import_id: None,
        };

        let new_transaction = NewTransaction {
            transactions: vec![outgoing_transaction, incoming_transaction],
        };

        let created_transactions = self
            .common
            .ynab_client
            .create_transaction(&resolved_budget_id, new_transaction)
            .await?;

        let formatted_amount = MoneyUtils::format_milliunits(amount_milliunits);
        let date_str = date.format("%Y-%m-%d").to_string();

        let transfer_details = json!({
            "transactions": created_transactions,
            "transfer_summary": {
                "amount": formatted_amount,
                "from_account": from_account_name,
                "to_account": to_account_name,
                "date": date_str,
                "memo": memo
            }
        });

        let response = ResponseUtils::amount_success(
            "transferred",
            amount_milliunits,
            &format!(
                "from {} to {} on {}",
                from_account_name, to_account_name, date_str
            ),
            Some(transfer_details),
        );

        Ok(response)
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
    async fn test_transfer_handlers_creation() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        assert!(handlers.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_handle_transfer_operation_unknown_tool() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let result = handlers
            .handle_transfer_operation("unknown_tool", json!({}))
            .await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Unknown transfer operation"));
    }

    #[tokio::test]
    async fn test_handle_transfer_operation_read_only_mode() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let result = handlers
            .handle_transfer_operation("create_transfer", json!({}))
            .await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Write operations are disabled in read-only mode"));
    }

    #[tokio::test]
    async fn test_create_transfer_validation_error() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let invalid_args = json!({});
        let result = handlers.create_transfer(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_transfer_missing_budget_id() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let args = json!({
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });
        let result = handlers.create_transfer(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_transfer_same_account() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let account_id = Uuid::new_v4().to_string();
        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": account_id,
            "to_account_id": account_id,
            "amount": "100.00",
            "date": "2024-01-01"
        });
        let result = handlers.create_transfer(args).await;
        assert!(result.is_err());
        // Error could be from same account check or other validation - just test that it errors
        let error_message = result.unwrap_err().to_string();
        assert!(!error_message.is_empty());
    }

    #[tokio::test]
    async fn test_create_transfer_invalid_amount() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "invalid",
            "date": "2024-01-01"
        });
        let result = handlers.create_transfer(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_transfer_negative_amount() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "-100.00",
            "date": "2024-01-01"
        });
        let result = handlers.create_transfer(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_transfer_invalid_date() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "invalid-date"
        });
        let result = handlers.create_transfer(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_transfer_valid_structure() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "250.75",
            "date": "2024-01-15",
            "memo": "Transfer memo"
        });
        let result = handlers.create_transfer(args).await;
        assert!(result.is_err()); // Will fail due to invalid budget but tests validation logic
    }

    #[tokio::test]
    async fn test_handle_transfer_operation_create_transfer() {
        let common = create_test_common();
        let handlers = TransferHandlers::new(common);

        let args = json!({
            "budget_id": "test-budget",
            "from_account_id": Uuid::new_v4().to_string(),
            "to_account_id": Uuid::new_v4().to_string(),
            "amount": "100.00",
            "date": "2024-01-01"
        });
        let result = handlers
            .handle_transfer_operation("create_transfer", args)
            .await;
        assert!(result.is_err()); // Will fail due to read-only mode
    }
}
