use serde_json::{json, Value};

/// Simplified tool schema definitions for the YNAB MCP server
pub struct ToolSchemas;

impl ToolSchemas {
    /// Get all tool schemas organized by category
    pub fn get_all_tools() -> Vec<Value> {
        let mut tools = Vec::new();

        // Core YNAB operations
        tools.extend(Self::get_transaction_tools());
        tools.extend(Self::get_transfer_tools());
        tools.extend(Self::get_analysis_tools());
        tools.extend(Self::get_budget_tools());
        tools.extend(Self::get_system_tools());

        tools
    }

    /// Transaction management tools
    pub fn get_transaction_tools() -> Vec<Value> {
        vec![
            Self::create_transaction_tool(),
            Self::update_transaction_tool(),
            Self::approve_transaction_tool(),
            Self::clear_transaction_tool(),
        ]
    }

    /// Transfer management tools
    pub fn get_transfer_tools() -> Vec<Value> {
        vec![Self::create_transfer_tool()]
    }

    /// Data analysis tools
    pub fn get_analysis_tools() -> Vec<Value> {
        vec![
            Self::analyze_transactions_tool(),
            Self::analyze_spending_by_category_tool(),
            Self::get_transactions_by_date_range_tool(),
            Self::get_all_account_transactions_tool(),
        ]
    }

    /// Budget management tools
    pub fn get_budget_tools() -> Vec<Value> {
        vec![Self::assign_category_budget_tool()]
    }

    /// System management tools
    pub fn get_system_tools() -> Vec<Value> {
        vec![
            Self::health_check_tool(),
            Self::cache_management_tool(),
            Self::get_api_documentation_tool(),
        ]
    }

    // Common parameter helpers
    fn budget_id_param() -> Value {
        json!({
            "type": "string",
            "description": "Budget ID (use 'default' for primary budget)"
        })
    }

    fn account_id_param() -> Value {
        json!({
            "type": "string",
            "description": "Account ID where the transaction occurred"
        })
    }

    fn amount_param() -> Value {
        json!({
            "type": ["string", "number"],
            "description": "Transaction amount (positive for inflow, negative for outflow)"
        })
    }

    fn date_param() -> Value {
        json!({
            "type": "string",
            "description": "Transaction date in YYYY-MM-DD format"
        })
    }

    fn optional_string_param(desc: &str) -> Value {
        json!({
            "type": "string",
            "description": desc
        })
    }

    fn optional_bool_param(desc: &str) -> Value {
        json!({
            "type": "boolean",
            "description": desc
        })
    }

    // Tool schema definitions using common parameters
    fn create_transaction_tool() -> Value {
        json!({
            "name": "create_transaction",
            "description": "Create a new transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "account_id": Self::account_id_param(),
                    "amount": Self::amount_param(),
                    "date": Self::date_param(),
                    "payee_id": Self::optional_string_param("Optional payee ID"),
                    "category_id": Self::optional_string_param("Optional category ID for budgeting"),
                    "memo": Self::optional_string_param("Optional memo/note for the transaction"),
                    "cleared": Self::optional_bool_param("Whether the transaction is cleared"),
                    "flag_color": Self::optional_string_param("Optional flag color (red, orange, yellow, green, blue, purple)")
                },
                "required": ["budget_id", "account_id", "amount", "date"]
            }
        })
    }

    fn update_transaction_tool() -> Value {
        json!({
            "name": "update_transaction",
            "description": "Update an existing transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "transaction_id": Self::optional_string_param("Transaction ID to update"),
                    "account_id": Self::account_id_param(),
                    "amount": Self::amount_param(),
                    "date": Self::date_param(),
                    "payee_id": Self::optional_string_param("Optional payee ID"),
                    "category_id": Self::optional_string_param("Optional category ID for budgeting"),
                    "memo": Self::optional_string_param("Optional memo/note for the transaction"),
                    "cleared": Self::optional_bool_param("Whether the transaction is cleared")
                },
                "required": ["budget_id", "transaction_id"]
            }
        })
    }

    fn approve_transaction_tool() -> Value {
        json!({
            "name": "approve_transaction",
            "description": "Approve a pending transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "transaction_id": Self::optional_string_param("Transaction ID to approve")
                },
                "required": ["budget_id", "transaction_id"]
            }
        })
    }

    fn clear_transaction_tool() -> Value {
        json!({
            "name": "clear_transaction",
            "description": "Change the cleared status of a transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "transaction_id": Self::optional_string_param("Transaction ID to update"),
                    "clear_state": json!({
                        "type": "string",
                        "enum": ["cleared", "uncleared", "reconciled"],
                        "description": "New cleared state for the transaction"
                    })
                },
                "required": ["budget_id", "transaction_id", "clear_state"]
            }
        })
    }

    fn create_transfer_tool() -> Value {
        json!({
            "name": "create_transfer",
            "description": "Create a transfer between two accounts in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "from_account_id": Self::optional_string_param("Source account ID"),
                    "to_account_id": Self::optional_string_param("Destination account ID"),
                    "amount": json!({
                        "type": ["string", "number"],
                        "description": "Transfer amount (always positive)"
                    }),
                    "date": Self::date_param(),
                    "memo": Self::optional_string_param("Optional memo/note for the transfer")
                },
                "required": ["budget_id", "from_account_id", "to_account_id", "amount", "date"]
            }
        })
    }

    fn analyze_transactions_tool() -> Value {
        json!({
            "name": "analyze_transactions",
            "description": "Comprehensive transaction analysis with unlimited data access and filtering",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "account_id": Self::optional_string_param("Optional account ID filter"),
                    "category_id": Self::optional_string_param("Optional category ID filter"),
                    "from_date": Self::optional_string_param("Optional start date filter (YYYY-MM-DD)"),
                    "to_date": Self::optional_string_param("Optional end date filter (YYYY-MM-DD)")
                },
                "required": ["budget_id"]
            }
        })
    }

    fn analyze_spending_by_category_tool() -> Value {
        json!({
            "name": "analyze_spending_by_category",
            "description": "Analyze spending patterns by category over time (unlimited data)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "months_back": json!({
                        "type": "number",
                        "description": "Number of months to analyze (default: 12)"
                    })
                },
                "required": ["budget_id"]
            }
        })
    }

    fn get_transactions_by_date_range_tool() -> Value {
        json!({
            "name": "get_transactions_by_date_range",
            "description": "Get ALL transactions within a specific date range (no limits)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "from_date": json!({
                        "type": "string",
                        "description": "Start date (YYYY-MM-DD format)"
                    }),
                    "to_date": json!({
                        "type": "string",
                        "description": "End date (YYYY-MM-DD format)"
                    })
                },
                "required": ["budget_id", "from_date", "to_date"]
            }
        })
    }

    fn get_all_account_transactions_tool() -> Value {
        json!({
            "name": "get_all_account_transactions",
            "description": "Get ALL transactions for a specific account (no limits)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "account_id": Self::optional_string_param("Account ID to get all transactions for")
                },
                "required": ["budget_id", "account_id"]
            }
        })
    }

    fn assign_category_budget_tool() -> Value {
        json!({
            "name": "assign_category_budget",
            "description": "Assign money to a budget category for a specific month",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": Self::budget_id_param(),
                    "category_id": Self::optional_string_param("Category ID to assign budget to"),
                    "budgeted_amount": json!({
                        "type": ["string", "number"],
                        "description": "Amount to assign (e.g., '50.00' for £50 or 50000 for milliunits)"
                    }),
                    "month": Self::optional_string_param("Month in YYYY-MM-DD format (default: current month)")
                },
                "required": ["budget_id", "category_id", "budgeted_amount"]
            }
        })
    }

    fn health_check_tool() -> Value {
        json!({
            "name": "health_check",
            "description": "Check the health and connectivity of the YNAB API connection",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        })
    }

    fn cache_management_tool() -> Value {
        json!({
            "name": "cache_management",
            "description": "Manage the YNAB API response cache for improved performance",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": json!({
                        "type": "string",
                        "enum": ["stats", "cleanup", "clear"],
                        "description": "Action to perform: 'stats' to show cache statistics, 'cleanup' to remove expired entries, 'clear' to remove all entries"
                    })
                },
                "required": ["action"]
            }
        })
    }

    fn get_api_documentation_tool() -> Value {
        json!({
            "name": "get_api_documentation",
            "description": "Get comprehensive API documentation for tools, resources, MCP protocol, and usage examples",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "type": json!({
                        "type": "string",
                        "enum": ["tools", "resources", "mcp", "examples", "all"],
                        "description": "Type of documentation to retrieve: 'tools' (tool reference), 'resources' (resource URIs), 'mcp' (protocol info), 'examples' (usage examples), 'all' (complete documentation)"
                    })
                },
                "required": [],
                "additionalProperties": false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_tools_returns_expected_count() {
        let tools = ToolSchemas::get_all_tools();
        assert_eq!(tools.len(), 13); // Expected number of simplified tools (added documentation tool)
    }

    #[test]
    fn test_all_tools_have_required_fields() {
        let tools = ToolSchemas::get_all_tools();

        for tool in tools {
            assert!(tool.get("name").is_some(), "Tool should have name field");
            assert!(
                tool.get("description").is_some(),
                "Tool should have description field"
            );
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool should have inputSchema field"
            );
        }
    }

    #[test]
    fn test_transaction_tools() {
        let tools = ToolSchemas::get_transaction_tools();
        assert_eq!(tools.len(), 4);

        let tool_names: Vec<_> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(tool_names.contains(&"create_transaction"));
        assert!(tool_names.contains(&"update_transaction"));
    }

    #[test]
    fn test_common_parameters_consistency() {
        let budget_param1 = ToolSchemas::budget_id_param();
        let budget_param2 = ToolSchemas::budget_id_param();
        assert_eq!(
            budget_param1, budget_param2,
            "Common parameters should be consistent"
        );
    }
}
