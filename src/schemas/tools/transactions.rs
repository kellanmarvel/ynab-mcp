use serde_json::{json, Value};

pub struct TransactionToolSchemas;

impl TransactionToolSchemas {
    pub fn get_transaction_tools() -> Vec<Value> {
        vec![
            Self::create_transaction_schema(),
            Self::create_transactions_bulk_schema(),
            Self::update_transaction_schema(),
            Self::approve_transaction_schema(),
            Self::clear_transaction_schema(),
        ]
    }

    pub fn create_transaction_schema() -> Value {
        json!({
            "name": "create_transaction",
            "description": "Create a new transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "account_id": {
                        "type": "string",
                        "description": "Account ID where the transaction occurred"
                    },
                    "amount": {
                        "type": ["string", "number"],
                        "description": "Transaction amount (positive for inflow, negative for outflow)"
                    },
                    "date": {
                        "type": "string",
                        "description": "Transaction date in YYYY-MM-DD format"
                    },
                    "payee_id": {
                        "type": "string",
                        "description": "Optional payee ID"
                    },
                    "payee_name": {
                        "type": "string",
                        "description": "Optional payee name (alternative to payee_id)"
                    },
                    "category_id": {
                        "type": "string",
                        "description": "Optional category ID for budgeting"
                    },
                    "memo": {
                        "type": "string",
                        "description": "Optional memo/note for the transaction"
                    },
                    "cleared": {
                        "type": "boolean",
                        "description": "Whether the transaction is cleared"
                    },
                    "flag_color": {
                        "type": "string",
                        "description": "Optional flag color (red, orange, yellow, green, blue, purple)"
                    },
                    "flag_name": {
                        "type": "string",
                        "description": "Optional flag name"
                    }
                },
                "required": ["budget_id", "account_id", "amount", "date"]
            }
        })
    }

    pub fn create_transactions_bulk_schema() -> Value {
        json!({
            "name": "create_transactions_bulk",
            "description": "Create multiple transactions using intelligent batch processing with concurrency control, retry logic, and partial failure handling (supports up to 1000 transactions)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "transactions": {
                        "type": "array",
                        "description": "Array of transactions to create (supports up to 1000 transactions with intelligent batching)",
                        "minItems": 1,
                        "maxItems": 1000,
                        "items": {
                            "type": "object",
                            "properties": {
                                "account_id": {
                                    "type": "string",
                                    "description": "Account ID where the transaction occurred"
                                },
                                "amount": {
                                    "type": ["string", "number"],
                                    "description": "Transaction amount (positive for inflow, negative for outflow)"
                                },
                                "date": {
                                    "type": "string",
                                    "description": "Transaction date in YYYY-MM-DD format"
                                },
                                "payee_name": {
                                    "type": "string",
                                    "description": "Payee name for the transaction"
                                },
                                "category_id": {
                                    "type": "string",
                                    "description": "Optional category ID for budgeting"
                                },
                                "memo": {
                                    "type": "string",
                                    "description": "Optional memo/note for the transaction"
                                },
                                "cleared": {
                                    "type": "string",
                                    "description": "Optional cleared status (cleared, uncleared, reconciled)"
                                },
                                "approved": {
                                    "type": "boolean",
                                    "description": "Optional approval status"
                                },
                                "flag_color": {
                                    "type": "string",
                                    "description": "Optional flag color"
                                },
                                "flag_name": {
                                    "type": "string",
                                    "description": "Optional flag name"
                                },
                                "import_id": {
                                    "type": "string",
                                    "description": "Optional unique import ID for duplicate detection"
                                }
                            },
                            "required": ["account_id", "amount", "date", "payee_name"]
                        }
                    }
                },
                "required": ["budget_id", "transactions"]
            }
        })
    }

    pub fn update_transaction_schema() -> Value {
        json!({
            "name": "update_transaction",
            "description": "Update an existing transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "transaction_id": {
                        "type": "string",
                        "description": "Transaction ID to update"
                    },
                    "account_id": {
                        "type": "string",
                        "description": "Account ID where the transaction occurred"
                    },
                    "amount": {
                        "type": ["string", "number"],
                        "description": "Transaction amount (positive for inflow, negative for outflow)"
                    },
                    "date": {
                        "type": "string",
                        "description": "Transaction date in YYYY-MM-DD format"
                    },
                    "payee_id": {
                        "type": "string",
                        "description": "Optional payee ID"
                    },
                    "category_id": {
                        "type": "string",
                        "description": "Optional category ID for budgeting"
                    },
                    "memo": {
                        "type": "string",
                        "description": "Optional memo/note for the transaction"
                    },
                    "cleared": {
                        "type": "boolean",
                        "description": "Whether the transaction is cleared"
                    },
                    "flag_color": {
                        "type": "string",
                        "description": "Optional flag color (red, orange, yellow, green, blue, purple)"
                    },
                    "flag_name": {
                        "type": "string",
                        "description": "Optional flag name"
                    }
                },
                "required": ["budget_id", "transaction_id"]
            }
        })
    }

    pub fn approve_transaction_schema() -> Value {
        json!({
            "name": "approve_transaction",
            "description": "Approve a pending transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "transaction_id": {
                        "type": "string",
                        "description": "Transaction ID to approve"
                    }
                },
                "required": ["budget_id", "transaction_id"]
            }
        })
    }

    pub fn clear_transaction_schema() -> Value {
        json!({
            "name": "clear_transaction",
            "description": "Change the cleared status of a transaction in YNAB",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "budget_id": {
                        "type": "string",
                        "description": "Budget ID (use 'default' for primary budget)"
                    },
                    "transaction_id": {
                        "type": "string",
                        "description": "Transaction ID to update"
                    },
                    "clear_state": {
                        "type": "string",
                        "description": "New cleared state for the transaction",
                        "enum": ["cleared", "uncleared", "reconciled"]
                    }
                },
                "required": ["budget_id", "transaction_id"]
            }
        })
    }
}