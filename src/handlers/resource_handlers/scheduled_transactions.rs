use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::handlers::common::CommonUtils;

pub struct ScheduledTransactionResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> ScheduledTransactionResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_scheduled_transactions(&self, budget_id: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let scheduled_transactions = self
            .common
            .ynab_client
            .get_scheduled_transactions(&resolved_budget_id)
            .await?;
        let data = json!({
            "type": "scheduled_transactions",
            "data": scheduled_transactions
        });
        let uri = format!("ynab://budgets/{}/scheduled_transactions", budget_id);
        Ok(self.common.create_mcp_response(
            &uri,
            "scheduled_transactions.json",
            "YNAB Scheduled Transactions",
            data,
        ))
    }

    pub async fn get_scheduled_transaction(
        &self,
        budget_id: &str,
        scheduled_id: &str,
    ) -> Result<Value> {
        let scheduled_uuid = Uuid::parse_str(scheduled_id)
            .map_err(|_| anyhow::anyhow!("Invalid scheduled transaction ID format"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let scheduled_transaction = self
            .common
            .ynab_client
            .get_scheduled_transaction(&resolved_budget_id, scheduled_uuid)
            .await?;

        let data = json!({
            "type": "scheduled_transaction",
            "data": scheduled_transaction
        });
        let uri = format!(
            "ynab://budgets/{}/scheduled_transactions/{}",
            budget_id, scheduled_id
        );
        Ok(self.common.create_mcp_response(
            &uri,
            "scheduled_transaction.json",
            "YNAB Scheduled Transaction Details",
            data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::common::CommonUtils;
    use crate::secrets::YnabApiToken;
    use crate::ynab::YnabClient;
    use uuid::Uuid;

    fn create_test_common() -> CommonUtils {
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, true)
    }

    #[tokio::test]
    async fn test_scheduled_transaction_resource_handler_creation() {
        let common = create_test_common();
        let handler = ScheduledTransactionResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_get_scheduled_transactions_invalid_budget() {
        let common = create_test_common();
        let handler = ScheduledTransactionResourceHandler::new(&common);

        let result = handler.get_scheduled_transactions("invalid-budget").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_scheduled_transaction_invalid_uuid() {
        let common = create_test_common();
        let handler = ScheduledTransactionResourceHandler::new(&common);

        let result = handler
            .get_scheduled_transaction("test-budget", "invalid-uuid")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid scheduled transaction ID format"));
    }

    #[tokio::test]
    async fn test_get_scheduled_transaction_valid_uuid_invalid_budget() {
        let common = create_test_common();
        let handler = ScheduledTransactionResourceHandler::new(&common);

        let scheduled_id = Uuid::new_v4().to_string();
        let result = handler
            .get_scheduled_transaction("invalid-budget", &scheduled_id)
            .await;
        assert!(result.is_err());
    }
}
