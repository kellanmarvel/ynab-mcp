# YNAB MCP Server Documentation

## Quick Links

- **[Features & Architecture](./FEATURES.md)** - Complete feature overview and technical architecture
- **[Configuration Guide](./CONFIGURATION.md)** - Environment variables, setup, and troubleshooting  
- **[Docker Deployment](./DOCKER.md)** - Containerized deployment guide with examples

## Getting Started

1. **Install**: Download prebuilt binary or build from source
2. **Configure**: Set your YNAB API token and preferences
3. **Connect**: Add to Claude Desktop or your preferred MCP client
4. **Use**: Start asking natural language questions about your budget

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   AI Assistant  │◄──►│  YNAB MCP       │◄──►│   YNAB API      │
│   (Claude)      │    │  Server         │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
    MCP Protocol           Rust Runtime              HTTPS/JSON
    (JSON-RPC)            Memory Safe              OAuth Bearer Token
```

## Key Features

- **16 Financial Tools** - Complete transaction and budget management
- **8 Resource Types** - Access to all YNAB data structures  
- **Enterprise Security** - OS keyring integration, memory safety
- **High Performance** - Async Rust with intelligent caching
- **Docker Ready** - 20.8MB container with health checks

## Support

- **Issues**: [GitHub Issues](https://github.com/cinnes/ynab-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/cinnes/ynab-mcp/discussions)
- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md)