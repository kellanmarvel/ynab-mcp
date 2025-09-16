# AI Tool Guidance for YNAB MCP Server

This document provides guidance for AI tools working with the YNAB MCP (Model Context Protocol) server codebase.

## Project Overview

This is a Rust-based MCP server that provides AI assistants with access to YNAB (You Need A Budget) API functionality. The server implements the MCP protocol using stdio/JSON-RPC communication.

### Key Architecture Points

- **Communication**: Uses stdio/JSON-RPC, NOT HTTP
- **Protocol**: Implements MCP (Model Context Protocol) specification
- **Purpose**: Enables AI assistants to interact with YNAB budgets, transactions, accounts, etc.
- **Security**: Implements token validation, read-only mode, and secure credential storage

## Development Environment Setup

### Prerequisites
```bash
# Rust toolchain (MSRV: 1.82.0)
rustup install 1.82.0
rustup default stable

# Required for testing
cargo install cargo-tarpaulin  # For coverage reports
```

### Environment Variables
```bash
# Required for operation
YNAB_API_TOKEN="your-token-here"

# Optional configuration
READ_ONLY_MODE="true"           # Prevents modifications
YNAB_BLOCK_TEST_TOKENS="false" # Allows test tokens in development
LOG_LEVEL="info"                # debug, info, warn, error
```

## Common Development Tasks

### Running Tests
```bash
# Unit tests only (recommended for development)
cargo test --lib

# All tests (may require network/API access)
cargo test

# Specific test patterns
cargo test test_name_pattern
cargo test --test integration_test
```

### Code Coverage
```bash
# Generate coverage report
YNAB_API_TOKEN="test-token" YNAB_BLOCK_TEST_TOKENS="false" READ_ONLY_MODE="true" \
  cargo tarpaulin --lib --timeout 120 --exclude-files 'src/main.rs' --out xml --output-dir ./coverage
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check for compilation issues
cargo check --all-features
```

### Building and Running
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run with environment setup
YNAB_API_TOKEN="your-token" cargo run

# Test server responsiveness
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}' | cargo run
```

## Project Structure

```
src/
├── main.rs                 # Entry point (excluded from coverage)
├── config.rs              # Configuration management
├── server.rs              # Main MCP server implementation
├── health.rs              # Health check system
├── handlers/               # Request handlers
│   ├── budgets.rs         # Budget operations
│   ├── tools.rs           # Tool handling
│   ├── prompts.rs         # Prompt handling
│   ├── validation.rs      # Input validation
│   ├── transaction_handlers/  # Transaction operations
│   └── resource_handlers/     # Resource operations
├── schemas/               # MCP protocol schemas
├── ynab/                  # YNAB API client
├── utils/                 # Utility functions
└── secrets.rs             # Secure credential management

tests/                     # Integration tests
docs/                      # Documentation
.github/workflows/         # CI/CD configuration
```

## Key Code Patterns

### Error Handling
```rust
use anyhow::{Result, anyhow};

// Use anyhow::Result for most functions
pub async fn some_operation() -> Result<Value> {
    // Validate input
    let validated = validate_input(input)?;
    
    // Perform operation
    let result = api_call(validated).await
        .map_err(|e| anyhow!("API call failed: {}", e))?;
    
    Ok(result)
}
```

### Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_client() -> YnabClient {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
        let token = YnabApiToken::new("test-token".to_string());
        YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap()
    }
    
    #[tokio::test]
    async fn test_operation() {
        let client = create_test_client();
        // Test implementation
    }
}
```

### MCP Handler Pattern
```rust
impl McpServer {
    pub async fn handle_tool_call(&self, tool_name: &str, args: Value) -> Result<Value> {
        match tool_name {
            "get_budgets" => self.handle_budgets(args).await,
            "create_transaction" => self.handle_create_transaction(args).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name))
        }
    }
}
```

## Security Considerations

### Token Management
- NEVER log API tokens
- Use `SecretString` for sensitive data
- Implement `Debug` carefully to avoid leaking secrets
- Use `Zeroize` trait for secure memory cleanup

### Input Validation
- Always validate UUIDs using `uuid` crate
- Validate monetary amounts using `MoneyUtils`
- Sanitize all user inputs before API calls
- Check permissions in read-only mode

### API Rate Limiting
- Respect YNAB API rate limits (200 requests/hour)
- Implement exponential backoff for retries
- Cache responses when appropriate

## Testing Strategy

### Unit Tests (Preferred)
- Fast execution
- No network dependencies
- Test individual functions and modules
- Use `#[cfg(test)]` modules in source files

### Integration Tests
- Test complete workflows
- May require network access
- Located in `tests/` directory
- Use mocking for external dependencies

### Test Environment
```bash
# Set up test environment
export YNAB_API_TOKEN="test-token"
export YNAB_BLOCK_TEST_TOKENS="false"
export READ_ONLY_MODE="true"
```

## CI/CD Information

### GitHub Actions Workflow
- Runs on Ubuntu and Windows
- Tests with Rust stable and MSRV (1.82.0)
- Includes formatting, linting, building, and testing
- Generates code coverage reports with Codecov integration

### Coverage Expectations
- Current coverage: ~22.7%
- Target: Focus on critical paths and new features
- Excludes `src/main.rs` and integration tests from coverage

## Common Issues and Solutions

### Compilation Errors
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for version conflicts
cargo tree
```

### Test Failures
```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run tests with environment variables
RUST_BACKTRACE=1 cargo test

# Skip integration tests
cargo test --lib
```

### API Token Issues
```bash
# Check token validity
YNAB_API_TOKEN="your-token" cargo run -- status

# Use test mode
YNAB_BLOCK_TEST_TOKENS="false" YNAB_API_TOKEN="test-token" cargo test
```

## MCP Protocol Implementation

### Required Methods
- `initialize` - Setup protocol communication
- `list_tools` - Return available YNAB operations
- `call_tool` - Execute YNAB operations
- `list_resources` - Return available YNAB resources
- `read_resource` - Fetch YNAB data
- `list_prompts` - Return available prompt templates

### Tool Categories
- **Budget Operations**: Get budgets, assign category budgets
- **Transaction Operations**: Create, update, clear, approve transactions
- **Account Operations**: List accounts, get account details
- **Analysis Operations**: Generate spending analysis and insights

## Performance Considerations

- Use async/await for all I/O operations
- Implement connection pooling for HTTP requests
- Cache frequently accessed data (budgets, accounts)
- Use streaming for large datasets
- Implement timeouts for all network operations

## Contributing Guidelines

1. **Code Style**: Use `cargo fmt` and `cargo clippy`
2. **Testing**: Add tests for new functionality
3. **Documentation**: Update relevant docs and comments
4. **Security**: Review for credential leaks and input validation
5. **Performance**: Consider rate limits and caching
6. **Error Handling**: Use `anyhow` for error propagation

## Useful Commands Reference

```bash
# Development workflow
cargo check                    # Quick compilation check
cargo test --lib              # Run unit tests
cargo fmt --all               # Format code
cargo clippy --fix            # Auto-fix linting issues

# Release preparation
cargo build --release         # Optimized build
cargo audit                   # Security audit
cargo outdated                # Check for updates

# Debugging
RUST_LOG=debug cargo run      # Enable debug logging
RUST_BACKTRACE=1 cargo test   # Show backtraces
cargo expand                  # Show macro expansions
```

This guidance should help any AI tool understand and work effectively with the YNAB MCP server codebase.