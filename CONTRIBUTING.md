# Contributing

Guidelines for contributing to the YNAB MCP Server.

## Prerequisites

- Rust 1.82.0+ (MSRV)
- Git
- YNAB API Token (for integration tests)

## Development Setup

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/ynab-mcp.git
cd ynab-mcp

# Build and test
cargo build
cargo test --lib
```

## Workflow

### 1. Create Branch
```bash
git checkout -b feature/your-feature-name
```

### 2. Development
- Follow existing code patterns
- Add tests for new functionality
- Update documentation as needed

### 3. Testing
```bash
# Format and lint
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings

# Unit tests
cargo test --lib

# Integration tests
YNAB_API_TOKEN="your-token" cargo test
```

### 4. Submit
```bash
git commit -m "feat: descriptive commit message"
git push origin feature/your-feature-name
```

Create pull request with clear description and reference related issues.

## Pull Request Requirements

- [ ] Tests pass locally (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated for public APIs
- [ ] No hardcoded secrets or credentials

## Bug Reports

Include the following information:

**Environment:**
- Rust version: `rustc --version`
- OS: Linux/macOS/Windows + version
- YNAB MCP version

**Issue:**
- Steps to reproduce
- Expected vs actual behavior
- Error messages
- Debug logs (`RUST_LOG=debug`)

## Code Guidelines

### Style
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` and address `cargo clippy` warnings
- Prefer explicit error handling over `unwrap()`

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        let input = setup_test_data();
        let result = function_under_test(input);
        assert_eq!(result, expected_output);
    }
}
```

### Security
- Never commit API tokens or secrets
- Use `secrecy` crate for sensitive data
- Implement `Zeroize` for credential cleanup
- Validate all user inputs
- Use OS keyring for token storage

### Documentation
- Document public APIs with examples
- Include error conditions
- Use `///` for documentation comments

## Testing

### Test Categories
- Unit tests: Individual functions/modules
- Integration tests: Component interactions
- E2E tests: Complete workflows

### Test Environment
```bash
# Test tokens
YNAB_API_TOKEN="test-token"
YNAB_BLOCK_TEST_TOKENS="false"
READ_ONLY_MODE="true"
```

## Resources

- Documentation: `/docs` directory
- Examples: `/examples` directory
- Issues: GitHub issue tracker