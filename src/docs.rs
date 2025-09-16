use serde_json::{json, Value};

/// Comprehensive API documentation for the YNAB MCP server
///
/// This module provides detailed documentation for all tools, resources,
/// and MCP protocol capabilities to help AI assistants and developers
/// understand and effectively use the server.
pub struct ApiDocumentation;

impl ApiDocumentation {
    /// Get comprehensive documentation for all tools
    pub fn get_tools_documentation() -> Value {
        json!({
            "version": "1.0.0",
            "last_updated": "2024-01-15",
            "description": "YNAB MCP Server Tool Documentation - Complete reference for all available tools",
            "categories": {
                "transaction_management": {
                    "description": "Create, update, and manage YNAB transactions",
                    "tools": [
                        {
                            "name": "create_transaction",
                            "description": "Create a new transaction in your YNAB budget",
                            "required_parameters": ["budget_id", "account_id", "amount", "date"],
                            "optional_parameters": ["payee_id", "category_id", "memo", "cleared", "flag_color", "flag_name"],
                            "examples": [
                                {
                                    "description": "Create a simple expense transaction",
                                    "parameters": {
                                        "budget_id": "default",
                                        "account_id": "12345678-1234-1234-1234-123456789abc",
                                        "amount": -2500,
                                        "date": "2024-01-15",
                                        "memo": "Coffee shop"
                                    },
                                    "expected_result": "Transaction created successfully with milliunits precision"
                                },
                                {
                                    "description": "Create income transaction with category",
                                    "parameters": {
                                        "budget_id": "default",
                                        "account_id": "12345678-1234-1234-1234-123456789abc",
                                        "amount": 500000,
                                        "date": "2024-01-15",
                                        "category_id": "87654321-4321-4321-4321-210987654321",
                                        "memo": "Freelance work"
                                    }
                                }
                            ],
                            "common_errors": {
                                "invalid_amount": "Amount must be an integer in milliunits (1000 milliunits = $1.00)",
                                "invalid_account_id": "Account ID must be a valid UUID from your YNAB budget",
                                "invalid_date": "Date must be in YYYY-MM-DD format",
                                "permission_denied": "Check read-only mode setting and API token permissions"
                            },
                            "tips": [
                                "Use negative amounts for expenses, positive for income",
                                "1 dollar = 1000 milliunits for precision",
                                "Get account IDs from resources/list endpoint",
                                "Use 'default' for budget_id to access primary budget"
                            ]
                        },
                        {
                            "name": "update_transaction",
                            "description": "Update an existing transaction",
                            "required_parameters": ["budget_id", "transaction_id"],
                            "optional_parameters": ["account_id", "amount", "date", "payee_id", "category_id", "memo", "cleared"],
                            "examples": [
                                {
                                    "description": "Update transaction amount and memo",
                                    "parameters": {
                                        "budget_id": "default",
                                        "transaction_id": "11111111-1111-1111-1111-111111111111",
                                        "amount": -3000,
                                        "memo": "Coffee shop - updated amount"
                                    }
                                }
                            ]
                        },
                        {
                            "name": "approve_transaction",
                            "description": "Approve a pending transaction (from bank imports)",
                            "required_parameters": ["budget_id", "transaction_id"],
                            "examples": [
                                {
                                    "description": "Approve an imported transaction",
                                    "parameters": {
                                        "budget_id": "default",
                                        "transaction_id": "22222222-2222-2222-2222-222222222222"
                                    }
                                }
                            ]
                        },
                        {
                            "name": "clear_transaction",
                            "description": "Change the cleared status of a transaction",
                            "required_parameters": ["budget_id", "transaction_id", "clear_state"],
                            "parameter_details": {
                                "clear_state": {
                                    "type": "string",
                                    "allowed_values": ["cleared", "uncleared", "reconciled"],
                                    "description": "New cleared state for the transaction"
                                }
                            }
                        }
                    ]
                },
                "transfer_management": {
                    "description": "Transfer money between accounts",
                    "tools": [
                        {
                            "name": "create_transfer",
                            "description": "Create a transfer between two accounts in YNAB",
                            "required_parameters": ["budget_id", "from_account_id", "to_account_id", "amount", "date"],
                            "optional_parameters": ["memo"],
                            "examples": [
                                {
                                    "description": "Transfer from checking to savings",
                                    "parameters": {
                                        "budget_id": "default",
                                        "from_account_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                                        "to_account_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                                        "amount": 100000,
                                        "date": "2024-01-15",
                                        "memo": "Monthly savings"
                                    }
                                }
                            ],
                            "tips": [
                                "Transfer amounts are always positive",
                                "Both accounts must belong to the same budget",
                                "Transfers automatically balance (debit from source, credit to destination)"
                            ]
                        }
                    ]
                },
                "data_analysis": {
                    "description": "Analyze spending patterns and financial data",
                    "tools": [
                        {
                            "name": "analyze_transactions",
                            "description": "Comprehensive transaction analysis with unlimited data access",
                            "required_parameters": ["budget_id"],
                            "optional_parameters": ["account_id", "category_id", "from_date", "to_date"],
                            "examples": [
                                {
                                    "description": "Analyze all transactions for current month",
                                    "parameters": {
                                        "budget_id": "default",
                                        "from_date": "2024-01-01",
                                        "to_date": "2024-01-31"
                                    }
                                },
                                {
                                    "description": "Analyze grocery spending",
                                    "parameters": {
                                        "budget_id": "default",
                                        "category_id": "grocery-category-id"
                                    }
                                }
                            ]
                        },
                        {
                            "name": "analyze_spending_by_category",
                            "description": "Analyze spending patterns by category over time",
                            "required_parameters": ["budget_id"],
                            "optional_parameters": ["months_back"],
                            "parameter_details": {
                                "months_back": {
                                    "type": "number",
                                    "default": 12,
                                    "description": "Number of months to analyze (default: 12)"
                                }
                            }
                        },
                        {
                            "name": "get_transactions_by_date_range",
                            "description": "Get ALL transactions within a specific date range (no pagination limits)",
                            "required_parameters": ["budget_id", "from_date", "to_date"],
                            "examples": [
                                {
                                    "description": "Get all December transactions",
                                    "parameters": {
                                        "budget_id": "default",
                                        "from_date": "2023-12-01",
                                        "to_date": "2023-12-31"
                                    }
                                }
                            ]
                        },
                        {
                            "name": "get_all_account_transactions",
                            "description": "Get ALL transactions for a specific account (no limits)",
                            "required_parameters": ["budget_id", "account_id"]
                        }
                    ]
                },
                "budget_management": {
                    "description": "Manage budget categories and assignments",
                    "tools": [
                        {
                            "name": "set_category_budget",
                            "description": "Assign money to a budget category for a specific month",
                            "required_parameters": ["budget_id", "category_id", "budgeted_amount"],
                            "optional_parameters": ["month"],
                            "parameter_details": {
                                "budgeted_amount": {
                                    "type": "string or number",
                                    "examples": ["50.00", "50000", 50000],
                                    "description": "Amount in dollars (e.g., '50.00') or milliunits (e.g., 50000)"
                                },
                                "month": {
                                    "type": "string",
                                    "format": "YYYY-MM-DD",
                                    "default": "current month",
                                    "description": "Month to assign budget for"
                                }
                            },
                            "examples": [
                                {
                                    "description": "Assign $200 to groceries for current month",
                                    "parameters": {
                                        "budget_id": "default",
                                        "category_id": "grocery-category-id",
                                        "budgeted_amount": "200.00"
                                    }
                                }
                            ]
                        }
                    ]
                },
                "system_management": {
                    "description": "Server health, cache management, and system operations",
                    "tools": [
                        {
                            "name": "health_check",
                            "description": "Check the health and connectivity of the YNAB API connection",
                            "required_parameters": [],
                            "response_format": {
                                "status": "healthy or unhealthy",
                                "uptime_seconds": "server uptime",
                                "response_time_ms": "API response time",
                                "api_connectivity": "YNAB API status"
                            }
                        },
                        {
                            "name": "cache_management",
                            "description": "Manage the YNAB API response cache for improved performance",
                            "required_parameters": ["action"],
                            "parameter_details": {
                                "action": {
                                    "type": "string",
                                    "allowed_values": ["stats", "cleanup", "clear"],
                                    "descriptions": {
                                        "stats": "Show cache statistics and hit rates",
                                        "cleanup": "Remove expired cache entries",
                                        "clear": "Remove all cache entries"
                                    }
                                }
                            }
                        }
                    ]
                }
            },
            "global_parameters": {
                "budget_id": {
                    "description": "YNAB budget identifier",
                    "special_values": {
                        "default": "Use the primary/first budget",
                        "last-used": "Use the most recently accessed budget"
                    },
                    "format": "UUID string or special value"
                },
                "amounts": {
                    "format": "milliunits (1000 milliunits = $1.00)",
                    "examples": {
                        "$1.00": 1000,
                        "$10.50": 10500,
                        "-$5.25": -5250
                    },
                    "notes": "Always use integers for milliunits precision"
                },
                "dates": {
                    "format": "YYYY-MM-DD",
                    "examples": ["2024-01-15", "2023-12-31"],
                    "timezone": "Server timezone (typically UTC)"
                },
                "uuids": {
                    "format": "Standard UUID format",
                    "example": "12345678-1234-1234-1234-123456789abc",
                    "how_to_get": "Use resources/list endpoint or other list operations"
                }
            },
            "error_codes": {
                "authentication_errors": {
                    "description": "API token issues, keyring problems",
                    "solutions": [
                        "Check YNAB API token at https://app.youneedabudget.com/settings/developer",
                        "Use 'ynab-mcp store-token' to store token securely",
                        "Verify token has not expired"
                    ]
                },
                "rate_limit_errors": {
                    "description": "YNAB API rate limit exceeded (200 requests/hour)",
                    "solutions": [
                        "Use cache_management tools to optimize requests",
                        "Wait 15+ minutes before retrying",
                        "Check rate limit status with health_check"
                    ]
                },
                "validation_errors": {
                    "description": "Invalid parameters or malformed requests",
                    "solutions": [
                        "Check parameter types and formats",
                        "Verify UUIDs are valid and exist",
                        "Use resources/list to get valid IDs"
                    ]
                }
            },
            "best_practices": {
                "performance": [
                    "Use cache_management to monitor and optimize cache usage",
                    "Batch related operations together",
                    "Use date ranges to limit large queries",
                    "Check health_check before bulk operations"
                ],
                "reliability": [
                    "Handle rate limit errors gracefully with retries",
                    "Validate parameters before making requests",
                    "Use read-only mode for exploration",
                    "Store sensitive tokens securely with keyring"
                ],
                "data_accuracy": [
                    "Use milliunits for precise money calculations",
                    "Verify account and category IDs exist before using",
                    "Use proper date formats (YYYY-MM-DD)",
                    "Double-check transaction amounts and signs"
                ]
            }
        })
    }

    /// Get comprehensive documentation for all resources
    pub fn get_resources_documentation() -> Value {
        json!({
            "version": "1.0.0",
            "last_updated": "2024-01-15",
            "description": "YNAB MCP Server Resource Documentation - Complete reference for all available resources",
            "base_uri": "ynab://",
            "resource_types": {
                "budgets": {
                    "description": "Access to YNAB budgets and their properties",
                    "uri_patterns": [
                        "ynab://budgets",
                        "ynab://budgets/{budget_id}"
                    ],
                    "examples": [
                        {
                            "uri": "ynab://budgets",
                            "description": "List all available budgets",
                            "response_contains": ["budget_id", "budget_name", "last_modified", "first_month", "last_month"]
                        },
                        {
                            "uri": "ynab://budgets/default",
                            "description": "Get details of the primary budget",
                            "response_contains": ["accounts", "categories", "payees", "currency_format"]
                        }
                    ]
                },
                "accounts": {
                    "description": "Bank accounts, credit cards, and other account types",
                    "uri_patterns": [
                        "ynab://budgets/{budget_id}/accounts",
                        "ynab://budgets/{budget_id}/accounts/{account_id}"
                    ],
                    "account_types": [
                        "checking", "savings", "creditCard", "cash",
                        "lineOfCredit", "otherAsset", "otherLiability"
                    ],
                    "examples": [
                        {
                            "uri": "ynab://budgets/default/accounts",
                            "description": "List all accounts in the default budget"
                        },
                        {
                            "uri": "ynab://budgets/default/accounts/12345678-1234-1234-1234-123456789abc",
                            "description": "Get specific account details including balance"
                        }
                    ]
                },
                "categories": {
                    "description": "Budget categories and category groups",
                    "uri_patterns": [
                        "ynab://budgets/{budget_id}/categories",
                        "ynab://budgets/{budget_id}/categories/{category_id}",
                        "ynab://budgets/{budget_id}/category_groups/{group_id}/categories"
                    ],
                    "examples": [
                        {
                            "uri": "ynab://budgets/default/categories",
                            "description": "List all categories with budgeted amounts and activity"
                        },
                        {
                            "uri": "ynab://budgets/default/categories/grocery-category-id",
                            "description": "Get specific category with detailed spending information"
                        }
                    ]
                },
                "transactions": {
                    "description": "All transaction data with filtering capabilities",
                    "uri_patterns": [
                        "ynab://budgets/{budget_id}/transactions",
                        "ynab://budgets/{budget_id}/accounts/{account_id}/transactions",
                        "ynab://budgets/{budget_id}/categories/{category_id}/transactions"
                    ],
                    "query_parameters": {
                        "since_date": "Get transactions since a specific date (YYYY-MM-DD)",
                        "type": "Filter by transaction type (uncategorized, unapproved)"
                    },
                    "examples": [
                        {
                            "uri": "ynab://budgets/default/transactions",
                            "description": "All transactions in the budget"
                        },
                        {
                            "uri": "ynab://budgets/default/accounts/checking-account-id/transactions",
                            "description": "All transactions for a specific account"
                        },
                        {
                            "uri": "ynab://budgets/default/categories/grocery-id/transactions",
                            "description": "All transactions in a specific category"
                        }
                    ]
                },
                "payees": {
                    "description": "Merchants, people, and organizations you pay",
                    "uri_patterns": [
                        "ynab://budgets/{budget_id}/payees",
                        "ynab://budgets/{budget_id}/payees/{payee_id}"
                    ],
                    "special_payees": {
                        "transfer_payees": "Generated automatically for transfers between accounts",
                        "starting_balance": "Special payee for account opening balances"
                    }
                },
                "months": {
                    "description": "Monthly budget data and category assignments",
                    "uri_patterns": [
                        "ynab://budgets/{budget_id}/months",
                        "ynab://budgets/{budget_id}/months/{month}",
                        "ynab://budgets/{budget_id}/months/{month}/categories"
                    ],
                    "month_format": "YYYY-MM-DD (first day of month)",
                    "examples": [
                        {
                            "uri": "ynab://budgets/default/months",
                            "description": "List all available months with budget data"
                        },
                        {
                            "uri": "ynab://budgets/default/months/2024-01-01",
                            "description": "Get budget data for January 2024"
                        }
                    ]
                },
                "user": {
                    "description": "User profile information",
                    "uri_patterns": ["ynab://user"],
                    "response_contains": ["user_id", "email", "subscription"]
                },
                "scheduled_transactions": {
                    "description": "Recurring transactions and scheduled transfers",
                    "uri_patterns": [
                        "ynab://budgets/{budget_id}/scheduled_transactions",
                        "ynab://budgets/{budget_id}/scheduled_transactions/{scheduled_transaction_id}"
                    ]
                }
            },
            "data_formats": {
                "monetary_values": {
                    "format": "milliunits (integers)",
                    "precision": "1000 milliunits = $1.00",
                    "examples": {
                        "positive_income": 250000,
                        "negative_expense": -150000,
                        "zero_balance": 0
                    }
                },
                "dates": {
                    "format": "YYYY-MM-DD or ISO 8601",
                    "timezone": "UTC or local timezone"
                },
                "cleared_states": {
                    "cleared": "Transaction has been confirmed/reconciled",
                    "uncleared": "Transaction is pending or unconfirmed",
                    "reconciled": "Transaction has been reconciled against bank statement"
                },
                "account_types": {
                    "on_budget": ["checking", "savings", "cash"],
                    "off_budget": ["otherAsset", "otherLiability"],
                    "debt": ["creditCard", "lineOfCredit"]
                }
            },
            "filtering_and_querying": {
                "date_filtering": {
                    "parameter": "since_date",
                    "format": "YYYY-MM-DD",
                    "description": "Get data modified since this date",
                    "example": "ynab://budgets/default/transactions?since_date=2024-01-01"
                },
                "transaction_types": {
                    "uncategorized": "Transactions without a category assigned",
                    "unapproved": "Transactions from bank imports awaiting approval"
                },
                "account_filtering": {
                    "on_budget": "Include only on-budget accounts",
                    "off_budget": "Include only off-budget (tracking) accounts"
                }
            },
            "caching_behavior": {
                "cache_ttl": {
                    "budgets": "30 minutes (structure changes infrequently)",
                    "accounts": "30 minutes",
                    "categories": "30 minutes",
                    "transactions": "2 minutes (changes frequently)",
                    "payees": "30 minutes",
                    "user": "60 minutes"
                },
                "cache_keys": "Based on URI and query parameters",
                "cache_invalidation": "Automatic expiration based on TTL"
            },
            "common_use_cases": {
                "budget_overview": {
                    "description": "Get complete budget picture",
                    "resources": [
                        "ynab://budgets/default",
                        "ynab://budgets/default/accounts",
                        "ynab://budgets/default/categories"
                    ]
                },
                "spending_analysis": {
                    "description": "Analyze spending by category or account",
                    "resources": [
                        "ynab://budgets/default/transactions",
                        "ynab://budgets/default/categories/{category_id}/transactions"
                    ]
                },
                "account_reconciliation": {
                    "description": "Match transactions with bank statements",
                    "resources": [
                        "ynab://budgets/default/accounts/{account_id}",
                        "ynab://budgets/default/accounts/{account_id}/transactions"
                    ]
                },
                "monthly_review": {
                    "description": "Review monthly budget performance",
                    "resources": [
                        "ynab://budgets/default/months/{month}",
                        "ynab://budgets/default/months/{month}/categories"
                    ]
                }
            },
            "troubleshooting": {
                "invalid_uri": {
                    "symptoms": "Resource not found errors",
                    "solutions": [
                        "Check URI format against patterns",
                        "Verify budget_id, account_id, etc. are valid UUIDs",
                        "Use 'default' for primary budget",
                        "List available resources first"
                    ]
                },
                "empty_responses": {
                    "symptoms": "No data returned",
                    "solutions": [
                        "Check date ranges are reasonable",
                        "Verify filters aren't too restrictive",
                        "Ensure budget has data for requested time period",
                        "Check account/category exists and has transactions"
                    ]
                },
                "performance_issues": {
                    "symptoms": "Slow responses or timeouts",
                    "solutions": [
                        "Use date ranges to limit large queries",
                        "Check cache_management stats",
                        "Monitor rate limiting with health_check",
                        "Consider using specific account/category filters"
                    ]
                }
            }
        })
    }

    /// Get MCP protocol compliance documentation  
    pub fn get_mcp_protocol_documentation() -> Value {
        json!({
            "protocol_version": "2024-11-05",
            "server_info": {
                "name": "ynab-mcp",
                "version": "0.1.0",
                "description": "YNAB Model Context Protocol Server",
                "author": "YNAB MCP Contributors",
                "homepage": "https://github.com/your-org/ynab-mcp"
            },
            "capabilities": {
                "tools": {
                    "supported": true,
                    "list_changed": false,
                    "count": 12
                },
                "resources": {
                    "supported": true,
                    "subscribe": false,
                    "list_changed": false
                },
                "prompts": {
                    "supported": true,
                    "list_changed": false,
                    "count": 5
                },
                "logging": {
                    "supported": false
                },
                "completion": {
                    "supported": false
                }
            },
            "supported_methods": [
                "initialize",
                "tools/list",
                "tools/call",
                "resources/list",
                "resources/read",
                "prompts/list",
                "prompts/get"
            ],
            "error_handling": {
                "supported_codes": {
                    "-32601": "Method not found",
                    "-32602": "Invalid params",
                    "-32603": "Internal error",
                    "-32001": "Authentication Error",
                    "-32002": "Rate Limit Exceeded",
                    "-32003": "Network Error",
                    "-32004": "Resource Not Found",
                    "-32005": "Permission Denied"
                },
                "error_format": {
                    "code": "Standard JSON-RPC error code",
                    "message": "Human-readable error description",
                    "data": {
                        "description": "Detailed error information",
                        "error_id": "Unique error identifier for troubleshooting",
                        "correlation_id": "Request correlation ID",
                        "timestamp": "ISO 8601 timestamp"
                    }
                }
            },
            "transport": {
                "method": "stdio",
                "format": "JSON-RPC 2.0",
                "encoding": "UTF-8"
            }
        })
    }

    /// Get comprehensive usage examples and tutorials
    pub fn get_usage_examples() -> Value {
        json!({
            "quick_start": {
                "description": "Common workflows for getting started",
                "examples": [
                    {
                        "title": "Basic Budget Overview",
                        "steps": [
                            {
                                "step": 1,
                                "action": "Get server capabilities",
                                "method": "initialize",
                                "expected_response": "Server info and capabilities"
                            },
                            {
                                "step": 2,
                                "action": "List available tools",
                                "method": "tools/list",
                                "expected_response": "12 available tools"
                            },
                            {
                                "step": 3,
                                "action": "Get budget list",
                                "method": "resources/read",
                                "params": {"uri": "ynab://budgets"},
                                "expected_response": "List of available budgets"
                            },
                            {
                                "step": 4,
                                "action": "Get budget details",
                                "method": "resources/read",
                                "params": {"uri": "ynab://budgets/default"},
                                "expected_response": "Complete budget information"
                            }
                        ]
                    },
                    {
                        "title": "Create and Track an Expense",
                        "steps": [
                            {
                                "step": 1,
                                "action": "Get available accounts",
                                "method": "resources/read",
                                "params": {"uri": "ynab://budgets/default/accounts"}
                            },
                            {
                                "step": 2,
                                "action": "Create expense transaction",
                                "method": "tools/call",
                                "params": {
                                    "name": "create_transaction",
                                    "arguments": {
                                        "budget_id": "default",
                                        "account_id": "checking-account-id",
                                        "amount": -2500,
                                        "date": "2024-01-15",
                                        "memo": "Lunch at cafe"
                                    }
                                }
                            },
                            {
                                "step": 3,
                                "action": "Verify transaction was created",
                                "method": "resources/read",
                                "params": {"uri": "ynab://budgets/default/transactions"}
                            }
                        ]
                    },
                    {
                        "title": "Monthly Spending Analysis",
                        "steps": [
                            {
                                "step": 1,
                                "action": "Analyze spending by category",
                                "method": "tools/call",
                                "params": {
                                    "name": "analyze_spending_by_category",
                                    "arguments": {
                                        "budget_id": "default",
                                        "months_back": 3
                                    }
                                }
                            },
                            {
                                "step": 2,
                                "action": "Get specific date range transactions",
                                "method": "tools/call",
                                "params": {
                                    "name": "get_transactions_by_date_range",
                                    "arguments": {
                                        "budget_id": "default",
                                        "from_date": "2024-01-01",
                                        "to_date": "2024-01-31"
                                    }
                                }
                            }
                        ]
                    }
                ]
            },
            "advanced_workflows": {
                "budget_optimization": {
                    "description": "Optimize budget allocations based on spending patterns",
                    "tools_used": ["analyze_spending_by_category", "set_category_budget"],
                    "workflow": [
                        "Analyze historical spending by category",
                        "Identify categories that are over/under budget",
                        "Adjust budget allocations accordingly",
                        "Monitor changes over time"
                    ]
                },
                "account_reconciliation": {
                    "description": "Reconcile accounts with bank statements",
                    "tools_used": ["get_all_account_transactions", "clear_transaction", "approve_transaction"],
                    "workflow": [
                        "Get all transactions for the account",
                        "Compare with bank statement",
                        "Approve pending transactions",
                        "Mark transactions as reconciled"
                    ]
                }
            },
            "ai_integration_tips": {
                "natural_language_processing": [
                    "Parse user intent for budget categories (e.g., 'food' -> grocery category)",
                    "Extract amounts from natural language (e.g., 'twenty-five dollars' -> 25000 milliunits)",
                    "Interpret relative dates (e.g., 'last month' -> calculate date range)"
                ],
                "error_recovery": [
                    "Handle rate limit errors with exponential backoff",
                    "Suggest alternative approaches when resources are not found",
                    "Provide helpful error messages with remediation steps"
                ],
                "performance_optimization": [
                    "Use caching to avoid repeated API calls",
                    "Batch related operations together",
                    "Use specific filters to limit data transfer"
                ]
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tools_documentation() {
        let docs = ApiDocumentation::get_tools_documentation();

        // Verify tools documentation structure
        assert!(docs.is_object());
        assert!(docs["version"].is_string());
        assert!(docs["description"].is_string());
        assert!(docs["categories"].is_object());

        // Verify categories structure
        let categories = &docs["categories"];
        assert!(categories["transaction_management"].is_object());
        assert!(categories["budget_management"].is_object());
    }

    #[test]
    fn test_get_resources_documentation() {
        let docs = ApiDocumentation::get_resources_documentation();

        // Basic structure validation
        assert!(docs.is_object());
        // The exact structure may vary, just verify it returns valid JSON
    }

    #[test]
    fn test_get_mcp_protocol_documentation() {
        let docs = ApiDocumentation::get_mcp_protocol_documentation();

        // Basic structure validation
        assert!(docs.is_object());
        assert!(docs["protocol_version"].is_string());
        // The exact structure may vary, just verify it returns valid JSON
    }

    #[test]
    fn test_get_usage_examples() {
        let docs = ApiDocumentation::get_usage_examples();

        // Basic structure validation
        assert!(docs.is_object());
        // The exact structure may vary, just verify it returns valid JSON
    }

    #[test]
    fn test_api_documentation_struct() {
        // Test that ApiDocumentation is a unit struct and can be referenced
        let _api_docs = ApiDocumentation;

        // Test that we can call static methods
        let tools_docs = ApiDocumentation::get_tools_documentation();
        assert!(tools_docs.is_object());

        let resources_docs = ApiDocumentation::get_resources_documentation();
        assert!(resources_docs.is_object());

        let protocol_docs = ApiDocumentation::get_mcp_protocol_documentation();
        assert!(protocol_docs.is_object());

        let usage_docs = ApiDocumentation::get_usage_examples();
        assert!(usage_docs.is_object());
    }

    #[test]
    fn test_tools_documentation_completeness() {
        let docs = ApiDocumentation::get_tools_documentation();

        // Check that tool categories have meaningful content
        let categories = &docs["categories"];
        let transaction_mgmt = &categories["transaction_management"];
        assert!(transaction_mgmt["tools"].is_array());

        let tools = transaction_mgmt["tools"].as_array().unwrap();
        assert!(
            tools.len() > 2,
            "Should have multiple transaction management tools"
        );
    }

    #[test]
    fn test_documentation_json_serialization() {
        let tools_docs = ApiDocumentation::get_tools_documentation();
        let resources_docs = ApiDocumentation::get_resources_documentation();
        let protocol_docs = ApiDocumentation::get_mcp_protocol_documentation();
        let usage_docs = ApiDocumentation::get_usage_examples();

        // Test that all documentation can be serialized to JSON
        assert!(serde_json::to_string(&tools_docs).is_ok());
        assert!(serde_json::to_string(&resources_docs).is_ok());
        assert!(serde_json::to_string(&protocol_docs).is_ok());
        assert!(serde_json::to_string(&usage_docs).is_ok());
    }
}
