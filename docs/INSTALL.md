# Installation Guide

Quick installation guide to get the YNAB MCP server running with Claude Desktop.

## Quick Start

### 1. Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **YNAB Account**: With API access enabled
- **YNAB Token**: Get from [YNAB Developer Settings](https://app.youneedabudget.com/settings/developer)
- **Claude Desktop**: Download from [claude.ai/desktop](https://claude.ai/desktop)

### 2. Install Server

```bash
# Clone the repository
git clone <repository-url>
cd ynab-mcp

# Build optimized release version
cargo build --release
```

### 3. Setup Token (Secure)

**All Platforms (Recommended):**
```bash
# Interactive token storage
cargo run --release -- store-token

# Verify it was stored
cargo run --release -- status
```

**macOS (using Keychain Access):**
```bash
# Store token in keychain
security add-generic-password -s "ynab-mcp" -a "api-token" -w "your_ynab_api_token_here"

# Verify it was stored
security find-generic-password -s "ynab-mcp" -a "api-token" -w
```

**Windows (using Windows Credential Manager):**
```powershell
# The server will automatically use Windows Credential Manager
# Store manually via Control Panel > Credential Manager, or the token will auto-migrate from environment variables
```

### 4. Configure Claude Desktop

Add to your Claude Desktop configuration file:

**Linux/macOS**: `~/.config/claude-desktop/config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "ynab-mcp": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/absolute/path/to/ynab-mcp"
    }
  }
}
```

**Important**: Use the absolute path to your ynab-mcp directory.

### 5. Test Connection

```bash
# Test the server starts correctly
cargo run --release

# Should show:
# ✅ Starting YNAB MCP Server
# ✅ Health check passed - YNAB API is accessible  
# ✅ Server is ready and listening for requests
```

### 6. Start Using!

Launch Claude Desktop and try:
- "Show me my account balances"
- "I spent £15 at Costa today"
- "How much have I spent on groceries this month?"

---

## 🛠 Alternative Installation Methods

### Option A: Pre-built Binary

Download the pre-built binary for your platform:

**Automatic platform detection (Linux/macOS):**
```bash
curl -L https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m) -o ynab-mcp
chmod +x ynab-mcp
```

**Manual downloads:**
- [Linux x64](https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-linux-x86_64)
- [Linux ARM64](https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-linux-aarch64)
- [macOS Intel](https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-macos-x86_64)
- [macOS Apple Silicon](https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-macos-aarch64)
- [Windows x64](https://github.com/cinnes/ynab-mcp/releases/latest/download/ynab-mcp-windows-x86_64.exe)

**Configure Claude Desktop:**
```json
{
  "mcpServers": {
    "ynab-mcp": {
      "command": "/absolute/path/to/ynab-mcp"
    }
  }
}
```

### Option B: Environment Variable (Less Secure)

If keyring is not available:

```bash
# Set environment variable
export YNAB_API_TOKEN="your_token_here"

# Configure Claude Desktop with environment
{
  "mcpServers": {
    "ynab-mcp": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/path/to/ynab-mcp",
      "env": {
        "YNAB_API_TOKEN": "your_token_here"
      }
    }
  }
}
```

---

## 🔧 Platform-Specific Instructions

### Linux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install keyring support (required for secure storage)
# No additional dependencies needed for keyring support

# Build and setup
cargo build --release
cargo run --release -- store-token
```

**Claude Desktop Config**: `~/.config/claude-desktop/config.json`

### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Keyring support is built-in (uses Keychain)
cargo build --release
security add-generic-password -s "ynab-mcp" -a "api-token" -w "your_token"
```

**Claude Desktop Config**: `~/.config/claude-desktop/config.json`

### Windows

```powershell
# Install Rust (use rustup-init.exe from rustup.rs)
# Or use winget:
winget install Rustlang.Rustup

# Build and setup
cargo build --release
# Token can be stored via Control Panel > Credential Manager
# Or use environment variable and let the server auto-migrate
$env:YNAB_API_TOKEN="your_token"
```

**Claude Desktop Config**: `%APPDATA%\Claude\claude_desktop_config.json`

---

## 🔒 Token Security Setup

### Get Your YNAB Token

1. Go to [YNAB Developer Settings](https://app.youneedabudget.com/settings/developer)
2. Click "New Token"
3. Enter a description (e.g., "Claude MCP Server")
4. Copy the generated token immediately (you won't see it again)

### Secure Storage Options

**Option 1: Keyring (Recommended)**

*Linux:*
```bash
cargo run --release -- store-token
```

*macOS:*
```bash
security add-generic-password -s "ynab-mcp" -a "api-token" -w "your_token_here"
```

*Windows:*
```powershell
# Use Control Panel > Credential Manager or environment variable with auto-migration
```

**Option 2: Environment Variable**
```bash
export YNAB_API_TOKEN="your_token_here"
```

**Option 3: .env File**
```bash
echo "YNAB_API_TOKEN=your_token_here" > .env
```

### Verify Token Security

*Linux:*
```bash
cargo run --release -- status
```

*macOS:*
```bash
security find-generic-password -s "ynab-mcp" -a "api-token" -w
```

*All platforms:*
Restart Claude Desktop to ensure the MCP server reads the token from keyring.

---

## ✅ Verification Checklist

After installation, verify everything works:

- [ ] **Build Success**: `cargo build --release` completes without errors
- [ ] **Token Stored**: `cargo run --release -- status` shows token is valid
- [ ] **Server Starts**: `cargo run --release` shows "Server is ready"  
- [ ] **Health Check**: Server reports "YNAB API is accessible"
- [ ] **Claude Config**: JSON file is valid and uses absolute paths
- [ ] **Claude Connection**: Claude Desktop shows YNAB tools available

### Test Commands

```bash
# 1. Test server startup
cargo run --release

# 2. Test tools are available
echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | cargo run --release

# 3. Test YNAB connection  
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "health_check", "arguments": {}}, "id": 1}' | cargo run --release
```

---

## 🚨 Troubleshooting

### "Health check failed"
- **Check token**: Run `cargo run -- status` to verify token
- **Check network**: Ensure you can reach `api.ynab.com`
- **Check permissions**: Token needs read/write access to your budget

### "Keyring unavailable"
- **Linux**: Uses kernel keyring (keyutils) - no additional setup required
- **macOS**: Keyring should work by default via Keychain Access
- **Windows**: Should work by default via Credential Manager, try environment variable if issues

### "Command not found" in Claude
- **Path issue**: Ensure absolute path in Claude config
- **Build issue**: Verify `cargo build --release` completed successfully
- **Permission issue**: Ensure binary is executable

### "Rate limit exceeded"
- **Wait**: YNAB allows 200 requests/hour, wait for reset
- **Optimize**: Use `cache_management` tool to check cache efficiency
- **Monitor**: Check logs with `LOG_LEVEL=debug`

### Claude Desktop Issues

**Config file not found:**
- Create the directory: `mkdir -p ~/.config/claude-desktop/`
- Create empty config: `echo '{"mcpServers": {}}' > ~/.config/claude-desktop/config.json`

**Server not starting:**
- Check absolute paths in config
- Verify binary exists and is executable
- Check Claude Desktop logs for error messages

---

## 📞 Getting Help

### Debug Information

If you need help, gather this information:

```bash
# System information
rustc --version
cargo --version
uname -a  # Linux/macOS
systeminfo  # Windows

# Check token storage
# Linux:
cargo run --release -- status
# macOS:
security find-generic-password -s "ynab-mcp" -a "api-token" -w

# Server logs
RUST_LOG=debug cargo run --release 2>&1 | head -20
```

### Common Solutions

1. **Use absolute paths** in Claude Desktop config
2. **Restart Claude Desktop** after config changes  
3. **Check token permissions** in YNAB settings
4. **Try environment variable** if keyring issues
5. **Enable debug logging** to see detailed errors

### Support Channels

- **GitHub Issues**: Report bugs and request features
- **Documentation**: Check `README.md` and `docs/API.md`
- **YNAB Support**: For YNAB-specific token/API issues

---

## 🎉 Success!

If everything is working, you should be able to talk to your budget naturally:

**You:** "I spent £12 on coffee this morning"  
**Claude:** "I'll add that expense to your budget. Which account should I use?"

**You:** "How much have I spent on restaurants this month?"  
**Claude:** "Let me check your dining expenses..." *(shows detailed breakdown)*

Welcome to AI-powered budgeting! 🚀