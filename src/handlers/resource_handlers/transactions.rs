use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::handlers::common::CommonUtils;

pub struct TransactionResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> TransactionResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_transactions(&self, budget_id: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let transactions = self
            .common
            .ynab_client
            .get_transactions(&resolved_budget_id)
            .await?;

        let mut recent_transactions = transactions;
        recent_transactions.sort_by(|a, b| b.date.cmp(&a.date));
        let total_available = recent_transactions.len();
        recent_transactions.truncate(25);

        let pending_transactions = recent_transactions.iter().filter(|t| !t.approved).count();
        let uncleared_transactions = recent_transactions
            .iter()
            .filter(|t| matches!(t.cleared, crate::ynab::models::ClearState::Uncleared))
            .count();

        let data = json!({
            "type": "transactions",
            "data": recent_transactions,
            "pagination": {
                "showing": recent_transactions.len(),
                "total_available": total_available,
                "note": "Showing recent 25 transactions to prevent timeouts. Use specific tools for full analysis."
            },
            "summary": {
                "total_transactions": total_available,
                "showing_recent": recent_transactions.len(),
                "pending": pending_transactions,
                "uncleared": uncleared_transactions
            }
        });
        let uri = format!("ynab://budgets/{}/transactions", budget_id);
        Ok(self.common.create_mcp_response(
            &uri,
            "transactions.json",
            "YNAB Budget Transactions",
            data,
        ))
    }

    pub async fn get_transaction(&self, budget_id: &str, transaction_id: &str) -> Result<Value> {
        let resolved_budget_id = self.common.resolve_budget_id(budget_id).await?;
        let transaction_uuid = Uuid::parse_str(transaction_id)
            .map_err(|_| anyhow::anyhow!("Invalid transaction ID format"))?;

        let transaction = self
            .common
            .ynab_client
            .get_transaction(&resolved_budget_id, transaction_uuid)
            .await?;
        let data = json!({
            "type": "transaction",
            "data": transaction
        });
        let uri = format!(
            "ynab://budgets/{}/transactions/{}",
            budget_id, transaction_id
        );
        Ok(self.common.create_mcp_response(
            &uri,
            "transaction.json",
            "YNAB Transaction Details",
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
    async fn test_transaction_resource_handler_creation() {
        let common = create_test_common();
        let handler = TransactionResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_get_transactions_invalid_budget() {
        let common = create_test_common();
        let handler = TransactionResourceHandler::new(&common);

        let result = handler.get_transactions("invalid-budget").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_transaction_invalid_uuid() {
        let common = create_test_common();
        let handler = TransactionResourceHandler::new(&common);

        let result = handler.get_transaction("test-budget", "invalid-uuid").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid transaction ID format"));
    }

    #[tokio::test]
    async fn test_get_transaction_valid_uuid_invalid_budget() {
        let common = create_test_common();
        let handler = TransactionResourceHandler::new(&common);

        let transaction_id = Uuid::new_v4().to_string();
        let result = handler
            .get_transaction("invalid-budget", &transaction_id)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_handler_struct() {
        let common = create_test_common();
        let handler = TransactionResourceHandler::new(&common);

        // Test the handler structure
        assert_eq!(handler.common.read_only_mode, common.read_only_mode);
    }
}
