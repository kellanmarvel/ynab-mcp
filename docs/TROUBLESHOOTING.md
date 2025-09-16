# Troubleshooting Guide

This guide covers common deployment and operational issues with detailed solutions.

## Quick Diagnosis

### Health Check Flow
```bash
# 1. Basic connectivity
./ynab-mcp health-check

# 2. Check configuration
./ynab-mcp status

# 3. Test MCP protocol
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | ./ynab-mcp

# 4. Test with debug logging
RUST_LOG=debug ./ynab-mcp
```

## Authentication & Token Issues

### Issue: "Authentication failed" or "Invalid token"

**Symptoms:**
- Health check fails with 401 Unauthorized
- Tools return authentication errors
- YNAB API calls fail

**Diagnosis:**
```bash
# Check if token is stored
./ynab-mcp status

# Test token manually
curl -H "Authorization: Bearer YOUR_TOKEN" https://api.ynab.com/v1/user

# Check keyring (macOS)
security find-generic-password -s "ynab-mcp" -a "api-token" -w

# Check keyring (Linux)
keyctl list @u | grep ynab
```

**Solutions:**

1. **Regenerate Token:**
   ```bash
   # Get new token from https://app.youneedabudget.com/settings/developer
   ./ynab-mcp store-token
   # Enter your new token when prompted
   ```

2. **Fix Keyring Issues (Linux):**
   ```bash
   # Install keyring support
   sudo apt-get install keyutils

   # Check if keyring is available
   keyctl describe @u

   # If keyring unavailable, use environment variable temporarily
   export YNAB_API_TOKEN="your_token"
   ```

3. **Fix Keyring Issues (macOS):**
   ```bash
   # Manually store in keychain
   security add-generic-password -s "ynab-mcp" -a "api-token" -w "your_token"

   # Check keychain access permissions
   security find-generic-password -s "ynab-mcp" -a "api-token"
   ```

4. **Windows Credential Manager:**
   - Open Control Panel > Credential Manager
   - Add Windows Credential with target "ynab-mcp"
   - Username: "api-token", Password: your YNAB token

### Issue: "Keyring not available" or "Secrets storage failed"

**Solutions:**

1. **Use Environment Variable Fallback:**
   ```bash
   export YNAB_API_TOKEN="your_token"
   export YNAB_MCP_USE_KEYRING=false
   ./ynab-mcp
   ```

2. **Enable Auto-Migration:**
   ```bash
   export YNAB_MCP_AUTO_MIGRATE=true
   ./ynab-mcp store-token
   ```

## 🌐 Network & Connectivity Issues

### Issue: "Connection timeout" or "Network unreachable"

**Symptoms:**
- Health check times out
- All YNAB API calls fail
- Network-related error messages

**Diagnosis:**
```bash
# Test YNAB API directly
curl -v https://api.ynab.com/v1/user

# Check DNS resolution
nslookup api.ynab.com

# Test with custom timeout
export REQUEST_TIMEOUT_SECS=60
export CONNECT_TIMEOUT_SECS=30
./ynab-mcp health-check
```

**Solutions:**

1. **Configure Proxy:**
   ```bash
   export HTTPS_PROXY=http://proxy.company.com:8080
   export HTTP_PROXY=http://proxy.company.com:8080
   ./ynab-mcp
   ```

2. **Adjust Timeouts:**
   ```bash
   # For slow networks
   export REQUEST_TIMEOUT_SECS=120
   export CONNECT_TIMEOUT_SECS=60
   ./ynab-mcp
   ```

3. **Corporate Firewall Issues:**
   ```bash
   # Test if YNAB API is blocked
   curl -I https://api.ynab.com

   # Contact IT to whitelist:
   # - api.ynab.com (HTTPS/443)
   # - api.youneedabudget.com (HTTPS/443)
   ```

### Issue: "Rate limit exceeded"

**Symptoms:**
- "429 Too Many Requests" errors
- Slow response times
- Intermittent failures

**Diagnosis:**
```bash
# Check cache statistics
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"cache_management","arguments":{"action":"stats"}},"id":1}' | ./ynab-mcp
```

**Solutions:**

1. **Optimize Cache Usage:**
   ```bash
   # Clear cache to reset counters
   echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"cache_management","arguments":{"action":"clear"}},"id":1}' | ./ynab-mcp

   # Check cache hit rate
   echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"cache_management","arguments":{"action":"stats"}},"id":1}' | ./ynab-mcp
   ```

2. **Reduce Request Frequency:**
   - YNAB API limit: 200 requests/hour
   - Use cached responses when possible
   - Avoid rapid successive calls

## 💻 Platform-Specific Issues

### Linux Issues

**Issue: "Permission denied" when running binary**
```bash
# Fix permissions
chmod +x ynab-mcp

# Check if file is executable
ls -la ynab-mcp

# Check if blocked by system policy
sudo ./ynab-mcp --version
```

**Issue: "Keyring not available"**
```bash
# Install keyring support
sudo apt-get install keyutils libsecret-1-dev

# For Ubuntu/Debian
sudo apt-get install gnome-keyring

# For RHEL/CentOS
sudo yum install keyutils
```

### macOS Issues

**Issue: "Cannot be opened because it is from an unidentified developer"**
```bash
# Allow the binary to run
sudo xattr -rd com.apple.quarantine ynab-mcp

# Or via System Preferences
# Security & Privacy > General > "Allow anyway"
```

**Issue: Keychain access denied**
```bash
# Reset keychain permissions
security delete-generic-password -s "ynab-mcp" -a "api-token"
./ynab-mcp store-token

# Check keychain access
security find-generic-password -s "ynab-mcp" -a "api-token"
```

### Windows Issues

**Issue: "Windows protected your PC" SmartScreen warning**
- Click "More info" → "Run anyway"
- Or download from releases page directly

**Issue: Credential Manager access**
```powershell
# Check stored credentials
cmdkey /list | findstr ynab

# Manually add credential
cmdkey /add:ynab-mcp /user:api-token /pass:your_token
```

## 🐳 Docker Deployment Issues

### Issue: Container fails to start

**Diagnosis:**
```bash
# Check container logs
docker logs ynab-mcp-container

# Run with debug
docker run -e RUST_LOG=debug -p 3000:3000 ynab-mcp

# Test container interactively
docker run -it ynab-mcp /bin/sh
```

**Common Solutions:**

1. **Port Binding Issues:**
   ```bash
   # Check if port is available
   netstat -tulpn | grep :3000

   # Use different port
   docker run -p 3001:3000 ynab-mcp
   ```

2. **Environment Variables:**
   ```bash
   # Pass token via environment
   docker run -e YNAB_API_TOKEN="your_token" ynab-mcp

   # Use env file
   docker run --env-file .env ynab-mcp
   ```

3. **Volume Mounting for Keyring:**
   ```bash
   # Linux keyring volume
   docker run -v /home/$USER/.local/share/keyrings:/home/app/.local/share/keyrings ynab-mcp
   ```

## 🔌 AI Client Integration Issues

### Claude Desktop

**Issue: "Command not found" or "Server unavailable"**

**Check Configuration:**
```bash
# Verify config file location
cat ~/.config/claude-desktop/config.json

# Test absolute path
ls -la /path/to/ynab-mcp
/path/to/ynab-mcp --version
```

**Fix Configuration:**
```json
{
  "mcpServers": {
    "ynab-mcp": {
      "command": "/absolute/path/to/ynab-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### VS Code with Continue

**Issue: MCP server not recognized**

**Check Configuration:**
```bash
# Verify config file
cat .continue/config.json

# Test connection
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | /path/to/ynab-mcp
```

**Fix Configuration:**
```json
{
  "mcpServers": [
    {
      "name": "ynab-mcp",
      "command": "/absolute/path/to/ynab-mcp",
      "args": []
    }
  ]
}
```

## 🔬 Advanced Debugging

### Debug Mode with Detailed Logging

```bash
# Enable all debug logging
export RUST_LOG="ynab_mcp=trace,reqwest=debug"
./ynab-mcp

# Component-specific logging
export RUST_LOG="ynab_mcp::client=debug,ynab_mcp::handlers=debug"
./ynab-mcp

# Log to file
export RUST_LOG="debug"
./ynab-mcp 2>&1 | tee debug.log
```

### Network Traffic Analysis

```bash
# Monitor network calls (macOS)
sudo dtruss -f -n ynab-mcp

# Monitor network calls (Linux)
strace -e trace=network ./ynab-mcp

# Use tcpdump to capture traffic
sudo tcpdump -i any host api.ynab.com
```

### Memory and Performance Issues

```bash
# Check memory usage
ps aux | grep ynab-mcp

# Monitor with htop
htop -p $(pgrep ynab-mcp)

# Profile with Rust tools (if compiled with debug symbols)
valgrind --tool=massif ./ynab-mcp
```

## 📊 Performance Optimization

### Cache Tuning

```bash
# Check cache performance
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"cache_management","arguments":{"action":"stats"}},"id":1}' | ./ynab-mcp

# Clean up old cache entries
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"cache_management","arguments":{"action":"cleanup"}},"id":1}' | ./ynab-mcp
```

### Resource Monitoring

```bash
# Monitor startup time
time ./ynab-mcp --version

# Monitor memory usage over time
while true; do ps -o pid,rss,cmd | grep ynab-mcp; sleep 5; done

# Monitor API response times
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"health_check","arguments":{}},"id":1}' | time ./ynab-mcp
```

## 🆘 Getting Help

If you're still experiencing issues:

1. **Check existing issues:** [GitHub Issues](https://github.com/cinnes/ynab-mcp/issues)
2. **Create detailed issue report:**
   - Include `./ynab-mcp --version` output
   - Include relevant log output with `RUST_LOG=debug`
   - Describe your platform and setup
   - Include minimal reproduction steps

3. **Security-sensitive issues:** Email maintainer directly (see README)

4. **Include diagnostics:**
   ```bash
   # Generate diagnostic report
   ./ynab-mcp --version
   ./ynab-mcp status
   echo $RUST_LOG
   uname -a
   ```

## 📝 Known Limitations

- YNAB API rate limit: 200 requests/hour
- Docker keyring integration limitations on some platforms
- Windows SmartScreen may flag unknown binaries
- Corporate proxy/firewall configurations may block YNAB API access
- Some Linux distributions require manual keyring setup

For the most up-to-date information, check the [project documentation](../README.md) and [GitHub issues](https://github.com/cinnes/ynab-mcp/issues).