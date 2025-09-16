use anyhow::Result;
use serde_json::Value;
use uuid::Uuid;

use crate::handlers::{common::CommonUtils, validation::InputValidator};
use crate::utils::MoneyUtils;
use crate::ynab::{
    models::{ClearState, TransactionDetail},
    UpdateTransaction,
};

pub struct TransactionUpdater<'a> {
    common: &'a CommonUtils,
    resolved_budget_id: String,
    transaction_id: Uuid,
    existing_transaction: Option<crate::ynab::models::Transaction>,
}

impl<'a> TransactionUpdater<'a> {
    pub async fn new(
        common: &'a CommonUtils,
        resolved_budget_id: String,
        transaction_id: Uuid,
    ) -> Result<Self> {
        Ok(Self {
            common,
            resolved_budget_id,
            transaction_id,
            existing_transaction: None,
        })
    }

    async fn get_existing_transaction(&mut self) -> Result<&crate::ynab::models::Transaction> {
        if self.existing_transaction.is_none() {
            let transaction = self
                .common
                .ynab_client
                .get_transaction(&self.resolved_budget_id, self.transaction_id)
                .await?;
            self.existing_transaction = Some(transaction);
        }
        // Safe unwrap: we just ensured the value exists above
        Ok(self
            .existing_transaction
            .as_ref()
            .expect("transaction must exist after initialization"))
    }

    pub async fn approve(mut self) -> Result<Value> {
        let existing = self.get_existing_transaction().await?;
        let transaction_detail = TransactionDetail::from_existing(existing).with_approved(true);

        self.update_and_respond(transaction_detail, "approved")
            .await
    }

    pub async fn clear(mut self, state: ClearState) -> Result<Value> {
        let existing = self.get_existing_transaction().await?;
        let transaction_detail = TransactionDetail::from_existing(existing).with_cleared(state);

        self.update_and_respond(transaction_detail, "cleared").await
    }

    pub async fn update_fields(mut self, args: &Value) -> Result<Value> {
        let existing = self.get_existing_transaction().await?;
        let mut transaction_detail = TransactionDetail::from_existing(existing);

        // Apply validated updates to transaction fields
        if let Some(account_id) = InputValidator::validate_optional_uuid(args, "account_id")? {
            transaction_detail.account_id = account_id;
        }

        if let Some(amount) = MoneyUtils::validate_optional_amount(args, "amount")? {
            transaction_detail.amount = amount;
        }

        if let Some(date) = InputValidator::validate_optional_date(args, "date")? {
            transaction_detail.date = date;
        }

        if let Some(memo) = InputValidator::validate_optional_string(args, "memo")? {
            transaction_detail.memo = Some(memo);
        }

        if let Some(payee_id) = InputValidator::validate_optional_uuid(args, "payee_id")? {
            transaction_detail.payee_id = Some(payee_id);
        }

        if let Some(category_id) = InputValidator::validate_optional_uuid(args, "category_id")? {
            transaction_detail.category_id = Some(category_id);
        }

        if let Some(cleared) = InputValidator::validate_optional_boolean(args, "cleared")? {
            transaction_detail.cleared = Some(if cleared {
                ClearState::Cleared
            } else {
                ClearState::Uncleared
            });
        }

        if let Some(flag_color) = InputValidator::validate_optional_enum(
            args,
            "flag_color",
            &["red", "orange", "yellow", "green", "blue", "purple"],
        )? {
            transaction_detail.flag_color = Some(flag_color);
        }

        if let Some(flag_name) = InputValidator::validate_optional_string(args, "flag_name")? {
            transaction_detail.flag_name = Some(flag_name);
        }

        self.update_and_respond(transaction_detail, "updated").await
    }

    async fn update_and_respond(
        &self,
        transaction_detail: TransactionDetail,
        action: &str,
    ) -> Result<Value> {
        let update_transaction = UpdateTransaction {
            transaction: transaction_detail,
        };

        let updated_transaction = self
            .common
            .ynab_client
            .update_transaction(
                &self.resolved_budget_id,
                self.transaction_id,
                update_transaction,
            )
            .await?;

        self.common
            .create_transaction_response(action, &updated_transaction)
    }
}

impl TransactionDetail {
    pub fn from_existing(transaction: &crate::ynab::models::Transaction) -> Self {
        Self {
            account_id: transaction.account_id,
            payee_id: transaction.payee_id,
            category_id: transaction.category_id,
            amount: transaction.amount,
            memo: transaction.memo.clone(),
            cleared: Some(transaction.cleared.clone()),
            approved: Some(transaction.approved),
            flag_color: transaction.flag_color.clone(),
            flag_name: transaction.flag_name.clone(),
            date: transaction.date,
            import_id: transaction.import_id.clone(),
        }
    }

    pub fn with_approved(mut self, approved: bool) -> Self {
        self.approved = Some(approved);
        self
    }

    pub fn with_cleared(mut self, cleared: ClearState) -> Self {
        self.cleared = Some(cleared);
        self
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

    fn create_mock_transaction() -> crate::ynab::models::Transaction {
        use crate::ynab::models::*;
        Transaction {
            id: uuid::Uuid::new_v4(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            amount: -50000, // $50.00 outflow
            memo: Some("Test transaction".to_string()),
            cleared: ClearState::Cleared,
            approved: true,
            flag_color: None,
            flag_name: None,
            account_id: uuid::Uuid::new_v4(),
            account_name: "Checking Account".to_string(),
            payee_id: Some(uuid::Uuid::new_v4()),
            payee_name: Some("Test Payee".to_string()),
            category_id: Some(uuid::Uuid::new_v4()),
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

    #[tokio::test]
    async fn test_transaction_updater_creation() {
        let common = create_test_common();
        let budget_id = "test-budget".to_string();
        let transaction_id = Uuid::new_v4();

        let updater = TransactionUpdater::new(&common, budget_id.clone(), transaction_id).await;
        assert!(updater.is_ok());

        let updater = updater.unwrap();
        assert_eq!(updater.resolved_budget_id, budget_id);
        assert_eq!(updater.transaction_id, transaction_id);
        assert!(updater.existing_transaction.is_none());
    }

    #[tokio::test]
    async fn test_transaction_detail_from_existing() {
        let mock_transaction = create_mock_transaction();
        let detail = TransactionDetail::from_existing(&mock_transaction);

        assert_eq!(detail.account_id, mock_transaction.account_id);
        assert_eq!(detail.amount, mock_transaction.amount);
        assert_eq!(detail.memo, mock_transaction.memo);
        assert_eq!(detail.date, mock_transaction.date);
    }

    #[tokio::test]
    async fn test_transaction_detail_with_approved() {
        let mock_transaction = create_mock_transaction();
        let detail = TransactionDetail::from_existing(&mock_transaction).with_approved(true);

        assert_eq!(detail.approved, Some(true));

        let detail = TransactionDetail::from_existing(&mock_transaction).with_approved(false);

        assert_eq!(detail.approved, Some(false));
    }

    #[tokio::test]
    async fn test_transaction_detail_with_cleared() {
        let mock_transaction = create_mock_transaction();
        let detail = TransactionDetail::from_existing(&mock_transaction)
            .with_cleared(ClearState::Reconciled);

        assert_eq!(detail.cleared, Some(ClearState::Reconciled));

        let detail2 =
            TransactionDetail::from_existing(&mock_transaction).with_cleared(ClearState::Uncleared);

        assert_eq!(detail2.cleared, Some(ClearState::Uncleared));
    }

    #[tokio::test]
    async fn test_transaction_detail_from_existing_all_fields() {
        let mock_transaction = create_mock_transaction();
        let detail = TransactionDetail::from_existing(&mock_transaction);

        assert_eq!(detail.account_id, mock_transaction.account_id);
        assert_eq!(detail.payee_id, mock_transaction.payee_id);
        assert_eq!(detail.category_id, mock_transaction.category_id);
        assert_eq!(detail.amount, mock_transaction.amount);
        assert_eq!(detail.memo, mock_transaction.memo);
        assert_eq!(detail.cleared, Some(mock_transaction.cleared.clone()));
        assert_eq!(detail.approved, Some(mock_transaction.approved));
        assert_eq!(detail.flag_color, mock_transaction.flag_color);
        assert_eq!(detail.flag_name, mock_transaction.flag_name);
        assert_eq!(detail.date, mock_transaction.date);
        assert_eq!(detail.import_id, mock_transaction.import_id);
    }

    #[tokio::test]
    async fn test_transaction_detail_chaining() {
        let mock_transaction = create_mock_transaction();
        let detail = TransactionDetail::from_existing(&mock_transaction)
            .with_approved(false)
            .with_cleared(ClearState::Reconciled);

        assert_eq!(detail.approved, Some(false));
        assert_eq!(detail.cleared, Some(ClearState::Reconciled));
        assert_eq!(detail.amount, mock_transaction.amount);
    }

    #[tokio::test]
    async fn test_update_fields_validation() {
        // test_common already available at module level

        let common = create_test_common();
        let budget_id = "test-budget".to_string();
        let transaction_id = Uuid::new_v4();

        let updater = TransactionUpdater::new(&common, budget_id, transaction_id).await;
        assert!(updater.is_ok());

        let updater = updater.unwrap();
        assert_eq!(updater.transaction_id, transaction_id);
        assert!(updater.existing_transaction.is_none());
    }

    #[tokio::test]
    async fn test_different_clear_states() {
        let mock_transaction = create_mock_transaction();

        let cleared_detail =
            TransactionDetail::from_existing(&mock_transaction).with_cleared(ClearState::Cleared);
        assert_eq!(cleared_detail.cleared, Some(ClearState::Cleared));

        let uncleared_detail =
            TransactionDetail::from_existing(&mock_transaction).with_cleared(ClearState::Uncleared);
        assert_eq!(uncleared_detail.cleared, Some(ClearState::Uncleared));

        let reconciled_detail = TransactionDetail::from_existing(&mock_transaction)
            .with_cleared(ClearState::Reconciled);
        assert_eq!(reconciled_detail.cleared, Some(ClearState::Reconciled));
    }

    #[tokio::test]
    async fn test_transaction_updater_fields() {
        // test_common already available at module level

        let common = create_test_common();
        let budget_id = "test-budget-123".to_string();
        let transaction_id = Uuid::new_v4();

        let updater = TransactionUpdater::new(&common, budget_id.clone(), transaction_id)
            .await
            .unwrap();

        assert_eq!(updater.resolved_budget_id, budget_id);
        assert_eq!(updater.transaction_id, transaction_id);
        assert!(updater.existing_transaction.is_none());
    }
}
