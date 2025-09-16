use serde_json::{json, Value};

/// Simplified prompt schema definitions for the YNAB MCP server
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

    fn quick_expense_prompt() -> Value {
        json!({
            "name": "quick_expense",
            "description": "Quickly log an expense with smart categorization suggestions",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": false
                }
            ]
        })
    }

    fn budget_review_prompt() -> Value {
        json!({
            "name": "budget_review",
            "description": "Review budget performance with insights and recommendations",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": false
                },
                {
                    "name": "month",
                    "description": "Month to review (default: current month)",
                    "required": false
                }
            ]
        })
    }

    fn spending_analysis_prompt() -> Value {
        json!({
            "name": "spending_analysis",
            "description": "Analyze spending patterns and trends with actionable insights",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": false
                },
                {
                    "name": "category_id",
                    "description": "Optional category ID to focus analysis",
                    "required": false
                },
                {
                    "name": "timeframe",
                    "description": "Time period for analysis (default: current month)",
                    "required": false
                }
            ]
        })
    }

    fn account_reconciliation_prompt() -> Value {
        json!({
            "name": "account_reconciliation",
            "description": "Guide through account reconciliation process step by step",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": false
                },
                {
                    "name": "account_id",
                    "description": "Account ID to reconcile",
                    "required": true
                }
            ]
        })
    }

    fn transaction_categorization_prompt() -> Value {
        json!({
            "name": "transaction_categorization",
            "description": "Help categorize uncategorized transactions with smart suggestions",
            "arguments": [
                {
                    "name": "budget_id",
                    "description": "Budget ID (use 'default' for primary budget)",
                    "required": false
                }
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_prompts_returns_expected_count() {
        let prompts = PromptSchemas::get_all_prompts();
        assert_eq!(prompts.len(), 5);
    }

    #[test]
    fn test_all_prompts_have_required_fields() {
        let prompts = PromptSchemas::get_all_prompts();

        for prompt in prompts {
            assert!(
                prompt.get("name").is_some(),
                "Prompt should have name field"
            );
            assert!(
                prompt.get("description").is_some(),
                "Prompt should have description field"
            );
            assert!(
                prompt.get("arguments").is_some(),
                "Prompt should have arguments field"
            );
        }
    }

    #[test]
    fn test_prompt_names() {
        let prompts = PromptSchemas::get_all_prompts();
        let prompt_names: Vec<_> = prompts
            .iter()
            .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(prompt_names.contains(&"quick_expense"));
        assert!(prompt_names.contains(&"budget_review"));
        assert!(prompt_names.contains(&"spending_analysis"));
        assert!(prompt_names.contains(&"account_reconciliation"));
        assert!(prompt_names.contains(&"transaction_categorization"));
    }
}
