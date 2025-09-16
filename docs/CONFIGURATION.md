# Configuration Guide

## Environment Variables

### Required Configuration
```bash
# Your YNAB Personal Access Token
export YNAB_API_TOKEN="your-token-here"
```

### Optional Settings
```bash
# YNAB API base URL (default: https://api.ynab.com/v1)
export YNAB_BASE_URL="https://api.ynab.com/v1"

# Logging level (default: info)  
export LOG_LEVEL="debug"

# Read-only mode for safe testing (default: false)
export READ_ONLY_MODE="true"

# Request timeout in seconds (default: 30)
export REQUEST_TIMEOUT_SECS="45"

# Connection timeout in seconds (default: 10)
export CONNECT_TIMEOUT_SECS="15"

# Block test tokens in production (default: true)
export YNAB_BLOCK_TEST_TOKENS="false"
```

## Secure Token Storage

### OS Keyring Integration

Store your YNAB token securely using the built-in token storage:

```bash
# Run the interactive token storage command
./ynab-mcp store-token
```

This stores your token securely in the OS-native keyring:
- **Linux**: Uses kernel keyring (keyutils) with persistent storage
- **macOS**: Uses macOS Keychain 
- **Windows**: Uses Windows Credential Manager

### Token Management Commands

```bash
# Store token securely
./ynab-mcp store-token

# Check token status
./ynab-mcp status

# Remove stored token
./ynab-mcp delete-token
```

### Advanced: Manual Keyring Storage

**macOS (using Keychain):**
```bash
security add-generic-password -s "ynab-mcp" -a "api-token" -w "your_token"
```

**Windows (using Credential Manager):**
- Use Control Panel > Credential Manager to store credentials
- Or use environment variables with auto-migration enabled

### Security Guidelines
- Use OS keyring when possible
- Avoid environment variables in production
- Rotate tokens regularly
- Never commit tokens to version control

## Read-Only Mode

Enable safe testing without modifying your budget:

```bash
export READ_ONLY_MODE="true"
./ynab-mcp
```

In read-only mode:
- All tools work but won't modify YNAB data
- Perfect for testing AI integrations
- Shows what would happen without executing changes

## MCP Client Configuration

### Claude Desktop

Configuration file locations:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "ynab": {
      "command": "/absolute/path/to/ynab-mcp"
    }
  }
}
```

### VS Code with Continue

Add to `.continue/config.json` in your workspace:

```json
{
  "mcpServers": {
    "ynab": {
      "command": "/absolute/path/to/ynab-mcp"
    }
  }
}
```

## Performance Configuration

### Caching Configuration
- Smart TTL based on data type
- Memory-efficient LRU cache
- Built-in cache management tools

### Rate Limit Management  
- YNAB API rate limit error handling
- Exponential backoff for retries
- No artificial delays (requests execute immediately)

### Network Optimization
```bash
# Adjust timeouts for your network conditions
export REQUEST_TIMEOUT_SECS="45"
export CONNECT_TIMEOUT_SECS="15"
```

## Docker Configuration

See [Docker Documentation](./DOCKER.md) for containerized deployment options.

## Troubleshooting

### Authentication Issues

```bash
# Verify token validity
curl -H "Authorization: Bearer your-token" https://api.ynab.com/v1/user

# Check token status
./ynab-mcp status

# Check if token is in keyring (macOS)
security find-generic-password -s "ynab-mcp" -a "api-token" -w
```

### Keyring Issues

```bash
# Linux: Ensure secret service is running
systemctl --user status gnome-keyring-daemon

# Restart Claude Desktop after storing token in keyring
```

### Rate Limit Errors

```bash
# YNAB enforces 200 requests/hour - check health status
# Use cache_management tool to optimize requests
export LOG_LEVEL="debug"
```

### Connection Issues

```bash
# Test network connectivity
export REQUEST_TIMEOUT_SECS="60"
export CONNECT_TIMEOUT_SECS="30"
```

### Debug Mode

Enable verbose logging:
```bash
export RUST_LOG="ynab_mcp=debug"
export LOG_LEVEL="debug"
```