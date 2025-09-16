use anyhow::Result;
use serde_json::Value;

use crate::define_handler;
use crate::handlers::{
    common::CommonUtils, transaction_updater::TransactionUpdater, validation::InputValidator,
};

define_handler!(ApproveTransactionHandler);

impl<'a> ApproveTransactionHandler<'a> {
    pub async fn approve_transaction(&self, args: Value) -> Result<Value> {
        // Validate input
        InputValidator::validate_approve_transaction(&args)?;

        let validated_input = self.common.validate_transaction_input(&args).await?;

        let updater = TransactionUpdater::new(
            self.common,
            validated_input.resolved_budget_id,
            validated_input.transaction_id,
        )
        .await?;

        updater.approve().await
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
    async fn test_approve_transaction_handler_creation() {
        let common = create_test_common();
        let handler = ApproveTransactionHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_approve_transaction_validation_error() {
        let common = create_test_common();
        let handler = ApproveTransactionHandler::new(&common);

        let invalid_args = json!({});
        let result = handler.approve_transaction(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_approve_transaction_invalid_budget() {
        let common = create_test_common();
        let handler = ApproveTransactionHandler::new(&common);

        let invalid_args = json!({
            "budget_id": "invalid-budget",
            "transaction_id": Uuid::new_v4().to_string()
        });
        let result = handler.approve_transaction(invalid_args).await;
        assert!(result.is_err());
    }
}
