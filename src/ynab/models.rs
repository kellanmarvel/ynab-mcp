use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

// Custom serialization for amounts - converts milliunits to decimal currency amounts
pub fn serialize_amount<S>(amount: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let decimal_amount = Decimal::from(*amount) / Decimal::from(1000);
    serializer.serialize_str(&decimal_amount.to_string())
}

// Custom serialization for optional amounts
pub fn serialize_amount_option<S>(amount: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match amount {
        Some(amt) => {
            let decimal_amount = Decimal::from(*amt) / Decimal::from(1000);
            serializer.serialize_some(&decimal_amount.to_string())
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetsResponse {
    pub budgets: Vec<Budget>,
    pub default_budget: Option<Budget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetResponse {
    pub budget: Budget,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub account: Account,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesResponse {
    pub category_groups: Vec<CategoryGroup>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub category: Category,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeesResponse {
    pub payees: Vec<Payee>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeeResponse {
    pub payee: Payee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction: Transaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransactionsResponse {
    pub scheduled_transactions: Vec<ScheduledTransaction>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransactionResponse {
    pub scheduled_transaction: ScheduledTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthsResponse {
    pub months: Vec<Month>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthResponse {
    pub month: Month,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    pub name: String,
    pub last_modified_on: Option<DateTime<Utc>>, // Optional in API
    pub first_month: Option<NaiveDate>,          // Optional in API
    pub last_month: Option<NaiveDate>,           // Optional in API
    pub date_format: Option<DateFormat>,         // Optional and nullable in API
    pub currency_format: Option<CurrencyFormat>, // Optional and nullable in API
    pub accounts: Option<Vec<Account>>,
    pub payees: Option<Vec<Payee>>,
    pub payee_locations: Option<Vec<PayeeLocation>>,
    pub category_groups: Option<Vec<CategoryGroup>>,
    pub categories: Option<Vec<Category>>,
    pub months: Option<Vec<Month>>,
    pub transactions: Option<Vec<Transaction>>,
    pub subtransactions: Option<Vec<SubTransaction>>,
    pub scheduled_transactions: Option<Vec<ScheduledTransaction>>,
    pub scheduled_subtransactions: Option<Vec<ScheduledSubTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFormat {
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: u32,
    pub decimal_separator: String,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    #[serde(serialize_with = "serialize_amount")]
    pub balance: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub cleared_balance: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub uncleared_balance: i64, // milliunits
    pub transfer_payee_id: Option<Uuid>, // Can be null according to API
    pub direct_import_linked: Option<bool>, // Optional field
    pub direct_import_in_error: Option<bool>, // Optional field
    pub last_reconciled_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "serialize_amount_option")]
    pub debt_original_balance: Option<i64>,
    pub debt_interest_rates: Option<serde_json::Value>, // HashMap in TS
    pub debt_minimum_payments: Option<serde_json::Value>, // HashMap in TS
    pub debt_escrow_amounts: Option<serde_json::Value>, // HashMap in TS
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    #[serde(rename = "checking")]
    Checking,
    #[serde(rename = "savings")]
    Savings,
    #[serde(rename = "creditCard")]
    CreditCard,
    #[serde(rename = "cash")]
    Cash,
    #[serde(rename = "lineOfCredit")]
    LineOfCredit,
    #[serde(rename = "otherAsset")]
    OtherAsset,
    #[serde(rename = "otherLiability")]
    OtherLiability,
    #[serde(rename = "mortgage")]
    Mortgage,
    #[serde(rename = "autoLoan")]
    AutoLoan,
    #[serde(rename = "studentLoan")]
    StudentLoan,
    #[serde(rename = "personalLoan")]
    PersonalLoan,
    #[serde(rename = "medicalDebt")]
    MedicalDebt,
    #[serde(rename = "otherDebt")]
    OtherDebt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeeLocation {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub latitude: String,  // Required field according to API
    pub longitude: String, // Required field according to API
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroup {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub category_group_name: Option<String>,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    #[serde(serialize_with = "serialize_amount")]
    pub budgeted: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub activity: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub balance: i64, // milliunits
    pub goal_type: Option<String>, // CategoryGoalTypeEnum in TS
    pub goal_needs_whole_amount: Option<bool>,
    pub goal_day: Option<u32>,
    pub goal_cadence: Option<u32>,
    pub goal_cadence_frequency: Option<u32>,
    pub goal_creation_month: Option<String>, // String in API, not NaiveDate
    pub goal_target: Option<i64>,
    pub goal_target_month: Option<String>, // String in API, not NaiveDate
    pub goal_percentage_complete: Option<u32>,
    pub goal_months_to_budget: Option<u32>,
    pub goal_under_funded: Option<i64>,
    pub goal_overall_funded: Option<i64>,
    pub goal_overall_left: Option<i64>,
    pub goal_snoozed_at: Option<String>, // Missing field from API
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Month {
    pub month: NaiveDate,
    pub note: Option<String>,
    #[serde(serialize_with = "serialize_amount")]
    pub income: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub budgeted: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub activity: i64, // milliunits
    #[serde(serialize_with = "serialize_amount")]
    pub to_be_budgeted: i64, // milliunits
    pub age_of_money: Option<u32>,
    pub deleted: bool,
    pub categories: Option<Vec<Category>>, // Not always present in all endpoints
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub date: NaiveDate,
    #[serde(serialize_with = "serialize_amount")]
    pub amount: i64, // milliunits
    pub memo: Option<String>,
    pub cleared: ClearState,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub flag_name: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<String>,
    pub import_payee_name: Option<String>,
    pub import_payee_name_original: Option<String>,
    pub debt_transaction_type: Option<String>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Option<Vec<SubTransaction>>, // Present in TransactionDetail only
    // HybridTransaction specific fields
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    pub parent_transaction_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClearState {
    #[serde(rename = "cleared")]
    Cleared,
    #[serde(rename = "uncleared")]
    Uncleared,
    #[serde(rename = "reconciled")]
    Reconciled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTransaction {
    pub id: Uuid,
    pub transaction_id: Uuid,
    #[serde(serialize_with = "serialize_amount")]
    pub amount: i64, // milliunits
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub deleted: bool,
}

impl Transaction {
    /// Fix amount inconsistencies by checking the import_id for the correct amount
    pub fn fix_amount_if_needed(&mut self) {
        if let Some(import_id) = &self.import_id {
            // Parse import_id format: "YNAB:{amount}:{date}:{sequence}"
            let parts: Vec<&str> = import_id.split(':').collect();
            if parts.len() >= 2 && parts[0] == "YNAB" {
                if let Ok(expected_milliunits) = parts[1].parse::<i64>() {
                    // Check if the current amount seems wrong (too small compared to import_id)
                    let current_abs = self.amount.abs();
                    let expected_abs = expected_milliunits.abs();

                    // If the expected amount is significantly larger, assume API returned wrong format
                    if expected_abs >= 1000 && current_abs < 100 {
                        // Likely the API returned currency units instead of milliunits
                        // Use the import_id amount as the source of truth
                        self.amount = expected_milliunits;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransaction {
    pub id: Uuid,
    pub date_first: NaiveDate,
    pub date_next: NaiveDate, // Missing from original model
    pub frequency: String,    // ScheduledTransactionSummaryFrequencyEnum in TS
    #[serde(serialize_with = "serialize_amount")]
    pub amount: i64, // milliunits
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub flag_name: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String, // Not in API spec - likely added in detailed responses
    pub payee_name: Option<String>, // Not in API spec - likely added in detailed responses
    pub category_name: Option<String>, // Not in API spec - likely added in detailed responses
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledSubTransaction {
    pub id: Uuid,
    pub scheduled_transaction_id: Uuid,
    #[serde(serialize_with = "serialize_amount")]
    pub amount: i64, // milliunits
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub amount: i64, // milliunits (YNAB API expects raw integer milliunits)
    pub memo: Option<String>,
    pub cleared: Option<ClearState>,
    pub approved: Option<bool>,
    pub flag_color: Option<String>,
    pub flag_name: Option<String>,
    pub date: NaiveDate,
    pub import_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub transactions: Vec<TransactionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransaction {
    pub transaction: TransactionDetail,
}
