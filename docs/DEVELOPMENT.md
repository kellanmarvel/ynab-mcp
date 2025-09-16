# Development Guide

This guide covers development setup, testing, and contributing to the YNAB MCP Server.

## Prerequisites

- **Rust 1.82.0+** (MSRV)
- **Git**
- **YNAB API Token** (for integration tests)

## Development Setup

```bash
# Clone the repository
git clone https://github.com/cinnes/ynab-mcp.git
cd ynab-mcp

# Install dependencies
cargo build

# Run tests
cargo test --lib

# Run with development logging
RUST_LOG=debug cargo run
```

## Testing

### Unit Tests
```bash
# Run only unit tests (fast)
cargo test --lib
```

### Integration Tests
```bash
# Run all tests (requires YNAB API token)
YNAB_API_TOKEN="your-token" cargo test

# Run specific test suites
cargo test --test integration_test
cargo test --test mcp_protocol_compliance_test
cargo test --test security_token_test
```

### Test Coverage

We use `cargo-tarpaulin` for code coverage analysis:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html --output-dir coverage

# Generate XML report for CI
cargo tarpaulin --out xml --output-dir coverage

# Open coverage report
open coverage/tarpaulin-report.html
```

**Coverage Targets:**
- **Project Coverage:** ≥80%
- **Patch Coverage:** ≥75%
- **Current Coverage:** ~17% (needs improvement)

### Coverage Exclusions
The following files are excluded from coverage requirements:
- `src/main.rs` - Minimal entry point logic
- `tests/` - Test files themselves
- `examples/` - Example code

## 🔧 Development Workflow

### 1. Code Style
```bash
# Format code
cargo fmt

# Check linting
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. Security Checks
```bash
# Audit dependencies
cargo audit

# Check for hardcoded secrets
grep -r "ynab_.*=" src/ --include="*.rs"
```

### 3. Performance Testing
```bash
# Release build
cargo build --release

# Benchmark critical paths
cargo test --release -- --ignored
```

## 📊 CI/CD Pipeline

Our GitHub Actions pipeline includes:

### CI Workflow (`.github/workflows/ci.yml`)
- **Build**: Multiple OS (Ubuntu, Windows)
- **Test**: Unit tests with MSRV support
- **Lint**: Clippy and rustfmt checks
- **Coverage**: Automated coverage reporting to Codecov

### Security Workflow (`.github/workflows/security.yml`)
- **Dependency Audit**: Weekly vulnerability scanning
- **Secret Scanning**: Trivy-based secret detection
- **Financial Security**: Custom tests for financial data handling

### Release Workflow (`.github/workflows/release.yml`)
- **Multi-platform Builds**: Linux, macOS (Intel/ARM), Windows
- **Automated Releases**: GitHub releases with binaries
- **Crates.io Publishing**: Automated for stable releases

## 🐛 Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug cargo run
```

### Test with Mock Data
```bash
YNAB_API_TOKEN="test-token" YNAB_BLOCK_TEST_TOKENS="false" cargo test
```

### Docker Development
```bash
# Build development image
docker build -t ynab-mcp:dev .

# Run with environment
docker run -e YNAB_API_TOKEN="your-token" ynab-mcp:dev
```

## 📈 Performance Optimization

### Memory Profiling
```bash
# Use valgrind (Linux)
valgrind --tool=massif cargo run

# Use heaptrack (Linux)
heaptrack cargo run
```

### CPU Profiling
```bash
# Use perf (Linux)
perf record -g cargo run
perf report
```

## 🔐 Security Best Practices

1. **Never commit API tokens**
2. **Use the keyring for token storage**
3. **Validate all user inputs**
4. **Use `secrecy` crate for sensitive data**
5. **Implement proper error handling**

## 📝 Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Write** tests for new functionality
4. **Ensure** coverage doesn't decrease
5. **Run** the full test suite
6. **Submit** a pull request

### Pull Request Checklist
- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Coverage maintains target levels
- [ ] Documentation updated if needed
- [ ] Security considerations addressed

## 📚 Architecture Overview

```
src/
├── handlers/           # MCP request handlers
├── ynab/              # YNAB API client
├── schemas/           # MCP schema definitions
├── config.rs          # Configuration management
├── secrets.rs         # Secure token handling
└── main.rs           # Application entry point
```

### Key Design Principles
- **Security First**: All sensitive data is properly protected
- **Performance**: Async/await with efficient caching
- **Reliability**: Comprehensive error handling and recovery
- **Maintainability**: Modular design with clear separation of concerns