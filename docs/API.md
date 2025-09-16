# API Reference

Complete reference documentation for all tools, resources, and capabilities of the YNAB MCP server.

## 🌟 Tools

Tools are actions that AI assistants can perform to modify or interact with your YNAB data.

### Transaction Management

#### `create_transaction`
Create a new income or expense transaction.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default" for primary budget
- `account_id` (string, required): Account UUID where transaction occurred
- `amount` (string|number, required): Transaction amount (positive for inflow, negative for outflow)
- `date` (string, required): Transaction date in YYYY-MM-DD format
- `memo` (string, optional): Transaction description/note
- `payee_id` (string, optional): Payee UUID
- `category_id` (string, optional): Category UUID for budgeting
- `cleared` (boolean, optional): Whether transaction is cleared (default: false)
- `flag_color` (string, optional): Flag color (red, orange, yellow, green, blue, purple)

**Example:**
```json
{
  "name": "create_transaction",
  "arguments": {
    "budget_id": "default",
    "account_id": "12345678-1234-1234-1234-123456789012",
    "amount": "-25.50",
    "date": "2024-09-14", 
    "memo": "Coffee and pastry",
    "category_id": "87654321-4321-4321-4321-210987654321",
    "cleared": true
  }
}
```

#### `update_transaction`
Modify an existing transaction.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `transaction_id` (string, required): UUID of transaction to update
- All other parameters from `create_transaction` (optional, only include fields to change)

#### `approve_transaction`
Approve a pending/imported transaction.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `transaction_id` (string, required): UUID of transaction to approve

#### `clear_transaction`
Change the cleared status of a transaction.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `transaction_id` (string, required): UUID of transaction to update
- `clear_state` (string, required): New status ("cleared", "uncleared", "reconciled")

### Account Operations

#### `create_transfer`
Move money between accounts.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `from_account_id` (string, required): Source account UUID
- `to_account_id` (string, required): Destination account UUID
- `amount` (string|number, required): Transfer amount (always positive)
- `date` (string, required): Transfer date in YYYY-MM-DD format
- `memo` (string, optional): Transfer description

### Budget Management

#### `set_category_budget`
Allocate money to a budget category.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `category_id` (string, required): Category UUID to assign budget to
- `budgeted_amount` (string|number, required): Amount to assign
- `month` (string, optional): Month in YYYY-MM-DD format (default: current month)

### Analysis Tools

#### `analyze_transactions`
Comprehensive transaction analysis with filtering.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `account_id` (string, optional): Filter by specific account
- `category_id` (string, optional): Filter by specific category
- `from_date` (string, optional): Start date filter (YYYY-MM-DD)
- `to_date` (string, optional): End date filter (YYYY-MM-DD)

#### `analyze_spending_by_category`
Category-based spending pattern analysis.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `months_back` (number, optional): Number of months to analyze (default: 12)

#### `generate_spending_report`
Generate comprehensive spending analysis report.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `months_back` (integer, optional): Number of months to analyze (default: 6)
- `include_transfers` (boolean, optional): Include transfer transactions (default: false)

#### `generate_category_trends`
Analyze spending trends by category over time.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `category_id` (string, optional): Specific category to analyze
- `time_period` (string, optional): Analysis period ("month", "quarter", "year") (default: "month")

### Data Access

#### `get_transactions_by_date_range`
Retrieve all transactions within a specific date range.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `from_date` (string, required): Start date (YYYY-MM-DD)
- `to_date` (string, required): End date (YYYY-MM-DD)

#### `get_all_account_transactions`
Get all transactions for a specific account.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"
- `account_id` (string, required): Account UUID to get transactions for

### System Operations

#### `check_budget_alerts`
Check for budget overspending and alert conditions.

**Parameters:**
- `budget_id` (string, required): Budget ID or "default"

#### `health_check`
Check system health and YNAB API connectivity.

**Parameters:** None

**Returns:**
- API connectivity status
- Response time metrics
- Rate limit status  
- Token validation results
- System recommendations

#### `cache_management`
Manage API response cache for performance optimization.

**Parameters:**
- `action` (string, required): Action to perform ("stats", "cleanup", "clear")

## Resources

Resources provide read-only access to your YNAB data through URI-based requests.

### Budget Resources

#### `ynab://budgets`
List all available budgets.

#### `ynab://budgets/{budget_id}`
Get detailed information about a specific budget.

#### `ynab://budgets/{budget_id}/accounts`
Get all accounts in a budget with balances and summary statistics.

#### `ynab://budgets/{budget_id}/accounts/{account_id}`
Get detailed information about a specific account.

#### `ynab://budgets/{budget_id}/categories`
Get all budget categories organized by groups.

#### `ynab://budgets/{budget_id}/categories/{category_id}`
Get detailed information about a specific category.

### Transaction Resources

#### `ynab://budgets/{budget_id}/transactions`
Get recent transactions across all accounts.

#### `ynab://budgets/{budget_id}/accounts/{account_id}/transactions`
Get transactions for a specific account.

#### `ynab://budgets/{budget_id}/categories/{category_id}/transactions`
Get transactions for a specific category.

### Monthly Data Resources

#### `ynab://budgets/{budget_id}/months/{month}`
Get budget data for a specific month.

**Parameters:**
- `month`: Date in YYYY-MM-DD format (e.g., "2024-09-01")

## Prompts

Prompts provide structured, interactive workflows for common financial tasks.

### `quick_expense`
Interactive prompt for adding expenses quickly.

### `budget_review`
Comprehensive budget analysis for a specified period.

### `spending_analysis`
Deep dive into spending patterns and trends.

### `account_reconciliation`
Step-by-step account reconciliation guidance.

### `budget_planning`
Interactive budget planning for upcoming months.

## Authentication & Security

### Token Management

The server uses secure token management with multiple storage options:

**OS Keyring (Recommended):**
- Linux: Secret Service (libsecret/gnome-keyring)
- macOS: Keychain
- Windows: Credential Manager

**Environment Variable (Fallback):**
```bash
export YNAB_API_TOKEN="your_token_here"
```

### Commands

```bash
# Store token securely
./ynab-mcp store-token

# Check token status
./ynab-mcp status

# Remove stored token  
./ynab-mcp delete-token
```

## Configuration

### Environment Variables

```bash
# Optional: YNAB API base URL
YNAB_BASE_URL="https://api.ynab.com/v1"

# Optional: Logging level
LOG_LEVEL="info"  # debug, info, warn, error

# Optional: Read-only mode  
READ_ONLY_MODE="false"  # true to prevent writes

# Optional: Request timeouts
REQUEST_TIMEOUT_SECS="30"
CONNECT_TIMEOUT_SECS="10"
```

### CLI Options

```bash
# Disable keyring (use env vars only)
./ynab-mcp --no-keyring

# Require secure storage (fail if keyring unavailable)
./ynab-mcp --require-secure-storage

# Disable auto-migration from env vars
./ynab-mcp --no-auto-migrate
```

## Error Handling

### Error Codes

The server returns structured error codes:

- `-32001`: Authentication Error (invalid token)
- `-32002`: Rate Limit Exceeded (too many requests)
- `-32003`: Network Error (connectivity issues)
- `-32004`: Resource Not Found (invalid IDs)
- `-32005`: Permission Denied (insufficient access)
- `-32602`: Invalid Parameters (validation errors)
- `-32603`: Internal Error (server issues)

### Error Response Format

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Authentication Error",
    "data": {
      "description": "Invalid YNAB API token",
      "error_id": "uuid-here",
      "correlation_id": "request-uuid", 
      "timestamp": "2024-09-14T10:30:00Z"
    }
  },
  "id": 1
}
```

## Rate Limits & Performance

### YNAB API Limits
- **Rate Limit**: 200 requests per hour per token
- **Reset**: Rate limit window resets every hour

### Caching Strategy
- **Account Data**: 5 minutes TTL
- **Transaction Data**: 2 minutes TTL  
- **Budget Data**: 10 minutes TTL
- **Category Data**: 15 minutes TTL

### Performance Optimization
1. Use cache management tools to monitor performance
2. Batch related operations when possible
3. Use date range filters to limit data scope
4. Enable read-only mode for testing/exploration

## Testing & Development

### Development Mode

```bash
# Enable debug logging
LOG_LEVEL=debug ./ynab-mcp

# Enable read-only mode for safe testing
READ_ONLY_MODE=true ./ynab-mcp

# Test specific functionality
echo '{"jsonrpc": "2.0", "method": "health", "id": 1}' | ./ynab-mcp
```

### Example Test Commands

```bash
# List all available tools
echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | ./ynab-mcp

# Test health check
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "health_check", "arguments": {}}, "id": 1}' | ./ynab-mcp

# Get account information
echo '{"jsonrpc": "2.0", "method": "resources/read", "params": {"uri": "ynab://budgets/default/accounts"}, "id": 1}' | ./ynab-mcp
```