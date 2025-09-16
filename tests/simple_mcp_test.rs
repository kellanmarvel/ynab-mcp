use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Simple MCP protocol test that verifies basic functionality
/// Each test spawns a fresh server instance for a single request
#[tokio::test]
async fn test_mcp_initialize() -> Result<()> {
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"roots": {"listChanged": true}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    }))
    .await?;

    // Verify basic structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["capabilities"].is_object());
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");

    Ok(())
}

#[tokio::test]
async fn test_tools_list() -> Result<()> {
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }))
    .await?;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "Should have at least one tool");

    // Check first tool has required fields
    let first_tool = &tools[0];
    assert!(first_tool["name"].is_string());
    assert!(first_tool["description"].is_string());
    assert!(first_tool["inputSchema"].is_object());

    Ok(())
}

#[tokio::test]
async fn test_resources_list() -> Result<()> {
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list",
        "params": {}
    }))
    .await?;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["result"]["resources"].is_array());

    Ok(())
}

#[tokio::test]
async fn test_health_check_tool() -> Result<()> {
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "health_check",
            "arguments": {}
        }
    }))
    .await?;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 4);
    assert!(response["result"].is_object());

    Ok(())
}

#[tokio::test]
async fn test_enhanced_error_message() -> Result<()> {
    // Test with invalid tool to trigger error handling
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "invalid_tool_name",
            "arguments": {}
        }
    }))
    .await?;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 5);

    // MCP tools return errors as successful results with isError flag
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["isError"], true);
    assert!(response["result"]["content"].is_array());

    let content = response["result"]["content"].as_array().unwrap();
    assert!(!content.is_empty(), "Should have error content");

    let error_text = &content[0];
    assert_eq!(error_text["type"], "text");
    assert!(error_text["text"].is_string());

    // Check that error message contains helpful information
    let message = error_text["text"].as_str().unwrap();
    assert!(message.len() > 10, "Error message should be descriptive");
    assert!(
        message.contains("Unknown tool"),
        "Should mention unknown tool"
    );

    Ok(())
}

// Helper function to send a single request to a fresh server instance
async fn send_single_request(request: Value) -> Result<Value> {
    let mut child = Command::new("./target/release/ynab-mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("YNAB_API_TOKEN", "test-token-for-mcp-compliance-testing")
        .env("YNAB_BLOCK_TEST_TOKENS", "false")
        .env("READ_ONLY_MODE", "true")
        .env("RUST_LOG", "error") // Minimal logging
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    // Send request
    let request_str = serde_json::to_string(&request)?;
    stdin.write_all(request_str.as_bytes()).await?;
    stdin.write_all(b"\n").await?;

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
            let response: Value = serde_json::from_str(response_line.trim())?;

            // Wait for server to finish
            let _ = child.wait().await;

            Ok(response)
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
            let _ = child.kill().await;
            Err(anyhow::anyhow!("Request timeout"))
        }
    }
}
