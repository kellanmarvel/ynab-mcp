# API Examples

This document provides detailed examples of using the YNAB MCP Server tools and resources with practical scenarios and complete JSON-RPC requests.

## Complete Workflow Examples

### Example 1: Daily Expense Tracking Workflow

This example shows a complete daily expense tracking session.

#### Scenario: Coffee Shop Visit

**Step 1: Create the expense transaction**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transaction",
    "arguments": {
      "budget_id": "default",
      "account_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "amount": "-4.50",
      "date": "2025-09-15",
      "payee_name": "Pret A Manger",
      "category_id": "12345678-1234-1234-1234-123456789012",
      "memo": "Morning latte",
      "cleared": "uncleared"
    }
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "✅ Successfully created transaction: Pret A Manger for -£4.50 from Checking Account on 2025-09-15"
      },
      {
        "type": "resource",
        "resource": {
          "uri": "ynab://response/data",
          "mimeType": "application/json",
          "text": "{\"success\": true, \"transaction_id\": \"tx-123\"}"
        }
      }
    ]
  },
  "id": 1
}
```

**Step 2: Check spending in dining category**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_spending_by_category",
    "arguments": {
      "budget_id": "default",
      "months_back": 1
    }
  },
  "id": 2
}
```

#### Scenario: Weekly Grocery Shopping

**Step 1: Create grocery transaction with receipt details**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transaction",
    "arguments": {
      "budget_id": "default",
      "account_id": "a1b2c3d4-5e6f-7890-abcd-ef1234567890",
      "amount": "-127.83",
      "date": "2025-09-15",
      "payee_name": "Tesco",
      "category_id": "groceries-uuid",
      "memo": "Weekly grocery run - organic produce, dairy, pantry staples",
      "cleared": "cleared"
    }
  },
  "id": 3
}
```

**Step 2: Check remaining grocery budget**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/categories/groceries-uuid"
  },
  "id": 4
}
```

### Example 2: Monthly Budget Analysis Workflow

#### Comprehensive Spending Analysis

**Step 1: Generate monthly spending report**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "generate_spending_report",
    "arguments": {
      "budget_id": "default",
      "months_back": 1,
      "include_transfers": false
    }
  },
  "id": 5
}
```

**Expected Response Structure:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "📊 Monthly Spending Report (September 2024)"
      },
      {
        "type": "resource",
        "resource": {
          "uri": "ynab://response/data",
          "mimeType": "application/json",
          "text": "{\"total_spent\": \"£2,847.32\", \"categories\": [{\"name\": \"Groceries\", \"budgeted\": \"£400\", \"activity\": \"£387.45\", \"remaining\": \"£12.55\"}]}"
        }
      }
    ]
  },
  "id": 5
}
```

**Step 2: Check for budget alerts**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "check_budget_alerts",
    "arguments": {
      "budget_id": "default"
    }
  },
  "id": 6
}
```

**Step 3: Analyze category trends**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "generate_category_trends",
    "arguments": {
      "budget_id": "default",
      "category_id": "dining-out-uuid",
      "time_period": "month"
    }
  },
  "id": 7
}
```

### Example 3: Transfer and Account Management

#### Moving Money Between Accounts

**Step 1: Check account balances**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/accounts"
  },
  "id": 8
}
```

**Step 2: Create transfer between accounts**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transfer",
    "arguments": {
      "budget_id": "default",
      "from_account_id": "checking-uuid",
      "to_account_id": "savings-uuid",
      "amount": "500.00",
      "date": "2025-09-15",
      "memo": "Emergency fund contribution"
    }
  },
  "id": 9
}
```

**Step 3: Verify transfer completion**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/accounts/checking-uuid"
  },
  "id": 10
}
```

### Example 4: Advanced Transaction Management

#### Bulk Transaction Creation

**Scenario: Entering multiple receipts from a day out**

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transactions_bulk",
    "arguments": {
      "budget_id": "default",
      "transactions": [
        {
          "account_id": "checking-uuid",
          "amount": "-12.50",
          "date": "2025-09-15",
          "payee_name": "Metro Card",
          "category_id": "transportation-uuid",
          "memo": "Subway rides"
        },
        {
          "account_id": "checking-uuid", 
          "amount": "-45.75",
          "date": "2025-09-15",
          "payee_name": "Museum of Art",
          "category_id": "entertainment-uuid",
          "memo": "Exhibition admission + gift shop"
        },
        {
          "account_id": "checking-uuid",
          "amount": "-28.90",
          "date": "2025-09-15",
          "payee_name": "Corner Bistro",
          "category_id": "dining-out-uuid",
          "memo": "Lunch after museum visit"
        }
      ]
    }
  },
  "id": 11
}
```

#### Transaction Updates and Corrections

**Step 1: Find transaction to update**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_transactions_by_date_range",
    "arguments": {
      "budget_id": "default",
      "from_date": "2025-09-15",
      "to_date": "2025-09-15"
    }
  },
  "id": 12
}
```

**Step 2: Update transaction details**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "update_transaction",
    "arguments": {
      "budget_id": "default",
      "transaction_id": "tx-uuid-found-above",
      "amount": "-32.90",
      "memo": "Lunch after museum visit - corrected amount",
      "category_id": "dining-out-uuid"
    }
  },
  "id": 13
}
```

**Step 3: Mark transaction as cleared**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "clear_transaction",
    "arguments": {
      "budget_id": "default",
      "transaction_id": "tx-uuid",
      "clear_state": "cleared"
    }
  },
  "id": 14
}
```

### Example 5: Budget Planning and Assignment

#### Monthly Budget Allocation

**Step 1: Analyze previous month's spending**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_spending_by_category",
    "arguments": {
      "budget_id": "default",
      "months_back": 1
    }
  },
  "id": 15
}
```

**Step 2: Assign budget to categories for next month**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "set_category_budget",
    "arguments": {
      "budget_id": "default",
      "category_id": "groceries-uuid",
      "budgeted_amount": "450.00",
      "month": "2024-10-01"
    }
  },
  "id": 16
}
```

**Step 3: Check total allocated vs. available**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/months/2024-10-01"
  },
  "id": 17
}
```

## 🔧 System Administration Examples

### Cache and Performance Management

**Check cache statistics**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "cache_management",
    "arguments": {
      "action": "stats"
    }
  },
  "id": 18
}
```

**Clear cache for fresh data**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "cache_management",
    "arguments": {
      "action": "clear"
    }
  },
  "id": 19
}
```

### Health Monitoring

**Comprehensive health check**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "health_check",
    "arguments": {}
  },
  "id": 20
}
```

**Expected Health Check Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "✅ YNAB API is healthy and responsive - Response time: 145ms"
      },
      {
        "type": "resource",
        "resource": {
          "uri": "ynab://response/data",
          "mimeType": "application/json",
          "text": "{\"api_healthy\": true, \"response_time_ms\": 145, \"rate_limited\": false, \"last_check\": 1694789234}"
        }
      }
    ]
  },
  "id": 20
}
```

## 📊 Resource Query Examples

### Budget and Account Information

**Get all budgets**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets"
  },
  "id": 21
}
```

**Get specific budget details**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default"
  },
  "id": 22
}
```

**Get account details with transactions**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/accounts/a1b2c3d4-5e6f-7890-abcd-ef1234567890/transactions"
  },
  "id": 23
}
```

### Category and Payee Information

**Get all categories**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/categories"
  },
  "id": 24
}
```

**Get payee information**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/payees"
  },
  "id": 25
}
```

**Get monthly budget data**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/months/2024-09-01"
  },
  "id": 26
}
```

## 🎛️ Advanced Analysis Examples

### Financial Health Analysis

**Deep dive into spending patterns**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_transactions",
    "arguments": {
      "budget_id": "default",
      "from_date": "2024-01-01",
      "to_date": "2025-09-15",
      "category_id": "dining-out-uuid"
    }
  },
  "id": 27
}
```

**Income vs. expense analysis**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_transactions_by_date_range",
    "arguments": {
      "budget_id": "default",
      "from_date": "2024-09-01",
      "to_date": "2024-09-30"
    }
  },
  "id": 28
}
```

### Goal Tracking and Progress

**Check savings goal progress**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/categories/emergency-fund-uuid"
  },
  "id": 29
}
```

**Track debt payoff progress**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_all_account_transactions",
    "arguments": {
      "budget_id": "default",
      "account_id": "credit-card-uuid"
    }
  },
  "id": 30
}
```

## 🚨 Error Handling Examples

### Common Error Scenarios

**Invalid UUID Error:**
```json
// Request with invalid account ID
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transaction",
    "arguments": {
      "budget_id": "default",
      "account_id": "invalid-uuid",
      "amount": "-10.00",
      "date": "2025-09-15"
    }
  },
  "id": 31
}

// Error Response
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Validation error: Invalid UUID format for parameter 'account_id'. Expected: valid UUID format. Please provide a valid account UUID."
  },
  "id": 31
}
```

**Rate Limit Error:**
```json
// Error Response when rate limited
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Rate limit exceeded. Next reset in 45 minutes. Please wait before making more requests or use cached data."
  },
  "id": 32
}
```

**Authentication Error:**
```json
// Error Response when token is invalid
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Authentication failed: Invalid or expired YNAB API token. Please check your token and try again."
  },
  "id": 33
}
```

## 📈 Performance Optimization Examples

### Efficient Data Fetching

**Use cache-friendly queries:**
```json
// First call - fetches from API and caches
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/accounts"
  },
  "id": 34
}

// Subsequent calls within cache window - served from cache
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ynab://budgets/default/accounts"
  },
  "id": 35
}
```

**Batch related operations:**
```json
// Instead of multiple single transaction calls, use bulk operations
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transactions_bulk",
    "arguments": {
      "budget_id": "default",
      "transactions": [
        // Multiple transactions here
      ]
    }
  },
  "id": 36
}
```

## 🔗 Integration Patterns

### Claude Desktop Integration

This shows how these API calls would be used within Claude Desktop conversations:

**User:** "I need to add my coffee purchase from this morning"

**Claude internally executes:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_transaction",
    "arguments": {
      "budget_id": "default",
      "account_id": "user-checking-account",
      "amount": "-4.75",
      "date": "2025-09-15",
      "payee_name": "Local Coffee Shop",
      "category_id": "dining-out-category",
      "memo": "Morning coffee"
    }
  },
  "id": 1
}
```

**AI responds:** "I've added your £4.75 coffee purchase to your Dining Out category. Your remaining dining budget for this month is £47.25."

### Custom Application Integration

For applications building on top of the YNAB MCP Server:

```javascript
// Example Node.js integration
const { spawn } = require('child_process');

const ynabMcp = spawn('./ynab-mcp');

const request = {
  jsonrpc: '2.0',
  method: 'tools/call',
  params: {
    name: 'health_check',
    arguments: {}
  },
  id: 1
};

ynabMcp.stdin.write(JSON.stringify(request) + '\n');

ynabMcp.stdout.on('data', (data) => {
  const response = JSON.parse(data.toString());
  console.log('YNAB API Health:', response.result);
});
```

## 🎓 Best Practices

### 1. Always Use Default Budget ID
Unless you have multiple budgets, use `"default"` as the budget_id.

### 2. Include Meaningful Memos
Add descriptive memos to help with future transaction identification.

### 3. Use Appropriate Categories
Ensure transactions are categorized correctly for accurate reporting.

### 4. Monitor Rate Limits
Check cache statistics regularly and avoid rapid successive API calls.

### 5. Handle Errors Gracefully
Always check for errors in responses and provide meaningful feedback.

### 6. Use Read-Only Mode for Testing
Set `READ_ONLY_MODE=true` when experimenting to prevent accidental data modification.

For more examples and patterns, see the [basic usage guide](./basic_usage.md) and [troubleshooting documentation](../docs/TROUBLESHOOTING.md).