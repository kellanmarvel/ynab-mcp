use crate::error::{Result, YnabMcpError};
use serde_json::{json, Value};
// debug import removed - no debug logging needed

pub struct PromptHandlers;

impl PromptHandlers {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_prompt_request(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<Value> {
        // Handle prompt request

        match name {
            "quick_expense" => self.quick_expense_prompt(arguments).await,
            "budget_review" => self.budget_review_prompt(arguments).await,
            "spending_analysis" => self.spending_analysis_prompt(arguments).await,
            "account_reconciliation" => self.account_reconciliation_prompt(arguments).await,
            "transaction_categorization" => self.transaction_categorization_prompt(arguments).await,
            _ => Err(YnabMcpError::validation(
                &format!("Unknown prompt: '{}'", name),
                "prompt_name",
                "one of: quick_expense, budget_review, spending_analysis, account_reconciliation, transaction_categorization",
                "Use the prompts/list endpoint to see all available prompts and their descriptions"
            )),
        }
    }

    async fn quick_expense_prompt(&self, arguments: Option<Value>) -> Result<Value> {
        let budget_id = arguments
            .as_ref()
            .and_then(|args| args.get("budget_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "I want to quickly add an expense to my YNAB budget ({}). \n\n\
                            Please help me create a transaction with the following information:\n\
                            - Amount: [Enter the expense amount]\n\
                            - Account: [Which account was used?]\n\
                            - Payee: [Where was the money spent?]\n\
                            - Category: [What budget category should this be assigned to?]\n\
                            - Date: [When did this expense occur? Default to today]\n\
                            - Memo: [Optional note about the expense]\n\n\
                            Use the create_transaction tool to add this to my budget once you have all the necessary information.",
                            budget_id
                        )
                    }
                }
            ]
        }))
    }

    async fn budget_review_prompt(&self, arguments: Option<Value>) -> Result<Value> {
        let budget_id = arguments
            .as_ref()
            .and_then(|args| args.get("budget_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let month = arguments
            .as_ref()
            .and_then(|args| args.get("month"))
            .and_then(|v| v.as_str())
            .unwrap_or("current");

        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Please help me review my YNAB budget for {} (budget: {}).\n\n\
                            I'd like to see:\n\
                            1. Overall budget performance - how much was budgeted vs. spent\n\
                            2. Categories that are over budget or have unusual spending\n\
                            3. Account balances and any discrepancies\n\
                            4. Recent transactions that might need attention (unapproved, uncategorized)\n\
                            5. Any recommendations for budget adjustments\n\n\
                            Start by fetching the month data and recent transactions, then provide \
                            a comprehensive analysis and actionable insights.",
                            month, budget_id
                        )
                    }
                }
            ]
        }))
    }

    async fn spending_analysis_prompt(&self, arguments: Option<Value>) -> Result<Value> {
        let budget_id = arguments
            .as_ref()
            .and_then(|args| args.get("budget_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let category_id = arguments
            .as_ref()
            .and_then(|args| args.get("category_id"))
            .and_then(|v| v.as_str());

        let timeframe = arguments
            .as_ref()
            .and_then(|args| args.get("timeframe"))
            .and_then(|v| v.as_str())
            .unwrap_or("current month");

        let analysis_focus = if let Some(cat_id) = category_id {
            format!("Focus the analysis on category ID: {}", cat_id)
        } else {
            "Provide an analysis across all spending categories".to_string()
        };

        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "I want to analyze my spending patterns in YNAB for {} (budget: {}).\n\n\
                            {}.\n\n\
                            Please help me understand:\n\
                            1. Spending trends and patterns\n\
                            2. Categories where I'm consistently over or under budget\n\
                            3. Largest expenses and frequent small expenses\n\
                            4. Comparison to previous periods if possible\n\
                            5. Recommendations for better budget allocation\n\n\
                            Start by gathering the relevant transaction and category data, \
                            then provide insights and actionable recommendations.",
                            timeframe, budget_id, analysis_focus
                        )
                    }
                }
            ]
        }))
    }

    async fn account_reconciliation_prompt(&self, arguments: Option<Value>) -> Result<Value> {
        let budget_id = arguments
            .as_ref()
            .and_then(|args| args.get("budget_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let account_id = arguments
            .as_ref()
            .and_then(|args| args.get("account_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| YnabMcpError::validation(
                "account_id parameter is required for account reconciliation",
                "account_id",
                "valid YNAB account UUID",
                "Get available account IDs from resources/list or by using the list budgets functionality"
            ))?;

        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "I need help reconciling my account {} in budget {}.\n\n\
                            Please help me:\n\
                            1. Review the current account balance and cleared balance\n\
                            2. Show me uncleared transactions that might need attention\n\
                            3. Identify any pending or unapproved transactions\n\
                            4. Guide me through clearing transactions that have appeared on my bank statement\n\
                            5. Check for any discrepancies between YNAB and my actual account balance\n\n\
                            Start by fetching the account details and recent transactions, \
                            then guide me through the reconciliation process step by step.",
                            account_id, budget_id
                        )
                    }
                }
            ]
        }))
    }

    async fn transaction_categorization_prompt(&self, arguments: Option<Value>) -> Result<Value> {
        let budget_id = arguments
            .as_ref()
            .and_then(|args| args.get("budget_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "I need help categorizing transactions in my YNAB budget ({}).\n\n\
                            Please help me:\n\
                            1. Find transactions that are uncategorized or need category review\n\
                            2. Suggest appropriate categories based on payee names and amounts\n\
                            3. Identify any transactions that might be incorrectly categorized\n\
                            4. Help me split transactions if needed (e.g., grocery store purchases that include non-food items)\n\
                            5. Update transaction categories to improve budget accuracy\n\n\
                            Start by fetching recent transactions and available categories, \
                            then help me systematically review and categorize transactions.",
                            budget_id
                        )
                    }
                }
            ]
        }))
    }
}

impl Default for PromptHandlers {
    fn default() -> Self {
        Self::new()
    }
}
