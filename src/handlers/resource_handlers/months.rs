use anyhow::Result;
use serde_json::{json, Value};

use crate::handlers::common::CommonUtils;

pub struct MonthResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> MonthResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_months(&self, budget_id: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let months = self
            .common
            .ynab_client
            .get_months(&resolved_budget_id)
            .await?;
        let data = json!({
            "type": "months",
            "data": months
        });
        let uri = format!("ynab://budgets/{}/months", budget_id);
        Ok(self
            .common
            .create_mcp_response(&uri, "months.json", "YNAB Budget Months", data))
    }

    pub async fn get_month(&self, budget_id: &str, month: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let month_data = self
            .common
            .ynab_client
            .get_month(&resolved_budget_id, month)
            .await?;
        let data = json!({
            "type": "month",
            "data": month_data
        });
        let uri = format!("ynab://budgets/{}/months/{}", budget_id, month);
        Ok(self
            .common
            .create_mcp_response(&uri, "month.json", "YNAB Budget Month Details", data))
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

    #[tokio::test]
    async fn test_month_resource_handler_creation() {
        let common = create_test_common();
        let handler = MonthResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_get_months_invalid_budget() {
        let common = create_test_common();
        let handler = MonthResourceHandler::new(&common);

        let result = handler.get_months("invalid-budget").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_month_invalid_budget() {
        let common = create_test_common();
        let handler = MonthResourceHandler::new(&common);

        let result = handler.get_month("invalid-budget", "2024-01").await;
        assert!(result.is_err());
    }
}
