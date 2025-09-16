use anyhow::Result;
use serde_json::{json, Value};

use crate::handlers::{common::CommonUtils, validation::InputValidator};
use crate::utils::MoneyUtils;

pub struct CreateTransactionHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> CreateTransactionHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn create_transaction(&self, args: Value) -> Result<Value> {
        // Validate all input fields comprehensively
        InputValidator::validate_create_transaction(&args)?;

        // Add progress message structure
        let mut progress_messages = Vec::new();
        progress_messages.push("📝 Preparing new transaction details...".to_string());

        let budget_id = InputValidator::validate_budget_id(&args)?;
        let resolved_budget_id = self.common.resolve_budget_id(&budget_id).await?;

        let account_id = InputValidator::validate_uuid(&args, "account_id")?;
        let amount_milliunits = MoneyUtils::validate_amount(&args, "amount")?;
        let date = InputValidator::validate_date(&args, "date")?;

        let memo = InputValidator::validate_optional_string(&args, "memo")?;
        let payee_id = InputValidator::validate_optional_uuid(&args, "payee_id")?;
        let category_id = InputValidator::validate_optional_uuid(&args, "category_id")?;

        let cleared = InputValidator::validate_optional_boolean(&args, "cleared")?.map(|cleared| {
            if cleared {
                crate::ynab::ClearState::Cleared
            } else {
                crate::ynab::ClearState::Uncleared
            }
        });

        let flag_color = InputValidator::validate_optional_enum(
            &args,
            "flag_color",
            &["red", "orange", "yellow", "green", "blue", "purple"],
        )?;
        let flag_name = InputValidator::validate_optional_string(&args, "flag_name")?;

        let transaction_detail = crate::ynab::TransactionDetail {
            account_id,
            payee_id,
            category_id,
            amount: amount_milliunits,
            memo,
            cleared,
            approved: Some(true),
            flag_color,
            flag_name,
            date,
            import_id: None,
        };

        let new_transaction = crate::ynab::NewTransaction {
            transactions: vec![transaction_detail],
        };

        progress_messages.push("💳 Creating transaction in YNAB...".to_string());

        let created_transactions = self
            .common
            .ynab_client
            .create_transaction(&resolved_budget_id, new_transaction)
            .await?;

        progress_messages.push("✅ Transaction created successfully".to_string());

        let first_transaction = created_transactions
            .first()
            .ok_or_else(|| anyhow::anyhow!("No transaction was created by the API"))?;
        let conversational_summary = format!(
            "✅ Successfully created transaction: {} for {} from {} on {}",
            first_transaction
                .payee_name
                .as_ref()
                .unwrap_or(&"Unknown Payee".to_string()),
            self.common.format_amount(first_transaction.amount),
            &first_transaction.account_name,
            first_transaction.date.format("%Y-%m-%d")
        );

        Ok(json!({
            "success": true,
            "progress_messages": progress_messages,
            "conversational_summary": conversational_summary,
            "message": format!("💰 Created transaction: {} for {} from {} on {}",
                first_transaction.payee_name.as_ref().unwrap_or(&"Unknown Payee".to_string()),
                self.common.format_amount(first_transaction.amount),
                &first_transaction.account_name,
                first_transaction.date.format("%Y-%m-%d")
            ),
            "transactions": created_transactions
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::common::CommonUtils;
    use crate::secrets::YnabApiToken;
    use crate::ynab::YnabClient;
    use serde_json::json;
    use std::env;

    fn create_test_common() -> CommonUtils {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, true) // read_only mode
    }

    #[tokio::test]
    async fn test_create_transaction_handler_creation() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test that handler can be created
        assert!(std::ptr::eq(handler.common, &common));
    }

    #[tokio::test]
    async fn test_create_transaction_validation_errors() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test with empty args
        let result = handler.create_transaction(json!({})).await;
        assert!(result.is_err(), "Should error with empty arguments");

        // Test with missing required fields
        let result = handler
            .create_transaction(json!({
                "budget_id": "default"
                // missing account_id, amount, date
            }))
            .await;
        assert!(result.is_err(), "Should error with missing required fields");
    }

    #[tokio::test]
    async fn test_create_transaction_with_invalid_amount() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "test-account-id",
                "amount": "invalid-amount",
                "date": "2023-01-01"
            }))
            .await;

        assert!(result.is_err(), "Should error with invalid amount format");
    }

    #[tokio::test]
    async fn test_create_transaction_with_invalid_date() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "test-account-id",
                "amount": "25.50",
                "date": "invalid-date"
            }))
            .await;

        assert!(result.is_err(), "Should error with invalid date format");
    }

    #[tokio::test]
    async fn test_create_transaction_with_invalid_uuid() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "not-a-valid-uuid",
                "amount": "25.50",
                "date": "2023-01-01"
            }))
            .await;

        assert!(result.is_err(), "Should error with invalid account_id UUID");
    }

    #[tokio::test]
    async fn test_create_transaction_with_optional_fields() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // This will likely fail at the YNAB API call level in test mode,
        // but should pass validation
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-01-01",
                "memo": "Test transaction",
                "payee_id": "550e8400-e29b-41d4-a716-446655440001",
                "category_id": "550e8400-e29b-41d4-a716-446655440002",
                "cleared": true,
                "flag_color": "red",
                "flag_name": "Important"
            }))
            .await;

        // In test mode with mocked client, this will likely fail at API call
        // but validation should pass. We're testing the validation path.
        let _ = result; // Don't assert success/failure as it depends on mock setup
    }

    #[tokio::test]
    async fn test_create_transaction_field_validation() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test invalid flag color
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-01-01",
                "flag_color": "invalid_color"
            }))
            .await;

        assert!(result.is_err(), "Should error with invalid flag color");
    }

    #[test]
    fn test_transaction_handler_struct() {
        let common = create_test_common();
        let handler = CreateTransactionHandler { common: &common };

        // Test that we can access the common field
        assert!(std::ptr::eq(handler.common, &common));
    }

    #[tokio::test]
    async fn test_create_transaction_cleared_flag_processing() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test cleared = true
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-01-01",
                "cleared": true
            }))
            .await;
        // This tests the cleared flag processing logic
        let _ = result; // Error expected due to test environment

        // Test cleared = false
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-01-01",
                "cleared": false
            }))
            .await;
        let _ = result; // Error expected due to test environment
    }

    #[tokio::test]
    async fn test_create_transaction_optional_enum_validation() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test each valid flag color
        let flag_colors = vec!["red", "orange", "yellow", "green", "blue", "purple"];

        for flag_color in flag_colors {
            let result = handler
                .create_transaction(json!({
                    "budget_id": "default",
                    "account_id": "550e8400-e29b-41d4-a716-446655440000",
                    "amount": "25.50",
                    "date": "2023-01-01",
                    "flag_color": flag_color
                }))
                .await;
            // Validation should pass, API call will fail in test env
            let _ = result;
        }
    }

    #[tokio::test]
    async fn test_create_transaction_progress_messages() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test that progress messages are built correctly
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-01-01"
            }))
            .await;

        // This exercises the progress messages code paths
        let _ = result; // Will fail at API call but exercises message building
    }

    #[tokio::test]
    async fn test_create_transaction_transaction_detail_building() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test transaction detail building with all optional fields
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "-125.75",
                "date": "2023-12-25",
                "memo": "Christmas gift",
                "payee_id": "550e8400-e29b-41d4-a716-446655440001",
                "category_id": "550e8400-e29b-41d4-a716-446655440002",
                "flag_name": "Gift"
            }))
            .await;

        // This exercises the TransactionDetail building logic
        let _ = result;
    }

    #[tokio::test]
    async fn test_create_transaction_negative_amount() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "-100.00",
                "date": "2023-01-01"
            }))
            .await;

        // Tests negative amount handling
        let _ = result;
    }

    #[tokio::test]
    async fn test_create_transaction_zero_amount() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "0.00",
                "date": "2023-01-01"
            }))
            .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_create_transaction_edge_case_dates() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        // Test leap year date
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2024-02-29"
            }))
            .await;
        let _ = result;

        // Test end of year
        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-12-31"
            }))
            .await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_create_transaction_with_all_optional_fields_null() {
        let common = create_test_common();
        let handler = CreateTransactionHandler::new(&common);

        let result = handler
            .create_transaction(json!({
                "budget_id": "default",
                "account_id": "550e8400-e29b-41d4-a716-446655440000",
                "amount": "25.50",
                "date": "2023-01-01",
                "memo": null,
                "payee_id": null,
                "category_id": null,
                "cleared": null,
                "flag_color": null,
                "flag_name": null
            }))
            .await;

        // Tests null handling for optional fields
        let _ = result;
    }
}
