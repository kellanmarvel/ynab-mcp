// mod benchmarks; // Removed - not needed for MCP operations (development/testing only)
mod config;
mod error;
mod handlers;
// mod metrics; // Removed - complex metrics system replaced with simple_health
mod health;
// mod plugins; // Removed - functionality embedded in system handlers
// mod progress; // Removed - not needed for MCP operations
mod schemas;
mod secrets;
mod server;
// mod streaming; // Removed - replaced with simple_pagination (also unused)
// mod simple_pagination; // Removed - not needed for current MCP operations
mod utils;
mod ynab;

use crate::config::Config;
use crate::error::{Result, YnabMcpError};
use clap::{Parser, Subcommand};
use secrets::{SecretsConfig, SecretsManager, YnabApiToken};
use server::YnabMcpServer;
use std::io::{self, BufRead, BufReader, Write};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "ynab-mcp")]
#[command(about = "YNAB Model Context Protocol Server")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Disable keyring usage (use environment variables only)
    #[arg(long)]
    no_keyring: bool,

    /// Require secure storage (fail if keyring unavailable)
    #[arg(long)]
    require_secure_storage: bool,

    /// Disable automatic migration from env vars to keyring
    #[arg(long)]
    no_auto_migrate: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Store API token in system keyring
    StoreToken,
    /// Delete API token from system keyring
    DeleteToken,
    /// Show secrets management status
    Status,
    /// Run the MCP server (default if no command specified)
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create secrets configuration
    let secrets_config = SecretsConfig {
        use_keyring: !cli.no_keyring,
        auto_migrate: !cli.no_auto_migrate,
        require_secure_storage: cli.require_secure_storage,
    };

    // Handle subcommands
    match cli.command {
        Some(Commands::StoreToken) => {
            return handle_store_token(secrets_config).await;
        }
        Some(Commands::DeleteToken) => {
            return handle_delete_token(secrets_config).await;
        }
        Some(Commands::Status) => {
            return handle_status(secrets_config).await;
        }
        Some(Commands::Run) | None => {
            // Continue to run the server
        }
    }

    // Load configuration with secrets support
    let config = Config::from_env_with_secrets(secrets_config)?;
    config.validate()?;

    // Initialize structured tracing/logging with key-value pairs
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("ynab_mcp={}", config.log_level).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_level(true)
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .compact(), // Use compact format for structured output
        )
        .init();

    info!(
        server = "ynab-mcp",
        version = env!("CARGO_PKG_VERSION"),
        "Starting YNAB MCP Server"
    );
    debug!(
        read_only_mode = config.read_only_mode,
        log_level = %config.log_level,
        request_timeout_secs = config.request_timeout_secs,
        connect_timeout_secs = config.connect_timeout_secs,
        "Configuration loaded successfully"
    );

    // Create the MCP server
    let server = YnabMcpServer::new(config).await?;

    // Perform health check
    match server.health_check().await {
        Ok(health) => {
            if health["status"] == "healthy" {
                info!(
                    api_status = "healthy",
                    "Health check passed - YNAB API is accessible"
                );
            } else {
                warn!(
                    api_status = "unhealthy",
                    health_details = ?health,
                    "Health check indicates issues"
                );
            }
        }
        Err(e) => {
            error!(
                api_status = "failed",
                error = %e,
                "Health check failed"
            );
            std::process::exit(1);
        }
    }

    info!(
        status = "ready",
        input = "stdin",
        protocol = "json-rpc",
        "YNAB MCP Server is ready and listening for requests"
    );

    // Set up graceful shutdown signal handling
    tokio::spawn(async {
        match tokio::signal::ctrl_c().await {
            Ok(_) => {
                info!(
                    event = "shutdown_signal",
                    signal = "SIGINT",
                    "Received shutdown signal, preparing graceful shutdown..."
                );
                std::process::exit(0);
            }
            Err(e) => {
                error!(
                    event = "signal_handler_error",
                    error = %e,
                    "Failed to install signal handler"
                );
            }
        }
    });

    // Handle JSON-RPC requests via stdin/stdout
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF - client disconnected
                info!(
                    event = "client_disconnected",
                    reason = "eof",
                    "Client disconnected"
                );
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Generate correlation ID for request tracing
                let correlation_id = Uuid::new_v4();

                // Process JSON-RPC request with correlation context
                debug!(
                    correlation_id = %correlation_id,
                    request_length = line.len(),
                    "Processing JSON-RPC request"
                );

                // Check if this is a batch request (JSON array)
                if line.trim_start().starts_with('[') {
                    // Batch requests are not supported - return proper error response
                    let error_response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32600,
                            "message": "Invalid Request",
                            "data": "Batch requests are not supported by this server"
                        },
                        "id": null
                    });
                    let error_str = serde_json::to_string(&error_response)?;
                    stdout.write_all(error_str.as_bytes())?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
                    continue;
                }

                match handle_json_rpc_request(&server, line, correlation_id).await {
                    Ok(Some(response)) => {
                        let response_str = serde_json::to_string(&response)?;
                        // Send JSON-RPC response

                        stdout.write_all(response_str.as_bytes())?;
                        stdout.write_all(b"\n")?;
                        stdout.flush()?;
                    }
                    Ok(None) => {
                        // This was a notification - no response should be sent
                        debug!(
                            correlation_id = %correlation_id,
                            "Notification processed - no response sent"
                        );
                    }
                    Err(e) => {
                        let error_id = Uuid::new_v4();
                        error!(
                            correlation_id = %correlation_id,
                            error_id = %error_id,
                            error = %e,
                            "Error handling JSON-RPC request"
                        );

                        // Parse the request to get the ID and check if it's a notification
                        let request_id =
                            if let Ok(request) = serde_json::from_str::<serde_json::Value>(line) {
                                let is_notification = !request
                                    .as_object()
                                    .map(|obj| obj.contains_key("id"))
                                    .unwrap_or(false);
                                if is_notification {
                                    // Don't send error responses for notifications
                                    debug!(
                                        correlation_id = %correlation_id,
                                        "Error in notification processing - no error response sent"
                                    );
                                    continue;
                                }
                                request
                                    .get("id")
                                    .cloned()
                                    .unwrap_or(serde_json::Value::Null)
                            } else {
                                serde_json::Value::Null
                            };

                        // Structured error handling with specific codes
                        let error_str = e.to_string();
                        let (error_code, error_message, sanitized_message) = if error_str
                            .contains("YNAB_API_TOKEN")
                            || error_str.contains("Bearer")
                        {
                            (
                                -32001,
                                "Authentication Error",
                                "Authentication error - please check your YNAB API token"
                                    .to_string(),
                            )
                        } else if error_str.contains("rate limit") {
                            (
                                -32002,
                                "Rate Limit Exceeded",
                                "YNAB API rate limit exceeded - please wait and try again"
                                    .to_string(),
                            )
                        } else if error_str.contains("network") || error_str.contains("timeout") {
                            (
                                -32003,
                                "Network Error",
                                "Network error - please check your internet connection".to_string(),
                            )
                        } else if error_str.contains("Invalid")
                            || error_str.contains("validation")
                            || error_str.contains("Missing")
                        {
                            (
                                -32602,
                                "Invalid params",
                                "Invalid request parameters - please check your input".to_string(),
                            )
                        } else if error_str.contains("not found") || error_str.contains("Not found")
                        {
                            (
                                -32004,
                                "Resource Not Found",
                                "Requested resource was not found".to_string(),
                            )
                        } else if error_str.contains("permission")
                            || error_str.contains("unauthorized")
                            || error_str.contains("read-only mode")
                        {
                            (
                                -32005,
                                "Permission Denied",
                                "Insufficient permissions to perform this operation".to_string(),
                            )
                        } else {
                            (
                                -32603,
                                "Internal Error",
                                "Internal server error".to_string(),
                            )
                        };

                        let error_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": error_code,
                                "message": error_message,
                                "data": {
                                    "description": sanitized_message,
                                    "error_id": error_id.to_string(),
                                    "correlation_id": correlation_id.to_string(),
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }
                            },
                            "id": request_id
                        });

                        let error_str = serde_json::to_string(&error_response)?;
                        stdout.write_all(error_str.as_bytes())?;
                        stdout.write_all(b"\n")?;
                        stdout.flush()?;
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    info!(
        event = "shutdown",
        status = "graceful",
        "YNAB MCP Server shutting down"
    );
    Ok(())
}

async fn handle_json_rpc_request(
    server: &YnabMcpServer,
    request_str: &str,
    correlation_id: Uuid,
) -> Result<Option<serde_json::Value>> {
    let request: serde_json::Value = serde_json::from_str(request_str)?;

    let is_notification = !request
        .as_object()
        .map(|obj| obj.contains_key("id"))
        .unwrap_or(false);
    let id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .ok_or_else(|| YnabMcpError::generic("Missing method in JSON-RPC request"))?;

    debug!(
        correlation_id = %correlation_id,
        method = method,
        request_id = ?id,
        "Processing JSON-RPC method"
    );

    let params = request
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // Handle MCP method

    let result = match method {
        // MCP Initialization
        "initialize" => {
            info!(operation = "init", "Initializing MCP server");
            server.get_server_info().await
        }

        // MCP Capabilities
        "tools/list" => server.list_tools().await,
        "resources/list" => server.list_resources().await,
        "prompts/list" => server.list_prompts().await,

        // MCP Tool Calls
        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| YnabMcpError::generic("Missing tool name"))?;

            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            server.handle_tool_call(tool_name, arguments).await?
        }

        // MCP Resource Requests
        "resources/read" => {
            let uri = params
                .get("uri")
                .and_then(|u| u.as_str())
                .ok_or_else(|| YnabMcpError::generic("Missing resource URI"))?;

            server.handle_resource_request(uri).await?
        }

        // MCP Prompt Requests
        "prompts/get" => {
            let prompt_name = params
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| YnabMcpError::generic("Missing prompt name"))?;

            let arguments = params.get("arguments").cloned();

            server.handle_prompt_request(prompt_name, arguments).await?
        }

        // Health Check
        "health" => server.health_check().await?,

        // Unknown method
        _ => {
            // For notifications, don't send error responses either
            if is_notification {
                return Ok(None);
            }
            return Ok(Some(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": "Method not found",
                    "data": format!("Unknown method: {}", method)
                },
                "id": id
            })));
        }
    };

    debug!(
        correlation_id = %correlation_id,
        method = method,
        is_notification = is_notification,
        "Successfully processed JSON-RPC request"
    );

    // For notifications, don't send a response
    if is_notification {
        return Ok(None);
    }

    let mut response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    });

    // Add correlation ID to response metadata (optional, for debugging)
    if let Some(result_obj) = response.get_mut("result") {
        if let Some(result_map) = result_obj.as_object_mut() {
            result_map.insert(
                "_correlation_id".to_string(),
                serde_json::Value::String(correlation_id.to_string()),
            );
        }
    }

    Ok(Some(response))
}

/// Handle the store-token command
async fn handle_store_token(secrets_config: SecretsConfig) -> Result<()> {
    println!("🔐 YNAB Token Storage Setup");
    println!("This will securely store your YNAB API token in the system keyring.\n");

    // Initialize basic logging for this command
    tracing_subscriber::fmt::init();

    let secrets_manager = SecretsManager::new(secrets_config)?;

    if !secrets_manager.is_keyring_available() {
        eprintln!("❌ System keyring is not available on this platform.");
        eprintln!("Please ensure your system supports keyring storage.");
        std::process::exit(1);
    }

    // Prompt for token
    print!("Enter your YNAB API token: ");
    io::stdout().flush()?;

    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token = token_input.trim().to_string();

    // Input validation and sanitization
    if token.is_empty() {
        eprintln!("❌ No token provided");
        std::process::exit(1);
    }

    // Length validation
    if token.len() > 512 {
        eprintln!("❌ Token too long (max 512 characters)");
        std::process::exit(1);
    }

    // Character validation - YNAB tokens should be alphanumeric with some special chars
    if !token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        eprintln!("❌ Token contains invalid characters");
        std::process::exit(1);
    }

    let ynab_token = YnabApiToken::new(token);

    // Validate token format
    if let Err(e) = ynab_token.validate() {
        eprintln!("❌ Token validation failed: {}", e);
        std::process::exit(1);
    }

    // Store in keyring
    match secrets_manager.store_token(&ynab_token) {
        Ok(()) => {
            println!("✅ Token stored successfully in system keyring!");
            println!("💡 You can now remove YNAB_API_TOKEN from your environment variables.");
            println!("   The server will automatically use the securely stored token.");
        }
        Err(e) => {
            eprintln!("❌ Failed to store token: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle the delete-token command
async fn handle_delete_token(secrets_config: SecretsConfig) -> Result<()> {
    println!("🗑️  YNAB Token Deletion");

    // Initialize basic logging for this command
    tracing_subscriber::fmt::init();

    let secrets_manager = SecretsManager::new(secrets_config)?;

    if !secrets_manager.is_keyring_available() {
        eprintln!("❌ System keyring is not available on this platform.");
        std::process::exit(1);
    }

    print!("Are you sure you want to delete the stored token? (y/N): ");
    io::stdout().flush()?;

    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;

    if confirmation.trim().to_lowercase() != "y" {
        println!("❌ Token deletion cancelled.");
        return Ok(());
    }

    match secrets_manager.delete_token() {
        Ok(()) => {
            println!("✅ Token deleted successfully from system keyring!");
            println!(
                "💡 You'll need to set YNAB_API_TOKEN environment variable or store a new token."
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to delete token: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle the status command
async fn handle_status(secrets_config: SecretsConfig) -> Result<()> {
    println!("📊 YNAB MCP Server - Secrets Management Status\n");

    // Initialize basic logging for this command
    tracing_subscriber::fmt::init();

    let secrets_manager = SecretsManager::new(secrets_config)?;

    // Check keyring availability
    println!("🔐 Keyring Status:");
    if secrets_manager.is_keyring_available() {
        println!("   ✅ System keyring is available");
        println!(
            "   📂 Storage method: {}",
            secrets_manager.get_storage_method()
        );
    } else {
        println!("   ❌ System keyring is not available");
        println!("   📂 Storage method: Environment Variables");
    }

    // Check for tokens
    println!("\n🔑 Token Status:");
    match secrets_manager.load_token() {
        Ok(token) => {
            println!("   ✅ Token found and valid");
            println!("   📍 Source: {}", secrets_manager.get_storage_method());
            if !token.is_empty() {
                if let Err(e) = token.validate() {
                    println!("   ⚠️  Token validation warning: {}", e);
                } else {
                    println!("   ✅ Token format appears valid");
                }
            }
        }
        Err(e) => {
            println!("   ❌ No valid token found: {}", e);
        }
    }

    // Show configuration
    println!("\n⚙️  Configuration:");
    println!("   Use keyring: {}", secrets_config.use_keyring);
    println!("   Auto migrate: {}", secrets_config.auto_migrate);
    println!(
        "   Require secure storage: {}",
        secrets_config.require_secure_storage
    );

    // Show guidance
    println!("\n💡 Recommendations:");
    for guidance in secrets_manager.setup_guidance() {
        println!("   {}", guidance);
    }

    Ok(())
}
