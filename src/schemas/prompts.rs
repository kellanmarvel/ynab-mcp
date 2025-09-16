use serde_json::{json, Value};

/// Prompt schema definitions for the YNAB MCP server
pub struct PromptSchemas;

impl PromptSchemas {
    /// Get all prompt schemas
    pub fn get_all_prompts() -> Vec<Value> {
        vec![
            Self::quick_expense_prompt(),
            Self::budget_review_prompt(),
            Self::spending_analysis_prompt(),
            Self::account_reconciliation_prompt(),
            Self::transaction_categorization_prompt(),
        ]
    }

    pub fn quick_expense_prompt() -> Value {
        json!({
            "name": "quick_expense",
            "description": "Quickly log an expense with smart categorization suggestions",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": true
                }
            ]
        })
    }

    pub fn budget_review_prompt() -> Value {
        json!({
            "name": "budget_review",
            "description": "Generate a comprehensive budget review with insights and recommendations",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": true
                },
                {
                    "name": "month",
                    "description": "Month to review (YYYY-MM-DD format, default: current month)",
                    "required": false
                }
            ]
        })
    }

    pub fn spending_analysis_prompt() -> Value {
        json!({
            "name": "spending_analysis",
            "description": "Analyze spending patterns and provide actionable insights",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": true
                },
                {
                    "name": "category_id",
                    "description": "Optional category ID to focus analysis on",
                    "required": false
                },
                {
                    "name": "timeframe",
                    "description": "Analysis timeframe: '1m', '3m', '6m', '1y' (default: '3m')",
                    "required": false
                }
            ]
        })
    }

    pub fn account_reconciliation_prompt() -> Value {
        json!({
            "name": "account_reconciliation",
            "description": "Help reconcile account balances and identify discrepancies",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": true
                },
                {
                    "name": "account_id",
                    "description": "Account ID to reconcile",
                    "required": true
                }
            ]
        })
    }

    pub fn transaction_categorization_prompt() -> Value {
        json!({
            "name": "transaction_categorization",
            "description": "Smart categorization suggestions for uncategorized transactions",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": true
                },
                {
                    "name": "limit",
                    "description": "Number of transactions to analyze (default: 20, max: 100)",
                    "required": false
                }
            ]
        })
    }
}