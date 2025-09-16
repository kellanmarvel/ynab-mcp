use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

/// Comprehensive MCP protocol compliance tests
///
/// These tests verify that the YNAB MCP server correctly implements
/// the Model Context Protocol specification, including proper JSON-RPC
/// handling, error codes, and response formats.

#[tokio::test]
async fn test_mcp_initialization_flow() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test initialize method
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": "init-1",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": true
                },
                "sampling": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response = send_request(&mut server, &init_request)
        .await
        .expect("Initialize request failed");

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "init-1");
    assert!(
        response.get("error").is_none(),
        "Initialize should not return error"
    );

    let result = response["result"]
        .as_object()
        .expect("Result should be object");

    // Verify server info
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(
        result.get("capabilities").is_some(),
        "Should include capabilities"
    );
    assert!(
        result.get("serverInfo").is_some(),
        "Should include server info"
    );

    let server_info = &result["serverInfo"];
    assert_eq!(server_info["name"], "ynab-mcp");
    assert!(server_info.get("version").is_some(), "Should have version");

    let capabilities = &result["capabilities"];
    assert_eq!(capabilities["tools"]["supported"], true);
    assert_eq!(capabilities["resources"]["supported"], true);
    assert_eq!(capabilities["prompts"]["supported"], true);

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_tools_list_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": "tools-list-1",
        "method": "tools/list"
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Tools list request failed");

    // Verify JSON-RPC compliance
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "tools-list-1");
    assert!(response.get("error").is_none());

    let result = response["result"]
        .as_object()
        .expect("Result should be object");
    let tools = result["tools"].as_array().expect("Tools should be array");

    // Should have expected number of tools
    assert_eq!(
        tools.len(),
        16,
        "Should have 16 tools including documentation and reporting tools"
    );

    // Verify each tool has required fields
    for tool in tools {
        let tool_obj = tool.as_object().expect("Tool should be object");

        assert!(tool_obj.get("name").is_some(), "Tool should have name");
        assert!(
            tool_obj.get("description").is_some(),
            "Tool should have description"
        );
        assert!(
            tool_obj.get("inputSchema").is_some(),
            "Tool should have inputSchema"
        );

        let input_schema = &tool_obj["inputSchema"];
        assert_eq!(
            input_schema["type"], "object",
            "Input schema should be object type"
        );
        assert!(
            input_schema.get("properties").is_some(),
            "Should have properties"
        );
    }

    // Verify specific expected tools exist
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        tool_names.contains(&"create_transaction"),
        "Should include create_transaction"
    );
    assert!(
        tool_names.contains(&"health_check"),
        "Should include health_check"
    );
    assert!(
        tool_names.contains(&"get_api_documentation"),
        "Should include documentation tool"
    );

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_resources_list_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": "resources-1",
        "method": "resources/list"
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Resources list request failed");

    // Verify JSON-RPC compliance
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "resources-1");
    assert!(response.get("error").is_none());

    let result = response["result"]
        .as_object()
        .expect("Result should be object");
    let resources = result["resources"]
        .as_array()
        .expect("Resources should be array");

    // Should have at least the expected resource types
    assert!(
        resources.len() >= 8,
        "Should have at least 8 resource types"
    );

    // Verify resource structure
    for resource in resources {
        let resource_obj = resource.as_object().expect("Resource should be object");

        assert!(
            resource_obj.get("uri").is_some(),
            "Resource should have URI"
        );
        assert!(
            resource_obj.get("name").is_some(),
            "Resource should have name"
        );
        assert!(
            resource_obj.get("description").is_some(),
            "Resource should have description"
        );

        let uri = resource_obj["uri"].as_str().expect("URI should be string");
        assert!(uri.starts_with("ynab://"), "URI should start with ynab://");
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_tools_call_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test health_check tool as it doesn't require authentication
    let request = json!({
        "jsonrpc": "2.0",
        "id": "tool-call-1",
        "method": "tools/call",
        "params": {
            "name": "health_check",
            "arguments": {}
        }
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Tool call request failed");

    // Verify JSON-RPC compliance
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "tool-call-1");

    if response.get("error").is_some() {
        // If error, verify error structure
        let error = &response["error"];
        assert!(error.get("code").is_some(), "Error should have code");
        assert!(error.get("message").is_some(), "Error should have message");
    } else {
        // If success, verify result structure
        let result = response["result"]
            .as_object()
            .expect("Result should be object");

        // Verify MCP protocol compliance
        assert!(result.get("content").is_some(), "Should have content array");
        let content = result["content"]
            .as_array()
            .expect("Content should be array");
        assert!(!content.is_empty(), "Content should not be empty");

        // Health check should provide text response
        assert!(
            content.iter().any(|c| c.get("text").is_some()),
            "Should have text content"
        );

        // Health check should also provide structured data as resource
        assert!(
            content.iter().any(|c| c.get("resource").is_some()),
            "Should have resource with structured data"
        );
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_resources_read_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test reading user resource (should work without full auth)
    let request = json!({
        "jsonrpc": "2.0",
        "id": "resource-read-1",
        "method": "resources/read",
        "params": {
            "uri": "ynab://user"
        }
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Resource read request failed");

    // Verify JSON-RPC compliance
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "resource-read-1");

    // This might fail due to auth, but should follow proper error format
    if response.get("error").is_some() {
        let error = &response["error"];
        assert!(error.get("code").is_some(), "Error should have code");
        assert!(error.get("message").is_some(), "Error should have message");

        // Should be authentication or permission error
        let code = error["code"].as_i64().expect("Code should be number");
        assert!(
            code == -32001 || code == -32005,
            "Should be auth or permission error"
        );
    } else {
        let result = response["result"]
            .as_object()
            .expect("Result should be object");
        assert!(result.get("contents").is_some(), "Should have contents");
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_prompts_list_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    let request = json!({
        "jsonrpc": "2.0",
        "id": "prompts-1",
        "method": "prompts/list"
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Prompts list request failed");

    // Verify JSON-RPC compliance
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "prompts-1");
    assert!(response.get("error").is_none());

    let result = response["result"]
        .as_object()
        .expect("Result should be object");
    let prompts = result["prompts"]
        .as_array()
        .expect("Prompts should be array");

    // Should have expected prompts
    assert_eq!(prompts.len(), 5, "Should have 5 prompts");

    // Verify prompt structure
    for prompt in prompts {
        let prompt_obj = prompt.as_object().expect("Prompt should be object");

        assert!(prompt_obj.get("name").is_some(), "Prompt should have name");
        assert!(
            prompt_obj.get("description").is_some(),
            "Prompt should have description"
        );
        assert!(
            prompt_obj.get("arguments").is_some(),
            "Prompt should have arguments"
        );

        let arguments = prompt_obj["arguments"]
            .as_array()
            .expect("Arguments should be array");
        for arg in arguments {
            let arg_obj = arg.as_object().expect("Argument should be object");
            assert!(arg_obj.get("name").is_some(), "Argument should have name");
            assert!(
                arg_obj.get("description").is_some(),
                "Argument should have description"
            );
            assert!(
                arg_obj.get("required").is_some(),
                "Argument should have required field"
            );
        }
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_error_handling_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test invalid method
    let invalid_method_request = json!({
        "jsonrpc": "2.0",
        "id": "error-1",
        "method": "invalid/method"
    });

    let response = send_request(&mut server, &invalid_method_request)
        .await
        .expect("Request should complete");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "error-1");
    assert!(
        response.get("error").is_some(),
        "Should return error for invalid method"
    );

    let error = &response["error"];
    assert_eq!(error["code"], -32601, "Should be method not found error");
    assert!(error.get("message").is_some(), "Should have error message");

    // Test invalid parameters
    let invalid_params_request = json!({
        "jsonrpc": "2.0",
        "id": "error-2",
        "method": "tools/call",
        "params": {
            "invalid": "params"
        }
    });

    let response2 = send_request(&mut server, &invalid_params_request)
        .await
        .expect("Request should complete");

    assert_eq!(response2["jsonrpc"], "2.0");
    assert_eq!(response2["id"], "error-2");
    assert!(
        response2.get("error").is_some(),
        "Should return error for invalid params"
    );

    let error2 = &response2["error"];
    assert_eq!(error2["code"], -32602, "Should be invalid params error");

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_correlation_id_handling() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    let correlation_id = Uuid::new_v4();
    let request = json!({
        "jsonrpc": "2.0",
        "id": correlation_id.to_string(),
        "method": "tools/list"
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Request should complete");

    // Verify correlation ID is preserved
    assert_eq!(response["id"], correlation_id.to_string());

    // Check if correlation ID is included in response metadata
    if let Some(result) = response["result"].as_object() {
        if let Some(correlation) = result.get("_correlation_id") {
            assert!(correlation.is_string(), "Correlation ID should be string");
        }
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_notification_handling() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Send notification (no ID)
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05"
        }
    });

    // Notifications should not return responses
    let result = send_notification(&mut server, &notification).await;

    // Should either get no response or empty response
    match result {
        Ok(None) => {
            // Correct - no response to notification
        }
        Ok(Some(_)) => {
            panic!("Should not respond to notifications");
        }
        Err(_) => {
            // Acceptable - server might close connection
        }
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_batch_request_compliance() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test batch request
    let batch_request = json!([
        {
            "jsonrpc": "2.0",
            "id": "batch-1",
            "method": "tools/list"
        },
        {
            "jsonrpc": "2.0",
            "id": "batch-2",
            "method": "resources/list"
        }
    ]);

    let response = send_request(&mut server, &batch_request)
        .await
        .expect("Batch request should complete");

    if let Some(responses) = response.as_array() {
        // Should return array of responses
        assert_eq!(responses.len(), 2, "Should return 2 responses");

        // Verify each response
        let mut ids = HashMap::new();
        for resp in responses {
            assert_eq!(resp["jsonrpc"], "2.0");
            let id = resp["id"].as_str().expect("Should have ID");
            assert!(id == "batch-1" || id == "batch-2", "Should have correct ID");
            ids.insert(id, true);
        }

        assert_eq!(ids.len(), 2, "Should have both response IDs");
    } else {
        // Server might not support batch requests - that's acceptable
        // But should still be valid JSON-RPC response
        assert_eq!(response["jsonrpc"], "2.0");
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_api_documentation_tool_functionality() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test the new documentation tool
    let request = json!({
        "jsonrpc": "2.0",
        "id": "doc-test-1",
        "method": "tools/call",
        "params": {
            "name": "get_api_documentation",
            "arguments": {
                "type": "tools"
            }
        }
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Documentation request failed");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "doc-test-1");

    if response.get("error").is_some() {
        // Might fail due to auth, but should be proper error format
        let error = &response["error"];
        assert!(error.get("code").is_some());
        assert!(error.get("message").is_some());
    } else {
        let result = response["result"].as_object().expect("Should have result");
        assert!(!result["isError"].as_bool().unwrap_or(true));

        // Parse the MCP tool result format
        let content = result["content"]
            .as_array()
            .expect("Should have content array");
        assert!(!content.is_empty(), "Content should not be empty");

        // Find the resource content with the actual tool result
        let resource_content = content
            .iter()
            .find(|item| item.get("resource").is_some())
            .and_then(|item| item["resource"]["text"].as_str())
            .expect("Should have resource with text content");

        let tool_result: serde_json::Value =
            serde_json::from_str(resource_content).expect("Resource text should be valid JSON");

        assert_eq!(tool_result["success"], true);
        assert!(
            tool_result.get("documentation").is_some(),
            "Should have documentation"
        );
        assert!(
            tool_result.get("conversational_summary").is_some(),
            "Should have summary"
        );
    }

    cleanup_server(server).await;
}

// Helper functions for test server management

async fn spawn_test_server() -> Result<tokio::process::Child> {
    let mut cmd = Command::new("./target/release/ynab-mcp");
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("YNAB_API_TOKEN", "test-token-for-mcp-compliance-testing")
        .env("YNAB_BLOCK_TEST_TOKENS", "false") // Allow test tokens
        .env("READ_ONLY_MODE", "true") // Prevent modifications
        .env("RUST_LOG", "warn") // Reduce logging noise
        .kill_on_drop(true);

    cmd.spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))
}

async fn spawn_test_server_for_validation() -> Result<tokio::process::Child> {
    let mut cmd = Command::new("./target/release/ynab-mcp");
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("YNAB_API_TOKEN", "test-token-for-mcp-compliance-testing")
        .env("YNAB_BLOCK_TEST_TOKENS", "false") // Allow test tokens
        .env("READ_ONLY_MODE", "false") // Allow validation errors to surface
        .env("RUST_LOG", "warn") // Reduce logging noise
        .kill_on_drop(true);

    cmd.spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))
}

async fn send_request(server: &mut tokio::process::Child, request: &Value) -> Result<Value> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stdin = server.stdin.as_mut().expect("Failed to get stdin");
    let stdout = server.stdout.as_mut().expect("Failed to get stdout");

    // Send request
    let request_str = serde_json::to_string(request)?;
    stdin.write_all(request_str.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read response
    let mut reader = BufReader::new(stdout);
    let mut response_line = String::new();

    // Wait for response with timeout
    tokio::select! {
        result = reader.read_line(&mut response_line) => {
            result?;
            if response_line.trim().is_empty() {
                return Err(anyhow::anyhow!("Empty response"));
            }
            serde_json::from_str(response_line.trim()).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
            Err(anyhow::anyhow!("Request timeout"))
        }
    }
}

async fn send_notification(
    server: &mut tokio::process::Child,
    notification: &Value,
) -> Result<Option<Value>> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stdin = server.stdin.as_mut().expect("Failed to get stdin");
    let stdout = server.stdout.as_mut().expect("Failed to get stdout");

    // Send notification
    let notification_str = serde_json::to_string(notification)?;
    stdin.write_all(notification_str.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Try to read response with short timeout
    let mut reader = BufReader::new(stdout);
    let mut response_line = String::new();

    tokio::select! {
        result = reader.read_line(&mut response_line) => {
            match result {
                Ok(0) => Ok(None), // EOF
                Ok(_) => {
                    if response_line.trim().is_empty() {
                        Ok(None)
                    } else {
                        let response = serde_json::from_str(response_line.trim())?;
                        Ok(Some(response))
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Read error: {}", e))
            }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
            Ok(None) // Timeout is expected for notifications
        }
    }
}

async fn cleanup_server(mut server: tokio::process::Child) {
    // Try graceful shutdown first
    if let Some(stdin) = server.stdin.as_mut() {
        let _ = stdin.shutdown().await;
    }

    // Wait a bit for graceful shutdown
    tokio::select! {
        _ = server.wait() => {}
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
            // Force kill if needed
            let _ = server.kill().await;
            let _ = server.wait().await;
        }
    }
}

#[tokio::test]
async fn test_enhanced_error_messages_integration() {
    // This test needs to run without read-only mode to test validation errors
    let mut server = spawn_test_server_for_validation()
        .await
        .expect("Failed to spawn server");

    // Test enhanced validation error
    let request = json!({
        "jsonrpc": "2.0",
        "id": "validation-error-test",
        "method": "tools/call",
        "params": {
            "name": "create_transaction",
            "arguments": {
                "budget_id": "default",
                "account_id": "invalid-uuid-format",
                "amount": "invalid-amount",
                "date": "2024-01-15"
            }
        }
    });

    let response = send_request(&mut server, &request)
        .await
        .expect("Request should complete");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], "validation-error-test");
    assert!(
        response.get("error").is_some(),
        "Should return validation error"
    );

    let error = &response["error"];
    let message = error["message"]
        .as_str()
        .expect("Should have error message");

    // Enhanced error should include actionable context
    assert!(
        message.contains("UUID") || message.contains("format") || message.contains("valid"),
        "Error message should provide actionable context: {}",
        message
    );

    // Error data should include remediation info
    if let Some(data) = error.get("data") {
        let data_str = data.to_string();
        assert!(
            data_str.contains("remediation") || data_str.contains("description"),
            "Error data should include remediation guidance"
        );
    }

    cleanup_server(server).await;
}

#[tokio::test]
async fn test_complete_mcp_workflow() {
    let mut server = spawn_test_server().await.expect("Failed to spawn server");

    // Test complete MCP workflow: initialize -> list -> call -> cleanup

    // Step 1: Initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": "workflow-init",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    });

    let init_response = send_request(&mut server, &init_request)
        .await
        .expect("Initialize failed");
    assert_eq!(init_response["jsonrpc"], "2.0");
    assert!(
        init_response.get("error").is_none(),
        "Initialize should succeed"
    );

    // Step 2: List capabilities
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": "workflow-tools",
        "method": "tools/list"
    });

    let tools_response = send_request(&mut server, &tools_request)
        .await
        .expect("Tools list failed");
    assert_eq!(tools_response["jsonrpc"], "2.0");
    assert!(
        tools_response.get("error").is_none(),
        "Tools list should succeed"
    );

    let resources_request = json!({
        "jsonrpc": "2.0",
        "id": "workflow-resources",
        "method": "resources/list"
    });

    let resources_response = send_request(&mut server, &resources_request)
        .await
        .expect("Resources list failed");
    assert_eq!(resources_response["jsonrpc"], "2.0");
    assert!(
        resources_response.get("error").is_none(),
        "Resources list should succeed"
    );

    // Step 3: Use tool
    let health_request = json!({
        "jsonrpc": "2.0",
        "id": "workflow-health",
        "method": "tools/call",
        "params": {
            "name": "health_check",
            "arguments": {}
        }
    });

    let health_response = send_request(&mut server, &health_request)
        .await
        .expect("Health check failed");
    assert_eq!(health_response["jsonrpc"], "2.0");
    // Health check might fail due to auth, but should be valid JSON-RPC

    // Step 4: Test documentation
    let doc_request = json!({
        "jsonrpc": "2.0",
        "id": "workflow-docs",
        "method": "tools/call",
        "params": {
            "name": "get_api_documentation",
            "arguments": {
                "type": "mcp"
            }
        }
    });

    let doc_response = send_request(&mut server, &doc_request)
        .await
        .expect("Documentation failed");
    assert_eq!(doc_response["jsonrpc"], "2.0");

    // All requests should maintain JSON-RPC compliance
    cleanup_server(server).await;
}
