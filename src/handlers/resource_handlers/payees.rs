use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::handlers::common::CommonUtils;

pub struct PayeeResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> PayeeResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_payees(&self, budget_id: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let payees = self
            .common
            .ynab_client
            .get_payees(&resolved_budget_id)
            .await?;
        let data = json!({
            "type": "payees",
            "data": payees
        });
        let uri = format!("ynab://budgets/{}/payees", budget_id);
        Ok(self
            .common
            .create_mcp_response(&uri, "payees.json", "YNAB Budget Payees", data))
    }

    pub async fn get_payee(&self, budget_id: &str, payee_id: &str) -> Result<Value> {
        let payee_uuid =
            Uuid::parse_str(payee_id).map_err(|_| anyhow::anyhow!("Invalid payee ID format"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let payee = self
            .common
            .ynab_client
            .get_payee(&resolved_budget_id, payee_uuid)
            .await?;
        let data = json!({
            "type": "payee",
            "data": payee
        });
        let uri = format!("ynab://budgets/{}/payees/{}", budget_id, payee_id);
        Ok(self
            .common
            .create_mcp_response(&uri, "payee.json", "YNAB Payee Details", data))
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
    async fn test_payee_resource_handler_creation() {
        let common = create_test_common();
        let handler = PayeeResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_get_payees_invalid_budget() {
        let common = create_test_common();
        let handler = PayeeResourceHandler::new(&common);

        let result = handler.get_payees("invalid-budget").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_payee_invalid_uuid() {
        let common = create_test_common();
        let handler = PayeeResourceHandler::new(&common);

        let result = handler.get_payee("test-budget", "invalid-uuid").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid payee ID format"));
    }

    #[tokio::test]
    async fn test_get_payee_valid_uuid_invalid_budget() {
        let common = create_test_common();
        let handler = PayeeResourceHandler::new(&common);

        let payee_id = Uuid::new_v4().to_string();
        let result = handler.get_payee("invalid-budget", &payee_id).await;
        assert!(result.is_err());
    }
}
