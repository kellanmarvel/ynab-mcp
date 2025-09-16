# Docker Deployment Guide

This guide covers how to deploy the YNAB MCP Server using Docker containers.

## Quick Start

### 1. Set Environment Variables

```bash
# Required: Your YNAB API token
export YNAB_API_TOKEN="your-ynab-api-token-here"

# Optional: Configure logging level
export RUST_LOG="info"
```

### 2. Build and Run

```bash
# Build the Docker image
docker-compose build

# Run the server
docker-compose up ynab-mcp
```

### 3. Test the Server

```bash
# Test basic functionality
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"roots":{"listChanged":true}},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | docker-compose exec -T ynab-mcp ynab-mcp
```

## Configuration Options

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `YNAB_API_TOKEN` | *required* | Your YNAB Personal Access Token |
| `RUST_LOG` | `info` | Logging level (error, warn, info, debug, trace) |
| `READ_ONLY_MODE` | `false` | Prevent write operations when true |
| `YNAB_BLOCK_TEST_TOKENS` | `true` | Block test tokens in production |
| `YNAB_HTTP_TIMEOUT_SECS` | `30` | HTTP request timeout |
| `YNAB_CONNECT_TIMEOUT_SECS` | `10` | Connection timeout |

### Using .env File

Create a `.env` file in the project root:

```bash
# .env
YNAB_API_TOKEN=your-token-here
RUST_LOG=info
READ_ONLY_MODE=false
```

Then run:
```bash
docker-compose up
```

## Deployment Scenarios

### 1. Production Deployment

```bash
# Production with security settings
YNAB_API_TOKEN=xxx READ_ONLY_MODE=true docker-compose up -d ynab-mcp
```

### 2. Development Mode

```bash
# Development with debug logging and test tokens allowed
docker-compose --profile dev up ynab-mcp-dev
```

### 3. Claude Desktop Integration

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "ynab": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "--env", "YNAB_API_TOKEN=your-token",
        "ynab-mcp:latest"
      ]
    }
  }
}
```

## Image Variants

### Standard Image (`ynab-mcp:latest`)
- **Size**: ~15MB (Alpine-based)
- **User**: Non-root (uid:1000)
- **Security**: Read-only filesystem, no new privileges
- **Use case**: Production deployments

### Development Image (`ynab-mcp:dev`)
- **Size**: ~500MB (includes Rust toolchain)
- **Features**: Source code access, development tools
- **Use case**: Development and debugging

## Health Checks

The container includes automatic health checks:

```bash
# Check container health
docker-compose ps

# Manual health check
docker-compose exec ynab-mcp /usr/local/bin/health-check.sh
```

Health check endpoint:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"health_check","arguments":{}}}
```

## Resource Limits

Default resource limits:
- **Memory**: 256MB limit, 128MB reserved
- **CPU**: 0.5 cores limit, 0.1 cores reserved

Adjust in `docker-compose.yml`:
```yaml
deploy:
  resources:
    limits:
      memory: 512M
      cpus: '1.0'
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   # Ensure proper permissions
   docker-compose exec ynab-mcp ls -la /usr/local/bin/ynab-mcp
   ```

2. **API Token Issues**
   ```bash
   # Test token validity
   docker-compose exec -e READ_ONLY_MODE=true ynab-mcp ynab-mcp <<< '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"health_check","arguments":{}}}'
   ```

3. **Container Won't Start**
   ```bash
   # Check logs
   docker-compose logs ynab-mcp
   
   # Check container status
   docker-compose ps
   ```

### Debug Mode

Run with debug logging:
```bash
RUST_LOG=debug docker-compose up ynab-mcp
```

### Shell Access

Access container shell for debugging:
```bash
# Production container
docker-compose exec ynab-mcp sh

# Development container
docker-compose --profile dev exec ynab-mcp-dev sh
```

## Security Considerations

### Container Security Features
- **Non-root user**: Runs as uid:1000
- **Read-only filesystem**: Prevents runtime modifications
- **No new privileges**: Blocks privilege escalation
- **Resource limits**: Prevents resource exhaustion

### Secrets Management
- Use Docker secrets for production:
  ```bash
  echo "your-token" | docker secret create ynab_api_token -
  ```

- Or use environment files:
  ```yaml
  env_file:
    - .env.production
  ```

### Network Security
- MCP servers don't expose network ports
- Communication via stdin/stdout only
- No inbound network connections

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Push Docker Image
on:
  push:
    branches: [main]

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Build and test
      run: |
        docker-compose build
        docker-compose run --rm ynab-mcp --version
    
    - name: Push to registry
      run: |
        echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
        docker-compose push
```

## Performance Tuning

### Build Optimization
```dockerfile
# Use buildkit for faster builds
export DOCKER_BUILDKIT=1
docker-compose build
```

### Runtime Optimization
```yaml
# Adjust resource limits based on usage
deploy:
  resources:
    limits:
      memory: 1G  # Increase for heavy workloads
      cpus: '2.0'
```

## Multi-Architecture Support

Build for multiple architectures:
```bash
# Enable buildx
docker buildx create --use

# Build multi-arch
docker buildx build --platform linux/amd64,linux/arm64 -t ynab-mcp:latest .
```