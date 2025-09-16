use serde_json::Value;

use self::tools::TransactionToolSchemas;

mod tools;

/// Tool schema definitions for the YNAB MCP server
pub struct ToolSchemas;

impl ToolSchemas {
    /// Get all tool schemas organized by category
    pub fn get_all_tools() -> Vec<Value> {
        let mut tools = Vec::new();
        
        // Transaction tools
        tools.extend(TransactionToolSchemas::get_transaction_tools());
        
        // Transfer tools  
        tools.extend(Self::get_transfer_tools());
        
        // Analysis tools
        tools.extend(Self::get_analysis_tools());
        
        // Budget tools
        tools.extend(Self::get_budget_tools());
        
        // System tools
        tools.extend(Self::get_system_tools());
        
        tools
    }

    /// Transaction management tools
    pub fn get_transaction_tools() -> Vec<Value> {
        TransactionToolSchemas::get_transaction_tools()
    }

    /// Transfer management tools
    pub fn get_transfer_tools() -> Vec<Value> {
        vec![
            Self::create_transfer_schema(),
        ]
    }

    /// Data analysis tools
    pub fn get_analysis_tools() -> Vec<Value> {
        vec![
            Self::analyze_transactions_schema(),
            Self::get_transactions_by_date_range_schema(),
            Self::analyze_spending_by_category_schema(),
            Self::get_all_account_transactions_schema(),
            Self::get_transactions_delta_schema(),
            Self::get_transactions_since_date_schema(),
            Self::get_transactions_paginated_schema(),
            Self::get_transactions_with_fields_schema(),
        ]
    }

    /// Budget management tools
    pub fn get_budget_tools() -> Vec<Value> {
        vec![
            Self::assign_category_budget_schema(),
        ]
    }

    /// System management and monitoring tools
    pub fn get_system_tools() -> Vec<Value> {
        vec![
            Self::health_check_schema(),
            Self::cache_management_schema(),
            Self::get_operation_progress_schema(),
            Self::list_active_operations_schema(),
            Self::cancel_operation_schema(),
            Self::get_metrics_schema(),
            Self::get_health_status_schema(),
            Self::export_metrics_schema(),
            Self::list_plugins_schema(),
            Self::get_plugin_info_schema(),
            Self::run_performance_benchmark_schema(),
            Self::performance_diagnostics_schema(),
        ]
    }

    // Individual tool schema definitions - keeping these in the main file for now
    // These could be further split if needed, but the main organization is by category

    pub fn create_transfer_schema() -> Value {
        // [Implementation moved from original file - content preserved]
        serde_json::json!({
            "name": "create_transfer",
            "description": "Create a transfer between two accounts in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "from_account_id": {
                        "type": "string",
                        "description": "Source account ID"
                    },
                    "to_account_id": {
                        "type": "string",
                        "description": "Destination account ID"
                    },
                    "amount": {
                        "type": ["string", "number"],
                        "description": "Transfer amount (always positive)"
                    },
                    "date": {
                        "type": "string",
                        "description": "Transfer date in YYYY-MM-DD format"
                    },
                    "memo": {
                        "type": "string",
                        "description": "Optional memo/note for the transfer"
                    }
                },
                "required": ["budget_id", "from_account_id", "to_account_id", "amount", "date"]
            }
        })
    }

    // Analysis tool schemas
    pub fn analyze_transactions_schema() -> Value {
        serde_json::json!({
            "name": "analyze_transactions",
            "description": "Comprehensive transaction analysis with unlimited data access and filtering",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "account_id": {
                        "type": "string",
                        "description": "Optional account ID filter"
                    },
                    "category_id": {
                        "type": "string",
                        "description": "Optional category ID filter"
                    },
                    "from_date": {
                        "type": "string",
                        "description": "Optional start date filter (YYYY-MM-DD)"
                    },
                    "to_date": {
                        "type": "string",
                        "description": "Optional end date filter (YYYY-MM-DD)"
                    }
                },
                "required": ["budget_id"]
            }
        })
    }

    pub fn get_transactions_by_date_range_schema() -> Value {
        serde_json::json!({
            "name": "get_transactions_by_date_range",
            "description": "Get ALL transactions within a specific date range (no limits)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "from_date": {
                        "type": "string",
                        "description": "Start date (YYYY-MM-DD format)"
                    },
                    "to_date": {
                        "type": "string",
                        "description": "End date (YYYY-MM-DD format)"
                    }
                },
                "required": ["budget_id", "from_date", "to_date"]
            }
        })
    }

    pub fn analyze_spending_by_category_schema() -> Value {
        serde_json::json!({
            "name": "analyze_spending_by_category",
            "description": "Analyze spending patterns by category over time (unlimited data)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "months_back": {
                        "type": "number",
                        "description": "Number of months to analyze (default: 12)"
                    }
                },
                "required": ["budget_id"]
            }
        })
    }

    pub fn get_all_account_transactions_schema() -> Value {
        serde_json::json!({
            "name": "get_all_account_transactions",
            "description": "Get ALL transactions for a specific account (no limits)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "account_id": {
                        "type": "string",
                        "description": "Account ID to get all transactions for"
                    }
                },
                "required": ["budget_id", "account_id"]
            }
        })
    }

    pub fn assign_category_budget_schema() -> Value {
        serde_json::json!({
            "name": "assign_category_budget",
            "description": "Assign money to a budget category for a specific month",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "category_id": {
                        "type": "string",
                        "description": "Category ID to assign budget to"
                    },
                    "budgeted_amount": {
                        "type": ["string", "number"],
                        "description": "Amount to assign (e.g., '50.00' for £50 or 50000 for milliunits)"
                    },
                    "month": {
                        "type": "string",
                        "description": "Month in YYYY-MM-DD format (default: current month)"
                    }
                },
                "required": ["budget_id", "category_id", "budgeted_amount"]
            }
        })
    }

    // System tool schemas  
    pub fn health_check_schema() -> Value {
        serde_json::json!({
            "name": "health_check",
            "description": "Check the health and connectivity of the YNAB API connection",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        })
    }

    pub fn cache_management_schema() -> Value {
        serde_json::json!({
            "name": "cache_management",
            "description": "Manage the YNAB API response cache for improved performance",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "Action to perform: 'stats' to show cache statistics, 'cleanup' to remove expired entries, 'clear' to remove all entries",
                        "enum": ["stats", "cleanup", "clear"]
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }
        })
    }

    // Additional system tools - keeping simplified for now
    pub fn get_operation_progress_schema() -> Value {
        serde_json::json!({
            "name": "get_operation_progress",
            "description": "Get progress information for a running operation"
        })
    }

    pub fn list_active_operations_schema() -> Value {
        serde_json::json!({
            "name": "list_active_operations", 
            "description": "List all currently active operations"
        })
    }

    pub fn cancel_operation_schema() -> Value {
        serde_json::json!({
            "name": "cancel_operation",
            "description": "Cancel a running operation"
        })
    }

    pub fn get_metrics_schema() -> Value {
        serde_json::json!({
            "name": "get_metrics",
            "description": "Get system performance metrics"
        })
    }

    pub fn get_health_status_schema() -> Value {
        serde_json::json!({
            "name": "get_health_status",
            "description": "Get detailed health status information"
        })
    }

    pub fn export_metrics_schema() -> Value {
        serde_json::json!({
            "name": "export_metrics", 
            "description": "Export system metrics"
        })
    }

    pub fn list_plugins_schema() -> Value {
        serde_json::json!({
            "name": "list_plugins",
            "description": "List available plugins"
        })
    }

    pub fn get_plugin_info_schema() -> Value {
        serde_json::json!({
            "name": "get_plugin_info",
            "description": "Get information about a specific plugin"
        })
    }

    pub fn run_performance_benchmark_schema() -> Value {
        serde_json::json!({
            "name": "run_performance_benchmark",
            "description": "Run performance benchmark tests"
        })
    }

    pub fn performance_diagnostics_schema() -> Value {
        serde_json::json!({
            "name": "performance_diagnostics",
            "description": "Run performance diagnostics"
        })
    }

    // Simplified schemas for remaining query tools
    pub fn get_transactions_delta_schema() -> Value {
        serde_json::json!({
            "name": "get_transactions_delta",
            "description": "Get transaction changes since last sync"
        })
    }

    pub fn get_transactions_since_date_schema() -> Value {
        serde_json::json!({
            "name": "get_transactions_since_date", 
            "description": "Get transactions since a specific date"
        })
    }

    pub fn get_transactions_paginated_schema() -> Value {
        serde_json::json!({
            "name": "get_transactions_paginated",
            "description": "Get transactions with pagination support"
        })
    }

    pub fn get_transactions_with_fields_schema() -> Value {
        serde_json::json!({
            "name": "get_transactions_with_fields",
            "description": "Get transactions with specific field selection"
        })
    }
}