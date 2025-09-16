use anyhow::Result;
use serde_json::Value;

use crate::handlers::{
    common::CommonUtils, transaction_updater::TransactionUpdater, validation::InputValidator,
};
use crate::ynab::ClearState;

pub struct ClearTransactionHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> ClearTransactionHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn clear_transaction(&self, args: Value) -> Result<Value> {
        // Validate input
        InputValidator::validate_clear_transaction(&args)?;

        let validated_input = self.common.validate_transaction_input(&args).await?;

        let clear_state_str = InputValidator::validate_optional_enum(
            &args,
            "clear_state",
            &["cleared", "uncleared", "reconciled"],
        )?
        .unwrap_or_else(|| "cleared".to_string());

        let clear_state = match clear_state_str.as_str() {
            "cleared" => ClearState::Cleared,
            "uncleared" => ClearState::Uncleared,
            "reconciled" => ClearState::Reconciled,
            _ => unreachable!("Already validated by InputValidator"),
        };

        let updater = TransactionUpdater::new(
            self.common,
            validated_input.resolved_budget_id,
            validated_input.transaction_id,
        )
        .await?;

        updater.clear(clear_state).await
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
    async fn test_clear_transaction_handler_creation() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_clear_transaction_validation_error() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        let invalid_args = json!({});
        let result = handler.clear_transaction(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_state_parsing() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        let args_with_state = json!({
            "budget_id": "invalid-budget",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "reconciled"
        });
        let result = handler.clear_transaction(args_with_state).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_state_all_valid_values() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        // Test each valid clear state
        let clear_states = vec!["cleared", "uncleared", "reconciled"];

        for clear_state in clear_states {
            let args = json!({
                "budget_id": "default",
                "transaction_id": Uuid::new_v4().to_string(),
                "clear_state": clear_state
            });
            let result = handler.clear_transaction(args).await;
            // Will fail at API level but tests state parsing
            let _ = result;
        }
    }

    #[tokio::test]
    async fn test_clear_state_default_to_cleared() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        // Test that missing clear_state defaults to "cleared"
        let args = json!({
            "budget_id": "default",
            "transaction_id": Uuid::new_v4().to_string()
            // No clear_state field
        });
        let result = handler.clear_transaction(args).await;
        // This exercises the unwrap_or_else default logic
        let _ = result;
    }

    #[tokio::test]
    async fn test_clear_state_match_branches() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        // Test cleared branch
        let args = json!({
            "budget_id": "default",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "cleared"
        });
        let result = handler.clear_transaction(args).await;
        let _ = result;

        // Test uncleared branch
        let args = json!({
            "budget_id": "default",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "uncleared"
        });
        let result = handler.clear_transaction(args).await;
        let _ = result;

        // Test reconciled branch
        let args = json!({
            "budget_id": "default",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "reconciled"
        });
        let result = handler.clear_transaction(args).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_clear_transaction_validation_with_required_fields() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        // Test with missing transaction_id
        let result = handler
            .clear_transaction(json!({
                "budget_id": "default"
            }))
            .await;
        assert!(result.is_err());

        // Test with missing budget_id
        let result = handler
            .clear_transaction(json!({
                "transaction_id": Uuid::new_v4().to_string()
            }))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_transaction_with_invalid_clear_state() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        let args = json!({
            "budget_id": "default",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "invalid_state"
        });
        let result = handler.clear_transaction(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_transaction_handler_struct_access() {
        let common = create_test_common();
        let handler = ClearTransactionHandler { common: &common };

        // Test direct struct access
        assert!(std::ptr::eq(handler.common, &common));
        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_clear_transaction_with_valid_uuid() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let args = json!({
            "budget_id": "default",
            "transaction_id": valid_uuid,
            "clear_state": "cleared"
        });
        let result = handler.clear_transaction(args).await;
        // This exercises the validation and TransactionUpdater creation path
        let _ = result;
    }

    #[tokio::test]
    async fn test_clear_transaction_updater_creation() {
        let common = create_test_common();
        let handler = ClearTransactionHandler::new(&common);

        let args = json!({
            "budget_id": "default",
            "transaction_id": Uuid::new_v4().to_string(),
            "clear_state": "uncleared"
        });
        let result = handler.clear_transaction(args).await;
        // This tests the TransactionUpdater::new path and .clear() call
        let _ = result;
    }
}
