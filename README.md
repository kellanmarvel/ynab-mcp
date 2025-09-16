# YNAB MCP Server

[![CI](https://github.com/cinnes/ynab-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/cinnes/ynab-mcp/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/cinnes/ynab-mcp/branch/master/graph/badge.svg)](https://codecov.io/gh/cinnes/ynab-mcp)
[![Security](https://github.com/cinnes/ynab-mcp/actions/workflows/security.yml/badge.svg)](https://github.com/cinnes/ynab-mcp/actions/workflows/security.yml)

A Model Context Protocol server that provides secure access to YNAB (You Need A Budget) data and operations through AI assistants.

This server enables natural language interactions with your YNAB budget, allowing AI assistants to create transactions, analyze spending patterns, and manage budget allocations through a standardized protocol interface.

## 🌟 Features

- **Transaction Management**: Create, update, and categorize transactions through natural language
- **Budget Analysis**: Query spending patterns, category performance, and financial trends
- **Account Operations**: Transfer funds between accounts and manage account balances
- **Secure Authentication**: OS keyring integration with encrypted token storage
- **Comprehensive Coverage**: 16+ tools covering all major YNAB operations
- **Read-only Mode**: Safe testing and analysis without data modification

## 📋 Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Available Tools](#available-tools)
- [Development](#development)
- [Documentation](#documentation)

## Installation

### Pre-built Binaries

Download the latest release for your platform:

```bash
# Linux/macOS (automatic detection)
curl -L https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m) -o ynab-mcp
chmod +x ynab-mcp

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-windows-x86_64.exe" -OutFile "ynab-mcp.exe"
```

### Docker

```bash
docker run --rm -i -e YNAB_API_TOKEN="your-token" ghcr.io/cinnes/ynab-mcp:latest
```

### Build from Source

```bash
git clone https://github.com/cinnes/ynab-mcp.git
cd ynab-mcp
cargo build --release
```

## Configuration

### 🔐 Authentication Setup

1. **Obtain YNAB API Token**
   - Visit [YNAB Developer Settings](https://app.youneedabudget.com/settings/developer)
   - Create a new token with description "MCP Server"
   - Copy the token immediately (not shown again)

2. **Secure Token Storage**
   ```bash
   ./ynab-mcp store-token
   # Enter token when prompted - stored in OS keyring
   
   # Verify configuration
   ./ynab-mcp health-check
   ```

### 🔧 MCP Client Configuration

Add the server to your MCP client configuration:

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

## Usage

### Transaction Operations

```
"I spent $45.67 at Whole Foods for groceries"
"Mark my $1,200 rent payment as cleared"
"Transfer $500 from checking to savings account"
```

### Budget Analysis

```
"How much have I spent on dining out this month?"
"Show me my biggest expense categories"
"What's my current account balance summary?"
```

### Budget Management

```
"Assign $300 to my vacation category"
"Am I overspending in any categories?"
"Generate a spending report for the last quarter"
```

## Available Tools

The server provides comprehensive YNAB integration through these tool categories:

### Core Operations
- **get_budgets**: List all available budgets
- **get_accounts**: Retrieve account information and balances
- **get_categories**: Access budget categories and allocations

### Transaction Management
- **create_transaction**: Add new transactions with automatic categorization
- **update_transaction**: Modify existing transaction details
- **clear_transaction**: Mark transactions as cleared/reconciled
- **approve_transaction**: Approve pending transactions

### Analysis Tools
- **analyze_transactions**: Generate spending analysis and trends
- **analyze_spending_by_category**: Category-specific spending breakdowns
- **get_transactions_by_date_range**: Query transactions within time periods

### Budget Operations
- **set_category_budget**: Set exact budget amounts for categories
- **create_transfer**: Transfer funds between accounts

### System Tools
- **health_check**: Verify API connectivity and system status
- **cache_management**: Monitor and control response caching

## Development

### Requirements
- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- YNAB account with API access

### Running Tests
```bash
# Run all tests
cargo test

# Run with test coverage
YNAB_API_TOKEN="test-token" YNAB_BLOCK_TEST_TOKENS="false" cargo test
```

### Environment Variables
- `YNAB_API_TOKEN`: API token (prefer keyring storage)
- `READ_ONLY_MODE`: Enable safe testing mode
- `LOG_LEVEL`: Set logging verbosity (debug, info, warn, error)
- `REQUEST_TIMEOUT_SECS`: HTTP request timeout (default: 30)

## Documentation

### Core Documentation
- [API Reference](./docs/API.md) - Complete tool and resource documentation
- [Configuration Guide](./docs/CONFIGURATION.md) - Advanced setup options
- [Features & Architecture](./docs/FEATURES.md) - Technical implementation details

### Examples
- [Basic Usage Guide](./examples/basic_usage.md) - Common workflow examples
- [API Examples](./examples/api_examples.md) - JSON-RPC request/response samples

### External Resources
- [Model Context Protocol](https://modelcontextprotocol.io) - Protocol specification
- [YNAB API Documentation](https://api.ynab.com) - Official API reference
- [Claude Desktop](https://claude.ai/desktop) - Compatible MCP client

## Security Considerations

- Tokens are stored securely in OS keyring (Linux: keyutils, macOS: Keychain, Windows: Credential Manager)
- All API communications use HTTPS with proper certificate validation
- Read-only mode available for safe testing and analysis
- Comprehensive input validation and sanitization
- Memory-safe implementation in Rust with automatic secret zeroization

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please read our [contributing guidelines](CONTRIBUTING.md) and submit pull requests to the main branch.

---

**Version**: 0.1.4 | **Language**: Rust | **Protocol**: MCP 2024-11-05