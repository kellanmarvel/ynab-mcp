use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::handlers::common::CommonUtils;

pub struct CategoryResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> CategoryResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_categories(&self, budget_id: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let category_groups = self
            .common
            .ynab_client
            .get_categories(&resolved_budget_id)
            .await?;
        let data = json!({
            "type": "categories",
            "data": category_groups
        });
        let uri = format!("ynab://budgets/{}/categories", budget_id);
        Ok(self
            .common
            .create_mcp_response(&uri, "categories.json", "YNAB Budget Categories", data))
    }

    pub async fn get_category(&self, budget_id: &str, category_id: &str) -> Result<Value> {
        let category_uuid = Uuid::parse_str(category_id)
            .map_err(|_| anyhow::anyhow!("Invalid category ID format"))?;

        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let category = self
            .common
            .ynab_client
            .get_category(&resolved_budget_id, category_uuid)
            .await?;

        let transactions_result = self
            .common
            .ynab_client
            .get_transactions_by_category(&resolved_budget_id, category_uuid)
            .await;

        let recent_transactions = match transactions_result {
            Ok(mut transactions) => {
                transactions.sort_by(|a, b| b.date.cmp(&a.date));
                let total = transactions.len();
                transactions.truncate(5);
                (transactions, total)
            }
            Err(_) => (Vec::new(), 0),
        };

        let data = json!({
            "type": "category",
            "data": category,
            "transactions": recent_transactions.0,
            "pagination": {
                "showing": recent_transactions.0.len(),
                "total_available": recent_transactions.1,
                "note": "Recent 5 transactions shown. Use category transactions endpoint for more."
            }
        });
        let uri = format!("ynab://budgets/{}/categories/{}", budget_id, category_id);
        Ok(self
            .common
            .create_mcp_response(&uri, "category.json", "YNAB Category Details", data))
    }

    pub async fn get_category_transactions(
        &self,
        budget_id: &str,
        category_id: &str,
    ) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let category_uuid = Uuid::parse_str(category_id)
            .map_err(|_| anyhow::anyhow!("Invalid category ID format"))?;

        let transactions = self
            .common
            .ynab_client
            .get_transactions_by_category(&resolved_budget_id, category_uuid)
            .await?;

        let mut sorted_transactions = transactions;
        sorted_transactions.sort_by(|a, b| b.date.cmp(&a.date));
        let total_available = sorted_transactions.len();
        sorted_transactions.truncate(30);

        let data = json!({
            "type": "category_transactions",
            "data": sorted_transactions,
            "category_id": category_id,
            "pagination": {
                "showing": sorted_transactions.len(),
                "total_available": total_available,
                "note": "Recent 30 transactions shown. Use tools for deeper analysis."
            },
            "summary": {
                "total_transactions": total_available
            }
        });
        let uri = format!(
            "ynab://budgets/{}/categories/{}/transactions",
            budget_id, category_id
        );
        Ok(self.common.create_mcp_response(
            &uri,
            "category_transactions.json",
            "YNAB Category Transactions",
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
    async fn test_category_resource_handler_creation() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_get_categories_invalid_budget() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        let result = handler.get_categories("invalid-budget").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_category_invalid_uuid() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        let result = handler.get_category("test-budget", "invalid-uuid").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid category ID format"));
    }

    #[tokio::test]
    async fn test_get_category_valid_uuid_invalid_budget() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        let category_id = Uuid::new_v4().to_string();
        let result = handler.get_category("invalid-budget", &category_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_category_transactions_invalid_uuid() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        let result = handler
            .get_category_transactions("test-budget", "invalid-uuid")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid category ID format"));
    }

    #[tokio::test]
    async fn test_get_category_transactions_valid_uuid_invalid_budget() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        let category_id = Uuid::new_v4().to_string();
        let result = handler
            .get_category_transactions("invalid-budget", &category_id)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_category_handler_struct() {
        let common = create_test_common();
        let handler = CategoryResourceHandler::new(&common);

        // Test the handler structure
        assert_eq!(handler.common.read_only_mode, common.read_only_mode);
    }
}
