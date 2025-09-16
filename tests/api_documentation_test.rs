use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[tokio::test]
async fn test_api_documentation_tool_complete() -> Result<()> {
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "get_api_documentation",
            "arguments": {"type": "all"}
        }
    }))
    .await?;

    eprintln!(
        "API doc response: {}",
        serde_json::to_string_pretty(&response)?
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 10);
    assert!(response["result"].is_object());

    if response["result"]["isError"] == true {
        panic!(
            "API documentation tool failed: {:?}",
            response["result"]["content"]
        );
    }

    assert!(response["result"]["content"].is_array());
    let content = response["result"]["content"].as_array().unwrap();
    assert!(!content.is_empty(), "Should have documentation content");

    Ok(())
}

#[tokio::test]
async fn test_api_documentation_tool_tools_only() -> Result<()> {
    let response = send_single_request(json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/call",
        "params": {
            "name": "get_api_documentation",
            "arguments": {"type": "tools"}
        }
    }))
    .await?;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 11);
    assert!(response["result"].is_object());

    if response["result"]["isError"] == true {
        panic!(
            "API documentation tool failed: {:?}",
            response["result"]["content"]
        );
    }

    assert!(response["result"]["content"].is_array());
    let content = response["result"]["content"].as_array().unwrap();
    assert!(!content.is_empty(), "Should have tools documentation");

    // Should contain tool information
    let content_text = serde_json::to_string(&content)?;
    assert!(
        content_text.contains("create_transaction"),
        "Should contain transaction tools"
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
