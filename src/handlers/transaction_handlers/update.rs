use anyhow::Result;
use serde_json::Value;

use crate::handlers::{
    common::CommonUtils, transaction_updater::TransactionUpdater, validation::InputValidator,
};

pub struct UpdateTransactionHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> UpdateTransactionHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn update_transaction(&self, args: Value) -> Result<Value> {
        // Validate all input fields comprehensively
        InputValidator::validate_update_transaction(&args)?;

        let validated_input = self.common.validate_transaction_input(&args).await?;

        let updater = TransactionUpdater::new(
            self.common,
            validated_input.resolved_budget_id,
            validated_input.transaction_id,
        )
        .await?;

        updater.update_fields(&args).await
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
    async fn test_update_transaction_handler_creation() {
        let common = create_test_common();
        let handler = UpdateTransactionHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_update_transaction_validation_error() {
        let common = create_test_common();
        let handler = UpdateTransactionHandler::new(&common);

        let invalid_args = json!({});
        let result = handler.update_transaction(invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_transaction_with_fields() {
        let common = create_test_common();
        let handler = UpdateTransactionHandler::new(&common);

        let args_with_fields = json!({
            "budget_id": "invalid-budget",
            "transaction_id": Uuid::new_v4().to_string(),
            "amount": "25.50",
            "memo": "Updated memo"
        });
        let result = handler.update_transaction(args_with_fields).await;
        assert!(result.is_err());
    }
}
