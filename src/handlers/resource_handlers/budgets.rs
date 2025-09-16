use anyhow::Result;
use serde_json::{json, Value};

use crate::handlers::common::CommonUtils;

pub struct BudgetResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> BudgetResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_budgets(&self) -> Result<Value> {
        let budgets = self.common.ynab_client.get_budgets().await?;
        let data = json!({
            "type": "budgets",
            "data": budgets
        });
        Ok(self
            .common
            .create_mcp_response("ynab://budgets", "budgets.json", "YNAB Budgets", data))
    }

    pub async fn get_budget(&self, budget_id: &str) -> Result<Value> {
        // For individual budget requests, get basic budget info without all the nested data
        // to avoid JSON parsing issues with very large responses
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let budgets = self.common.ynab_client.get_budgets().await?;
        let budget = budgets
            .into_iter()
            .find(|b| b.id.to_string() == resolved_budget_id)
            .ok_or_else(|| anyhow::anyhow!("Budget not found: {}", resolved_budget_id))?;

        let data = json!({
            "type": "budget",
            "data": budget,
            "note": "Use specific endpoints like /accounts, /categories, /transactions for detailed data"
        });
        let uri = format!("ynab://budgets/{}", budget_id);
        Ok(self
            .common
            .create_mcp_response(&uri, "budget.json", "YNAB Budget Details", data))
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
    async fn test_budget_resource_handler_creation() {
        let common = create_test_common();
        let handler = BudgetResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_budget_handler_struct() {
        let common = create_test_common();
        let handler = BudgetResourceHandler::new(&common);

        // Test the handler structure
        assert_eq!(handler.common.read_only_mode, common.read_only_mode);
    }
}
