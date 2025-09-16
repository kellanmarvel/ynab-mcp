//! Common test utilities to reduce test setup duplication

#[cfg(test)]
use std::env;
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::handlers::common::CommonUtils;
#[cfg(test)]
use crate::secrets::YnabApiToken;
#[cfg(test)]
use crate::ynab::{models::*, YnabClient};

/// Creates a CommonUtils instance configured for testing
///
/// This function sets up a test environment with:
/// - Test API token enabled
/// - Read-only mode activated  
/// - Mock YNAB client pointing to test API
#[cfg(test)]
pub fn create_test_common() -> CommonUtils {
    env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
    let token = YnabApiToken::new("test-token".to_string());
    let client = YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
    CommonUtils::new(client, true)
}

/// Creates a realistic mock transaction for testing
///
/// Returns a Transaction with:
/// - Random UUID for ID
/// - Date set to 2024-01-15
/// - Amount of -$50.00 (outflow)
/// - Test payee and category data
/// - Cleared status
#[cfg(test)]
pub fn create_mock_transaction() -> Transaction {
    Transaction {
        id: Uuid::new_v4(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        amount: -50000, // $50.00 outflow
        memo: Some("Test transaction".to_string()),
        cleared: ClearState::Cleared,
        approved: true,
        flag_color: None,
        flag_name: None,
        account_id: Uuid::new_v4(),
        account_name: "Checking Account".to_string(),
        payee_id: Some(Uuid::new_v4()),
        payee_name: Some("Test Payee".to_string()),
        category_id: Some(Uuid::new_v4()),
        category_name: Some("Groceries".to_string()),
        transfer_account_id: None,
        transfer_transaction_id: None,
        matched_transaction_id: None,
        import_id: None,
        import_payee_name: None,
        import_payee_name_original: None,
        debt_transaction_type: None,
        deleted: false,
        subtransactions: None,
        transaction_type: None,
        parent_transaction_id: None,
    }
}

/// Creates a realistic mock account for testing
///
/// Returns an Account with:
/// - Random UUID for ID  
/// - "Test Checking" account name
/// - Checking account type
/// - $150.00 balance
/// - On-budget account
#[cfg(test)]
pub fn create_mock_account() -> Account {
    Account {
        id: Uuid::new_v4(),
        name: "Test Checking".to_string(),
        account_type: AccountType::Checking,
        on_budget: true,
        closed: false,
        note: None,
        balance: 150000, // $150.00
        cleared_balance: 150000,
        uncleared_balance: 0,
        transfer_payee_id: Some(Uuid::new_v4()),
        direct_import_linked: Some(false),
        direct_import_in_error: Some(false),
        last_reconciled_at: None,
        debt_original_balance: None,
        debt_interest_rates: None,
        debt_minimum_payments: None,
        debt_escrow_amounts: None,
        deleted: false,
    }
}

/// Sets up test environment variables
///
/// Configures:
/// - YNAB_BLOCK_TEST_TOKENS=false (allows test tokens)
/// - READ_ONLY_MODE=true (prevents API writes)
/// - YNAB_API_TOKEN=test-token (safe test token)
#[cfg(test)]
pub fn setup_test_env() {
    env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
    env::set_var("READ_ONLY_MODE", "true");
    env::set_var("YNAB_API_TOKEN", "test-token");
}

/// Cleans up test environment variables
///
/// Removes all test environment variables set by setup_test_env():
/// - YNAB_BLOCK_TEST_TOKENS
/// - READ_ONLY_MODE  
/// - YNAB_API_TOKEN
#[cfg(test)]
pub fn cleanup_test_env() {
    env::remove_var("YNAB_BLOCK_TEST_TOKENS");
    env::remove_var("READ_ONLY_MODE");
    env::remove_var("YNAB_API_TOKEN");
}
