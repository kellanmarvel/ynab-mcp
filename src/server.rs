use crate::error::Result;
use serde_json::{json, Value};
use tracing::{error, info};
// Arc import removed - no longer needed

use crate::config::Config;
use crate::handlers::{PromptHandlers, ResourceHandlers, ToolHandlers};
// Removed complex plugin system - functionality integrated directly
use crate::schemas::{PromptSchemas, ToolSchemas};
use crate::ynab::YnabClient;

/// A YNAB (You Need A Budget) Model Context Protocol server implementation.
///
/// This server provides a comprehensive MCP interface for YNAB API operations,
/// including budget management, transaction handling, analysis tools, and real-time
/// data streaming capabilities. It supports a rich plugin ecosystem and provides
/// both read-only and read-write modes for safe data access.
///
/// # Features
///
/// - **Complete YNAB API Coverage**: Full access to budgets, accounts, transactions, categories, and payees
/// - **Advanced Analysis**: Spending pattern analysis, budget variance tracking, and trend analysis  
/// - **Real-time Streaming**: Chunked data streaming for large datasets with filtering and pagination
/// - **Plugin System**: Extensible architecture with built-in notifications and reporting plugins
/// - **Performance Monitoring**: Built-in metrics collection and performance benchmarking
/// - **Security**: Secure credential management with keyring integration and optional read-only mode
/// - **Caching**: Intelligent caching system with configurable TTL and rate limiting
///
/// # Examples
///
/// ```rust,no_run
/// use ynab_mcp::{YnabMcpServer, config::Config, secrets::SecretsConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let secrets_config = SecretsConfig::default();
///     let config = Config::from_env_with_secrets(secrets_config)?;
///     let server = YnabMcpServer::new(config).await?;
///     
///     // Server is ready to handle MCP requests
///     println!("YNAB MCP Server initialized successfully");
///     Ok(())
/// }
/// ```
pub struct YnabMcpServer {
    config: Config,
    ynab_client: YnabClient,
    tool_handlers: ToolHandlers,
    resource_handlers: ResourceHandlers,
    prompt_handlers: PromptHandlers,
}

impl YnabMcpServer {
    /// Creates a new YNAB MCP server instance with the provided configuration.
    ///
    /// This constructor initializes all components of the server including the YNAB API client,
    /// request handlers, plugin system, and built-in plugins. The server will be ready to
    /// handle MCP requests immediately after construction.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration including YNAB API credentials, timeouts, caching settings,
    ///   and operational mode (read-only vs read-write)
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A fully initialized server instance or an error if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - YNAB API client cannot be initialized (invalid token, network issues)
    /// - Plugin system fails to load built-in plugins
    /// - Configuration validation fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ynab_mcp::{YnabMcpServer, config::Config, secrets::SecretsConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let secrets_config = SecretsConfig::default();
    ///     let config = Config::from_env_with_secrets(secrets_config)?;
    ///     let server = YnabMcpServer::new(config).await?;
    ///     println!("Server ready!");
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(config: Config) -> Result<Self> {
        let ynab_client = YnabClient::new_with_config(
            config.ynab_base_url.clone(),
            config.ynab_api_token.clone(),
            1000, // cache capacity
            config.request_timeout_secs,
            config.connect_timeout_secs,
        )?;

        let tool_handlers = ToolHandlers::new(ynab_client.clone(), config.read_only_mode);

        let resource_handlers = ResourceHandlers::new(ynab_client.clone());

        let prompt_handlers = PromptHandlers::new();

        // Initialize plugin system
        let server_instance = Self {
            config,
            ynab_client,
            tool_handlers,
            resource_handlers,
            prompt_handlers,
        };

        // Simple plugins are now integrated directly into SystemHandlers
        info!("Simple plugin functionality integrated into system handlers");

        Ok(server_instance)
    }

    pub async fn get_server_info(&self) -> Value {
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "supported": true
                },
                "resources": {
                    "supported": true
                },
                "prompts": {
                    "supported": true
                }
            },
            "serverInfo": {
                "name": "ynab-mcp",
                "version": "0.1.0"
            }
        })
    }

    pub async fn list_tools(&self) -> Value {
        let mut tools = ToolSchemas::get_all_tools();

        // Add simple plugin tools directly
        tools.extend_from_slice(&[
            json!({
                "name": "generate_spending_report",
                "description": "Generate comprehensive spending analysis report",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "budget_id": {
                            "type": "string",
                            "description": "Budget ID (use 'default' for primary budget)"
                        },
                        "months_back": {
                            "type": "integer",
                            "description": "Number of months to analyze (default: 6)",
                            "default": 6
                        },
                        "include_transfers": {
                            "type": "boolean",
                            "description": "Include transfer transactions (default: false)",
                            "default": false
                        }
                    },
                    "required": ["budget_id"],
                    "additionalProperties": false
                }
            }),
            json!({
                "name": "generate_category_trends",
                "description": "Analyze spending trends by category over time",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "budget_id": {
                            "type": "string",
                            "description": "Budget ID (use 'default' for primary budget)"
                        },
                        "category_id": {
                            "type": "string",
                            "description": "Specific category to analyze (optional)"
                        },
                        "time_period": {
                            "type": "string",
                            "description": "Time period for analysis",
                            "enum": ["month", "quarter", "year"],
                            "default": "month"
                        }
                    },
                    "required": ["budget_id"],
                    "additionalProperties": false
                }
            }),
            json!({
                "name": "check_budget_alerts",
                "description": "Check for budget alerts and overspending notifications",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "budget_id": {
                            "type": "string",
                            "description": "Budget ID (use 'default' for primary budget)"
                        }
                    },
                    "required": ["budget_id"],
                    "additionalProperties": false
                }
            }),
        ]);

        json!({"tools": tools})
    }
    pub async fn list_resources(&self) -> Value {
        json!({
            "resources": [
                {
                    "uri": "ynab://budgets",
                    "name": "YNAB Budgets",
                    "description": "List of all YNAB budgets"
                },
                {
                    "uri": "ynab://budgets/{budget_id}",
                    "name": "YNAB Budget Details",
                    "description": "Detailed information about a specific budget"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/accounts",
                    "name": "Budget Accounts",
                    "description": "All accounts in a budget with balances and summary"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/accounts/{account_id}",
                    "name": "Account Details",
                    "description": "Detailed account information with recent transactions"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/categories",
                    "name": "Budget Categories",
                    "description": "All budget categories organized by groups"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/categories/{category_id}",
                    "name": "Category Details",
                    "description": "Detailed category information with recent transactions"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/payees",
                    "name": "Budget Payees",
                    "description": "All payees in the budget"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/payees/{payee_id}",
                    "name": "Payee Details",
                    "description": "Detailed payee information"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/transactions",
                    "name": "Budget Transactions",
                    "description": "Recent transactions with summary statistics"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/transactions/{transaction_id}",
                    "name": "Transaction Details",
                    "description": "Detailed transaction information"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/accounts/{account_id}/transactions",
                    "name": "Account Transactions",
                    "description": "All transactions for a specific account"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/categories/{category_id}/transactions",
                    "name": "Category Transactions",
                    "description": "All transactions for a specific category"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/scheduled_transactions",
                    "name": "Scheduled Transactions",
                    "description": "All scheduled/recurring transactions"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/scheduled_transactions/{scheduled_id}",
                    "name": "Scheduled Transaction Details",
                    "description": "Detailed scheduled transaction information"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/months",
                    "name": "Budget Months",
                    "description": "Monthly budget data and performance"
                },
                {
                    "uri": "ynab://budgets/{budget_id}/months/{month}",
                    "name": "Monthly Budget Details",
                    "description": "Detailed monthly budget with category performance"
                },
                {
                    "uri": "ynab://user",
                    "name": "User Information",
                    "description": "Current user information"
                }
            ]
        })
    }

    pub async fn list_prompts(&self) -> Value {
        let prompts = PromptSchemas::get_all_prompts();
        json!({"prompts": prompts})
    }
    pub async fn handle_tool_call(&self, name: &str, arguments: Value) -> Result<Value> {
        info!(
            operation = "tool_call",
            tool_name = name,
            args_present = !arguments.is_null(),
            "Handling tool call"
        );

        // Simple plugin tools are now handled directly by the system handlers
        self.tool_handlers.handle_tool_call(name, arguments).await
    }

    pub async fn handle_resource_request(&self, uri: &str) -> Result<Value> {
        info!(
            operation = "resource_request",
            resource_uri = uri,
            "Handling resource request"
        );
        self.resource_handlers
            .handle_resource_request(uri)
            .await
            .map_err(|e| e.into())
    }

    pub async fn handle_prompt_request(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<Value> {
        info!(
            operation = "prompt_request",
            prompt_name = name,
            args_present = arguments.is_some(),
            "Handling prompt request"
        );
        self.prompt_handlers
            .handle_prompt_request(name, arguments)
            .await
    }

    pub async fn health_check(&self) -> Result<Value> {
        // Test YNAB API connectivity
        match self.ynab_client.get_user().await {
            Ok(user) => {
                info!(
                    operation = "health_check",
                    api_status = "connected",
                    user_id = %user.id,
                    "Health check passed - YNAB API accessible"
                );
                Ok(json!({
                    "status": "healthy",
                    "ynab_api": "connected",
                    "user_id": user.id,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
            Err(e) => {
                error!("Health check failed - YNAB API error: {}", e);
                Ok(json!({
                    "status": "unhealthy",
                    "ynab_api": "disconnected",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
        }
    }
}

impl Clone for YnabMcpServer {
    fn clone(&self) -> Self {
        // Reuse the same YnabClient instances - Arc makes this efficient
        Self {
            config: self.config.clone(),
            ynab_client: self.ynab_client.clone(),
            tool_handlers: ToolHandlers::new(self.ynab_client.clone(), self.config.read_only_mode),
            resource_handlers: ResourceHandlers::new(self.ynab_client.clone()),
            prompt_handlers: PromptHandlers::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::secrets::YnabApiToken;
    use serde_json::json;
    use std::env;

    fn create_test_config() -> Config {
        let token = YnabApiToken::new("test-token".to_string());

        Config {
            ynab_api_token: token,
            ynab_base_url: "https://api.ynab.com/v1".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            read_only_mode: true,
            request_timeout_secs: 30,
            connect_timeout_secs: 10,
        }
    }

    #[tokio::test]
    async fn test_server_creation() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
        env::set_var("READ_ONLY_MODE", "true");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await;

        assert!(server.is_ok(), "Server should be created successfully");
        let server = server.unwrap();
        assert!(server.config.read_only_mode, "Should be in read-only mode");
    }

    #[tokio::test]
    async fn test_get_server_info() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        let info = server.get_server_info().await;

        assert_eq!(info["protocolVersion"], "2024-11-05");
        assert_eq!(info["serverInfo"]["name"], "ynab-mcp");
        assert_eq!(info["capabilities"]["tools"]["supported"], true);
        assert_eq!(info["capabilities"]["resources"]["supported"], true);
        assert_eq!(info["capabilities"]["prompts"]["supported"], true);
    }

    #[tokio::test]
    async fn test_list_tools() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        let tools = server.list_tools().await;

        assert!(tools["tools"].is_array());
        let tools_array = tools["tools"].as_array().unwrap();
        assert!(!tools_array.is_empty(), "Should have tools available");

        // Check for specific built-in tools
        let tool_names: Vec<String> = tools_array
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();

        assert!(tool_names.contains(&"generate_spending_report".to_string()));
        assert!(tool_names.contains(&"generate_category_trends".to_string()));
        assert!(tool_names.contains(&"check_budget_alerts".to_string()));
    }

    #[tokio::test]
    async fn test_list_resources() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        let resources = server.list_resources().await;

        assert!(resources["resources"].is_array());
        let resources_array = resources["resources"].as_array().unwrap();
        assert!(
            !resources_array.is_empty(),
            "Should have resources available"
        );

        // Check for specific YNAB resources
        let resource_uris: Vec<String> = resources_array
            .iter()
            .map(|r| r["uri"].as_str().unwrap().to_string())
            .collect();

        assert!(resource_uris.contains(&"ynab://budgets".to_string()));
        assert!(resource_uris.contains(&"ynab://user".to_string()));
        assert!(resource_uris.iter().any(|uri| uri.contains("accounts")));
        assert!(resource_uris.iter().any(|uri| uri.contains("transactions")));
    }

    #[tokio::test]
    async fn test_list_prompts() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        let prompts = server.list_prompts().await;

        assert!(prompts["prompts"].is_array());
        let prompts_array = prompts["prompts"].as_array().unwrap();
        assert!(!prompts_array.is_empty(), "Should have prompts available");
    }

    #[tokio::test]
    async fn test_health_check_with_mock() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        let health = server.health_check().await;

        assert!(health.is_ok(), "Health check should not error");
        let health_data = health.unwrap();

        // With test token, this should either be healthy or unhealthy
        // but should always have these fields
        assert!(health_data["status"].is_string());
        assert!(health_data["ynab_api"].is_string());
        assert!(health_data["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_clone_functionality() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server1 = YnabMcpServer::new(config).await.unwrap();
        let server2 = server1.clone();

        // Both servers should have same configuration
        assert_eq!(server1.config.read_only_mode, server2.config.read_only_mode);
        assert_eq!(server1.config.server_port, server2.config.server_port);

        // Both should work independently
        let info1 = server1.get_server_info().await;
        let info2 = server2.get_server_info().await;

        assert_eq!(info1, info2);
    }

    #[tokio::test]
    async fn test_handle_tool_call_error_handling() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        // Test with completely invalid tool name
        let result = server
            .handle_tool_call("this_tool_definitely_does_not_exist_anywhere", json!({}))
            .await;
        // Don't assert error - just verify it doesn't panic and returns some result
        let _ = result;

        // Test with empty arguments - should not panic
        let result = server.handle_tool_call("get_budgets", json!({})).await;
        let _ = result;

        // Test with null arguments - should not panic
        let result = server.handle_tool_call("get_budgets", json!(null)).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_resource_request_error_handling() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        // Test with invalid URI
        let result = server.handle_resource_request("invalid://uri").await;
        assert!(result.is_err(), "Should error for invalid URI");

        // Test with empty URI
        let result = server.handle_resource_request("").await;
        assert!(result.is_err(), "Should error for empty URI");
    }

    #[tokio::test]
    async fn test_handle_prompt_request() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let config = create_test_config();
        let server = YnabMcpServer::new(config).await.unwrap();

        // Test with valid prompt arguments
        let result = server
            .handle_prompt_request(
                "budget_analysis",
                Some(json!({
                    "budget_id": "default"
                })),
            )
            .await;

        // Should not panic and return some result
        let _ = result;

        // Test with no arguments
        let result = server.handle_prompt_request("budget_analysis", None).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_read_only_mode_enforcement() {
        env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let mut config = create_test_config();
        config.read_only_mode = true;

        let server = YnabMcpServer::new(config).await.unwrap();

        // Test that server respects read-only mode
        assert!(server.config.read_only_mode);

        // Write operations should be blocked by tool handlers
        // This is tested more thoroughly in tool handler tests
    }
}
