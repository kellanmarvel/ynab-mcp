use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    AccountResourceHandler, BudgetResourceHandler, CategoryResourceHandler, MonthResourceHandler,
    PayeeResourceHandler, ScheduledTransactionResourceHandler, TransactionResourceHandler,
    UserResourceHandler,
};
use crate::handlers::common::CommonUtils;
use crate::ynab::YnabClient;

/// URI validation and parsing utilities for YNAB MCP resources
struct UriValidator;

impl UriValidator {
    /// Validate that a string is a properly formatted YNAB resource URI
    fn validate_ynab_uri(uri: &str) -> Result<()> {
        if !uri.starts_with("ynab://") {
            return Err(anyhow::anyhow!(
                "Resource URI must start with 'ynab://' scheme"
            ));
        }

        if uri.len() < 8 {
            // "ynab://" + at least 1 char
            return Err(anyhow::anyhow!("Resource URI is too short"));
        }

        // Check for invalid characters that could cause parsing issues
        if uri.matches("//").count() > 1 {
            return Err(anyhow::anyhow!(
                "Resource URI contains invalid double slashes"
            ));
        }

        Ok(())
    }

    /// Validate that a string is a valid UUID
    fn validate_uuid(id: &str, param_name: &str) -> Result<()> {
        match id {
            "default" => {
                // "default" is a special case for budget_id
                if param_name == "budget_id" {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "'default' is only valid for budget_id parameter"
                    ))
                }
            }
            _ => Uuid::parse_str(id)
                .map_err(|_| anyhow::anyhow!("{} must be a valid UUID, got: {}", param_name, id))
                .map(|_| ()),
        }
    }

    /// Parse and validate URI components
    fn parse_and_validate(uri: &str) -> Result<Vec<String>> {
        Self::validate_ynab_uri(uri)?;

        let path = &uri[7..]; // Remove "ynab://" prefix
        if path.is_empty() {
            return Err(anyhow::anyhow!("Resource URI path cannot be empty"));
        }

        let parts: Vec<String> = path.split('/').map(|s| s.to_string()).collect();

        // Convert to string references for pattern matching
        let parts_refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

        // Validate the structure based on expected patterns
        match parts_refs.as_slice() {
            [resource] if *resource == "budgets" || *resource == "user" => Ok(()),
            ["budgets", budget_id] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "accounts"] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "accounts", account_id] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(account_id, "account_id")
            }
            ["budgets", budget_id, "categories"] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "categories", category_id] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(category_id, "category_id")
            }
            ["budgets", budget_id, "payees"] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "payees", payee_id] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(payee_id, "payee_id")
            }
            ["budgets", budget_id, "transactions"] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "transactions", transaction_id] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(transaction_id, "transaction_id")
            }
            ["budgets", budget_id, "accounts", account_id, "transactions"] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(account_id, "account_id")
            }
            ["budgets", budget_id, "categories", category_id, "transactions"] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(category_id, "category_id")
            }
            ["budgets", budget_id, "scheduled_transactions"] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "scheduled_transactions", scheduled_id] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                Self::validate_uuid(scheduled_id, "scheduled_id")
            }
            ["budgets", budget_id, "months"] => {
                Self::validate_uuid(budget_id, "budget_id")
            }
            ["budgets", budget_id, "months", month] => {
                Self::validate_uuid(budget_id, "budget_id")?;
                // Month validation - should be YYYY-MM-DD format (first day of month)
                if !month.matches('-').count() == 2 || month.len() != 10 {
                    return Err(anyhow::anyhow!("Month must be in YYYY-MM-DD format, got: {}", month));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid resource URI pattern. Supported patterns: ynab://budgets, ynab://budgets/{{id}}, ynab://budgets/{{id}}/accounts, etc. Got: {}", 
                uri
            ))
        }?;

        Ok(parts)
    }
}

pub struct ResourceHandlers {
    common: CommonUtils,
}

impl ResourceHandlers {
    pub fn new(ynab_client: YnabClient) -> Self {
        Self {
            common: CommonUtils::new(ynab_client, false), // Resources don't need read-only mode
        }
    }

    pub async fn handle_resource_request(&self, uri: &str) -> Result<Value> {
        // Parse URI and query parameters first
        let (base_uri, _query_params) = if let Some(pos) = uri.find('?') {
            let base = &uri[..pos];
            let query = &uri[pos + 1..];
            let params: HashMap<String, String> = query
                .split('&')
                .filter_map(|param| {
                    let mut split = param.split('=');
                    match (split.next(), split.next()) {
                        (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                        _ => None,
                    }
                })
                .collect();
            (base, params)
        } else {
            (uri, HashMap::new())
        };

        // Validate and parse the URI using our validator
        let parts = UriValidator::parse_and_validate(base_uri)?;
        let parts_refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

        match parts_refs.as_slice() {
            ["budgets"] => BudgetResourceHandler::new(&self.common).get_budgets().await,
            ["budgets", budget_id] => {
                BudgetResourceHandler::new(&self.common)
                    .get_budget(budget_id)
                    .await
            }
            ["budgets", budget_id, "accounts"] => {
                AccountResourceHandler::new(&self.common)
                    .get_accounts(budget_id)
                    .await
            }
            ["budgets", budget_id, "accounts", account_id] => {
                AccountResourceHandler::new(&self.common)
                    .get_account(budget_id, account_id)
                    .await
            }
            ["budgets", budget_id, "categories"] => {
                CategoryResourceHandler::new(&self.common)
                    .get_categories(budget_id)
                    .await
            }
            ["budgets", budget_id, "categories", category_id] => {
                CategoryResourceHandler::new(&self.common)
                    .get_category(budget_id, category_id)
                    .await
            }
            ["budgets", budget_id, "payees"] => {
                PayeeResourceHandler::new(&self.common)
                    .get_payees(budget_id)
                    .await
            }
            ["budgets", budget_id, "payees", payee_id] => {
                PayeeResourceHandler::new(&self.common)
                    .get_payee(budget_id, payee_id)
                    .await
            }
            ["budgets", budget_id, "transactions"] => {
                TransactionResourceHandler::new(&self.common)
                    .get_transactions(budget_id)
                    .await
            }
            ["budgets", budget_id, "transactions", transaction_id] => {
                TransactionResourceHandler::new(&self.common)
                    .get_transaction(budget_id, transaction_id)
                    .await
            }
            ["budgets", budget_id, "accounts", account_id, "transactions"] => {
                AccountResourceHandler::new(&self.common)
                    .get_account_transactions(budget_id, account_id)
                    .await
            }
            ["budgets", budget_id, "categories", category_id, "transactions"] => {
                CategoryResourceHandler::new(&self.common)
                    .get_category_transactions(budget_id, category_id)
                    .await
            }
            ["budgets", budget_id, "scheduled_transactions"] => {
                ScheduledTransactionResourceHandler::new(&self.common)
                    .get_scheduled_transactions(budget_id)
                    .await
            }
            ["budgets", budget_id, "scheduled_transactions", scheduled_id] => {
                ScheduledTransactionResourceHandler::new(&self.common)
                    .get_scheduled_transaction(budget_id, scheduled_id)
                    .await
            }
            ["budgets", budget_id, "months"] => {
                MonthResourceHandler::new(&self.common)
                    .get_months(budget_id)
                    .await
            }
            ["budgets", budget_id, "months", month] => {
                MonthResourceHandler::new(&self.common)
                    .get_month(budget_id, month)
                    .await
            }
            ["user"] => UserResourceHandler::new(&self.common).get_user().await,
            _ => Err(anyhow::anyhow!("Unknown resource URI: {}", uri)),
        }
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

    #[test]
    fn test_uri_validator_validate_ynab_uri_valid() {
        let result = UriValidator::validate_ynab_uri("ynab://budgets");
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_validate_ynab_uri_invalid_scheme() {
        let result = UriValidator::validate_ynab_uri("http://budgets");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with 'ynab://'"));
    }

    #[test]
    fn test_uri_validator_validate_ynab_uri_too_short() {
        let result = UriValidator::validate_ynab_uri("ynab://");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_uri_validator_validate_ynab_uri_double_slashes() {
        let result = UriValidator::validate_ynab_uri("ynab://budgets//accounts");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("double slashes"));
    }

    #[test]
    fn test_uri_validator_validate_uuid_valid() {
        let uuid = Uuid::new_v4().to_string();
        let result = UriValidator::validate_uuid(&uuid, "budget_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_validate_uuid_default_budget_id() {
        let result = UriValidator::validate_uuid("default", "budget_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_validate_uuid_default_invalid_param() {
        let result = UriValidator::validate_uuid("default", "account_id");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("'default' is only valid for budget_id"));
    }

    #[test]
    fn test_uri_validator_validate_uuid_invalid() {
        let result = UriValidator::validate_uuid("invalid-uuid", "budget_id");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be a valid UUID"));
    }

    #[test]
    fn test_uri_validator_parse_and_validate_budgets() {
        let result = UriValidator::parse_and_validate("ynab://budgets");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts, vec!["budgets"]);
    }

    #[test]
    fn test_uri_validator_parse_and_validate_budget_by_id() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts, vec!["budgets", &budget_id]);
    }

    #[test]
    fn test_uri_validator_parse_and_validate_accounts() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/accounts", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_account_by_id() {
        let budget_id = Uuid::new_v4().to_string();
        let account_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/accounts/{}", budget_id, account_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_categories() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/categories", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_category_by_id() {
        let budget_id = Uuid::new_v4().to_string();
        let category_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/categories/{}", budget_id, category_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_payees() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/payees", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_payee_by_id() {
        let budget_id = Uuid::new_v4().to_string();
        let payee_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/payees/{}", budget_id, payee_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_transactions() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/transactions", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_transaction_by_id() {
        let budget_id = Uuid::new_v4().to_string();
        let transaction_id = Uuid::new_v4().to_string();
        let uri = format!(
            "ynab://budgets/{}/transactions/{}",
            budget_id, transaction_id
        );
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_account_transactions() {
        let budget_id = Uuid::new_v4().to_string();
        let account_id = Uuid::new_v4().to_string();
        let uri = format!(
            "ynab://budgets/{}/accounts/{}/transactions",
            budget_id, account_id
        );
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_category_transactions() {
        let budget_id = Uuid::new_v4().to_string();
        let category_id = Uuid::new_v4().to_string();
        let uri = format!(
            "ynab://budgets/{}/categories/{}/transactions",
            budget_id, category_id
        );
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_scheduled_transactions() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/scheduled_transactions", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_scheduled_transaction_by_id() {
        let budget_id = Uuid::new_v4().to_string();
        let scheduled_id = Uuid::new_v4().to_string();
        let uri = format!(
            "ynab://budgets/{}/scheduled_transactions/{}",
            budget_id, scheduled_id
        );
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_months() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/months", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_month_by_date() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/months/2024-01-01", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_validator_parse_and_validate_month_invalid_format() {
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/months/2024-01", budget_id);
        let result = UriValidator::parse_and_validate(&uri);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("YYYY-MM-DD format"));
    }

    #[test]
    fn test_uri_validator_parse_and_validate_user() {
        let result = UriValidator::parse_and_validate("ynab://user");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts, vec!["user"]);
    }

    #[test]
    fn test_uri_validator_parse_and_validate_empty_path() {
        let result = UriValidator::parse_and_validate("ynab://");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_uri_validator_parse_and_validate_invalid_pattern() {
        let result = UriValidator::parse_and_validate("ynab://invalid/pattern");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid resource URI pattern"));
    }

    #[tokio::test]
    async fn test_resource_handlers_creation() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        assert!(!handlers.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_budgets() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let result = handlers.handle_resource_request("ynab://budgets").await;
        assert!(result.is_err()); // Will fail due to mock client
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_budget_by_id() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}", budget_id);
        let result = handlers.handle_resource_request(&uri).await;
        assert!(result.is_err()); // Will fail due to mock client
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_accounts() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let budget_id = Uuid::new_v4().to_string();
        let uri = format!("ynab://budgets/{}/accounts", budget_id);
        let result = handlers.handle_resource_request(&uri).await;
        assert!(result.is_err()); // Will fail due to mock client
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_user() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let result = handlers.handle_resource_request("ynab://user").await;
        assert!(result.is_err()); // Will fail due to mock client
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_with_query_params() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let result = handlers
            .handle_resource_request("ynab://budgets?limit=10&offset=0")
            .await;
        assert!(result.is_err()); // Will fail due to mock client
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_invalid_uri() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let result = handlers.handle_resource_request("invalid://uri").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with 'ynab://'"));
    }

    #[tokio::test]
    async fn test_resource_handlers_handle_resource_request_unknown_pattern() {
        let common = create_test_common();
        let handlers = ResourceHandlers::new(common.ynab_client.clone());
        let result = handlers
            .handle_resource_request("ynab://unknown/pattern")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid resource URI pattern"));
    }
}
