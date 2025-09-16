# Development Guide

This guide covers the development setup and best practices for contributing to the YNAB MCP Server project.

## Prerequisites

- Rust 1.70+ 
- Git
- A YNAB Personal Access Token (for testing)

## Getting Started

1. **Clone the repository:**
   ```bash
   git clone https://github.com/cinnes/ynab-mcp.git
   cd ynab-mcp
   ```

2. **Install the pre-commit hook:**
   ```bash
   git config core.hooksPath .githooks
   ```

3. **Install dependencies:**
   ```bash
   cargo build
   ```

4. **Set up environment variables for testing:**
   ```bash
   export YNAB_API_TOKEN="your-personal-access-token"
   export READ_ONLY_MODE="true"  # Recommended for development
   ```

## Code Quality & Pre-commit Hooks

This project enforces strict code quality standards through automated pre-commit hooks.

### Pre-commit Hook Setup

To install the pre-commit hook:

```bash
git config core.hooksPath .githooks
```

The hook runs the following checks before every commit:

1. **Formatting Check (`cargo fmt`)** - Ensures consistent code formatting
2. **Linting (`cargo clippy`)** - Catches common mistakes and enforces best practices  
3. **Unit Tests (`cargo test`)** - Verifies all tests pass

### What the Pre-commit Hook Does

- ✅ **Prevents commits** if any check fails
- 🎨 **Enforces consistent formatting** with rustfmt
- 🔍 **Catches code issues** with clippy (treats warnings as errors)
- 🧪 **Ensures tests pass** before code reaches the repository
- 🌈 **Provides colorized output** for easy debugging

### Example Output

```bash
🔍 Running pre-commit checks...

🦀 Running Rust quality checks...
Running rustfmt formatting check...
✅ rustfmt formatting check passed
Running clippy lints...
✅ clippy lints passed  
Running unit tests...
✅ unit tests passed

🎉 All pre-commit checks passed! Proceeding with commit...
```

### Manual Quality Checks

You can run these checks manually at any time:

```bash
# Format code
cargo fmt

# Check formatting (without fixing)
cargo fmt -- --check

# Run clippy lints
cargo clippy

# Run clippy with stricter settings (same as pre-commit hook)
cargo clippy -- -D warnings

# Run tests
YNAB_API_TOKEN="test-token" YNAB_BLOCK_TEST_TOKENS="false" READ_ONLY_MODE="true" cargo test --lib
```

### Bypassing the Pre-commit Hook

In exceptional circumstances, you can bypass the pre-commit hook:

```bash
git commit --no-verify -m "emergency fix"
```

**⚠️ Warning:** Only use `--no-verify` for genuine emergencies. The CI pipeline will still enforce these checks.

## Testing

### Test Environment

The project uses specific environment variables for testing:

- `YNAB_API_TOKEN="test-token"` - Uses a test token
- `YNAB_BLOCK_TEST_TOKENS="false"` - Allows test tokens  
- `READ_ONLY_MODE="true"` - Prevents actual API writes

### Running Tests

```bash
# Run all tests (recommended)
cargo test

# Run only unit tests
cargo test --lib

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Test Utilities

The project includes centralized test utilities in `src/test_utils.rs`:

- `create_test_common()` - Creates a configured CommonUtils for testing
- `create_mock_transaction()` - Creates realistic test transaction data
- `create_mock_account()` - Creates realistic test account data
- `setup_test_env()` / `cleanup_test_env()` - Environment management

## Code Organization

### Architecture Overview

- `src/handlers/` - Core business logic organized by functionality
- `src/ynab/` - YNAB API client and models
- `src/utils/` - Shared utilities (money handling, etc.)
- `src/test_utils.rs` - Centralized testing utilities

### Recent Improvements

The codebase has been significantly refactored to reduce code duplication:

- **TransactionUpdater** - Unified transaction update operations
- **Handler Base Pattern** - Macros to reduce boilerplate across handlers
- **Validation Pipelines** - Standardized input validation and processing
- **Test Utilities** - Centralized test setup and mock data creation

## Contributing

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the existing patterns

3. **Ensure all checks pass:**
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```

4. **Commit your changes** (pre-commit hook will run automatically):
   ```bash
   git add .
   git commit -m "feat: your feature description"
   ```

5. **Push and create a pull request:**
   ```bash
   git push origin feature/your-feature-name
   ```

## Continuous Integration

The project uses GitHub Actions for CI/CD:

- **Build & Test** - Runs on every push and pull request
- **Security Scan** - Checks for hardcoded secrets and vulnerabilities  
- **Code Coverage** - Tracks test coverage with Codecov

All CI checks must pass before merging pull requests.

## Common Issues

### Pre-commit Hook Fails

If the pre-commit hook fails:

1. **Read the error message** - it will tell you exactly what's wrong
2. **Fix formatting issues:** `cargo fmt`
3. **Fix clippy issues:** `cargo clippy --fix`
4. **Fix failing tests:** Debug with `cargo test -- --nocapture`
5. **Re-stage changes:** `git add .` 
6. **Try committing again:** `git commit -m "your message"`

### Test Failures

- Ensure you have the correct environment variables set
- Check that you're not using real API tokens in tests
- Verify that `READ_ONLY_MODE="true"` is set for safety

For more help, check the project's GitHub issues or create a new one.