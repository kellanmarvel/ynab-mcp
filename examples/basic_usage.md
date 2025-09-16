# Usage Guide

This guide shows how to use the YNAB MCP server with AI assistants like Claude to manage your budget naturally through conversation.

## Quick Setup

### 1. Install and Configure

```bash
# Clone the repository
git clone https://github.com/cinnes/ynab-mcp.git
cd ynab-mcp

# Build the project
cargo build --release

# Store your YNAB token securely
./target/release/ynab-mcp store-token
# Enter your token from: https://app.youneedabudget.com/settings/developer

# Test connection
./target/release/ynab-mcp health-check
```

### 2. Configure Your AI Client

Add the server to your MCP client configuration. Examples:

**Claude Desktop** (`~/.config/claude-desktop/config.json`):
```json
{
  "mcpServers": {
    "ynab": {
      "command": "/absolute/path/to/ynab-mcp"
    }
  }
}
```

**VS Code with Continue** (`.continue/config.json`):
```json
{
  "mcpServers": {
    "ynab": {
      "command": "/absolute/path/to/ynab-mcp"
    }
  }
}
```

**Other MCP clients**: Use the absolute path to your built binary as the command.

### 3. Start Using

Launch your AI client and start talking to your budget!

## Natural Language Examples

### Basic Expense Entry

**You:** "I just spent £12.50 at Costa"

**AI Assistant will:**
1. Ask which account the expense came from
2. Suggest the "Dining Out" or "Coffee" category
3. Use the `create_transaction` tool to add the expense
4. Confirm the transaction was created successfully

**Behind the scenes:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_transaction", 
    "arguments": {
      "budget_id": "default",
      "account_id": "a1b2c3d4-5e6f-7890-abcd-ef1234567890",
      "amount": "-12.50",
      "date": "2025-09-15",
      "memo": "Costa coffee",
      "category_id": "dining-out-uuid",
      "cleared": true
    }
  }
}
```

### Budget Analysis

**You:** "How did I do with my restaurant spending last month?"

**AI Assistant will:**
1. Use `analyze_spending_by_category` tool to get category data
2. Compare actual vs. budgeted amounts
3. Show trends and patterns  
4. Provide insights and recommendations

**Example response:**
- "You spent £245 on dining out vs. your £200 budget (22% over)"
- "That's up 15% from the previous month"
- "Your biggest expenses were: Nando's (£35), Wagamama (£28)"

### Account Management

**You:** "Move £500 from my checking to my emergency fund"

**AI Assistant will:**
1. Confirm the transfer details
2. Use `create_transfer` tool to move the money
3. Show updated account balances
4. Provide confirmation of the successful transfer

### Financial Health Check

**You:** "How's my budget looking overall?"

**AI Assistant will:**
1. Use `check_budget_alerts` to identify overspending
2. Use `analyze_transactions` for spending trends
3. Check account balances and cash flow
4. Provide a comprehensive financial health summary

## Advanced Scenarios

### Split Transactions

**You:** "I spent £85 at Tesco, but £20 was for household items and £65 was groceries"

**AI Assistant will:**
1. Create the main grocery transaction for £65
2. Create a separate transaction for £20 household items
3. Ensure both transactions are properly categorized
4. Confirm the total matches your receipt

### Budget Planning

**You:** "Help me set up my budget for next month based on last month's spending"

**AI Assistant will:**
1. Use `generate_spending_report` to analyze previous month
2. Identify spending patterns and trends
3. Use `set_category_budget` to allocate money to categories
4. Provide recommendations for budget adjustments

### Transaction Cleanup

**You:** "I have some duplicate transactions that need cleaning up"

**AI Assistant will:**
1. Help you identify potential duplicates
2. Use resource queries to find similar transactions
3. Guide you through using `update_transaction` or deletion
4. Verify the cleanup was successful

## Resource Queries

Resources provide read-only access to your YNAB data:

### Budget Overview
**Request:** Access to `ynab://budgets/default`
**Returns:** Complete budget information, accounts, categories, and summary

### Recent Transactions  
**Request:** Access to `ynab://budgets/default/transactions`
**Returns:** Your most recent transactions across all accounts

### Account Details
**Request:** Access to `ynab://budgets/default/accounts/a1b2c3d4-5e6f-7890-abcd-ef1234567890`
**Returns:** Specific account information, balance, and recent activity

### Category Analysis
**Request:** Access to `ynab://budgets/default/categories/category-uuid`
**Returns:** Category budget vs. actual spending, transaction history

## Performance Features

### Smart Caching

The server automatically caches API responses to improve performance:

**You:** "Show me my account balances" (first request - hits YNAB API)
**You:** "What's my checking account balance?" (second request - served from cache)

Cache statistics:
```bash
# Check cache performance
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "cache_management", "arguments": {"action": "stats"}}, "id": 1}' | ./ynab-mcp
```

### Health Monitoring

**You:** "Is everything working properly?"

**AI Assistant will:**
1. Use `health_check` tool to test API connectivity
2. Check response times and rate limit status
3. Validate token permissions
4. Report overall system health

## Security Examples

### Token Management

```bash
# Check current token status
./ynab-mcp status

# Store a new token securely  
./ynab-mcp store-token

# Remove stored token
./ynab-mcp delete-token
```

### Read-Only Mode Testing

```bash
# Enable safe testing mode
export READ_ONLY_MODE=true
./ynab-mcp
```

**You:** "Add a £1000 test transaction"

**AI Assistant will:**
- Show exactly what transaction would be created
- Display "READ-ONLY MODE: Transaction not actually created"
- Explain the impact on your budget without making changes

## Conversation Patterns

### Natural Language Flexibility

These all work the same way:

- "Add a coffee expense for £4.50"
- "I bought coffee this morning, cost me four pounds fifty"  
- "£4.50 expense at cafe today"
- "Coffee: -4.50 GBP"

### Budget Questions

- "How much have I spent on groceries?"
- "What's left in my entertainment budget?"
- "Am I over budget in any categories?"
- "Show me my biggest expenses this month"

### Account Operations

- "Transfer money to savings"
- "Move £200 from checking to my holiday fund"  
- "What's my current account balance?"
- "Show me recent transactions"

## Direct JSON-RPC Examples

For developers or advanced users:

### List Available Tools
```bash
echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | ./ynab-mcp
```

### Create Transaction  
```bash
echo '{
  "jsonrpc": "2.0", 
  "method": "tools/call",
  "params": {
    "name": "create_transaction",
    "arguments": {
      "budget_id": "default",
      "account_id": "a1b2c3d4-5e6f-7890-abcd-ef1234567890", 
      "amount": "-25.00",
      "date": "2025-09-15",
      "memo": "Lunch",
      "category_id": "12345678-1234-1234-1234-123456789012"
    }
  },
  "id": 1
}' | ./ynab-mcp
```

### Get Account Information
```bash
echo '{
  "jsonrpc": "2.0",
  "method": "resources/read", 
  "params": {
    "uri": "ynab://budgets/default/accounts"
  },
  "id": 1  
}' | ./ynab-mcp
```

## Troubleshooting

### Common Issues

**"No budgets found"**
- Ensure your YNAB token has access to at least one budget
- Check token permissions in YNAB settings

**"Account not found"**
- Use resource queries to get valid account IDs first
- Account UUIDs are different from account names

**"Rate limit exceeded"**
- YNAB allows 200 requests/hour per token
- Use cache management to optimize requests
- Wait for the rate limit window to reset

### Debug Mode

```bash
# Enable detailed logging
LOG_LEVEL=debug ./ynab-mcp

# See all JSON-RPC communication  
LOG_LEVEL=trace ./ynab-mcp
```

### Testing Connectivity

```bash
# Test YNAB API connection
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "health_check", "arguments": {}}, "id": 1}' | ./ynab-mcp
```

## Best Practices

### Be Specific
- Include amounts, dates, and categories when possible
- "£15 for lunch at Pret on Monday" is better than "lunch expense"

### Use Natural Language
- "I went grocery shopping and spent £67" works perfectly
- No need to learn specific commands or syntax

### Leverage AI Understanding
- "Split this £40 restaurant bill with my friend" 
- AI will create a £20 expense and may suggest creating a receivable

### Check Before Big Changes
- Use read-only mode for testing large operations
- Ask "what would happen if..." questions first

### Regular Maintenance  
- Periodically ask "any budget alerts?" 
- Use "show cache stats" to monitor performance
- Run health checks if something seems slow

## Advanced Tips

1. **Batch Operations**: "Add these three expenses: coffee £4, lunch £12, parking £5"

2. **Pattern Recognition**: AI learns your spending patterns and can suggest categories automatically

3. **Reconciliation Help**: "Help me reconcile my checking account" - AI will guide you through clearing transactions

4. **Budget Optimization**: "Where can I cut spending?" - Get AI-powered insights into your budget

5. **Goal Tracking**: "How much more do I need to save for my vacation?" - Track progress toward savings goals