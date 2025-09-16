# Multi-stage build for optimal image size
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY tests ./tests

# Build the application
RUN cargo build --release

# Runtime stage with minimal Alpine image
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    && rm -rf /var/cache/apk/*

# Create non-root user for security
RUN addgroup -g 1000 ynab && \
    adduser -D -s /bin/sh -u 1000 -G ynab ynab

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/ynab-mcp /usr/local/bin/ynab-mcp

# Make binary executable
RUN chmod +x /usr/local/bin/ynab-mcp

# Switch to non-root user
USER ynab

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"health_check","arguments":{}}}' | ynab-mcp > /dev/null || exit 1

# Set environment variables with defaults
ENV RUST_LOG=info
ENV READ_ONLY_MODE=false
ENV YNAB_BLOCK_TEST_TOKENS=true

# Expose no ports (MCP uses stdin/stdout)
# EXPOSE is not needed for MCP servers

# Labels for metadata
LABEL org.opencontainers.image.title="YNAB MCP Server"
LABEL org.opencontainers.image.description="Model Context Protocol server for YNAB integration"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.source="https://github.com/cinnes/ynab-mcp"
LABEL org.opencontainers.image.licenses="MIT"

# Entry point
ENTRYPOINT ["ynab-mcp"]