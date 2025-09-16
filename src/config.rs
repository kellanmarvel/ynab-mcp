use crate::error::{Result, YnabMcpError};
use crate::secrets::{SecretsConfig, SecretsManager, YnabApiToken};
use std::env;

/// Configuration for the YNAB MCP server
#[derive(Clone)]
pub struct Config {
    /// YNAB API token managed through secure storage
    pub ynab_api_token: YnabApiToken,
    /// YNAB API base URL
    pub ynab_base_url: String,
    /// Port for the MCP server
    pub server_port: u16,
    /// Logging level
    pub log_level: String,
    /// Read-only mode for safety
    pub read_only_mode: bool,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
}

impl Config {
    /// Create configuration from environment with custom secrets management
    pub fn from_env_with_secrets(secrets_config: SecretsConfig) -> Result<Self> {
        dotenvy::dotenv().ok();

        let secrets_manager = SecretsManager::new(secrets_config)?;
        let ynab_api_token = secrets_manager.load_token()?;

        let config = Config {
            ynab_api_token,
            ynab_base_url: env::var("YNAB_BASE_URL")
                .unwrap_or_else(|_| "https://api.ynab.com/v1".to_string()),
            server_port: parse_port(&env::var("MCP_SERVER_PORT").unwrap_or_default()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            read_only_mode: parse_bool(&env::var("READ_ONLY_MODE").unwrap_or_default()),
            request_timeout_secs: parse_timeout("REQUEST_TIMEOUT_SECS", 30, 5, 300),
            connect_timeout_secs: parse_timeout("CONNECT_TIMEOUT_SECS", 10, 1, 60),
        };

        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        self.ynab_api_token.validate()?;

        if !self.ynab_base_url.starts_with("https://") {
            return Err(YnabMcpError::config(
                &format!("YNAB base URL must use HTTPS. Got: '{}'", self.ynab_base_url),
                "Set YNAB_BASE_URL to 'https://api.ynab.com/v1' or ensure your custom URL uses HTTPS. HTTP is not supported for security reasons."
            ));
        }

        Ok(())
    }
}

// Helper functions for parsing environment variables
fn parse_port(value: &str) -> u16 {
    // Parse as u32 first to handle values > 65535, then clamp to u16 range
    match value.parse::<u32>() {
        Ok(port) => port.clamp(1, 65535) as u16,
        Err(_) => 3000, // Default fallback for invalid input
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
}

fn parse_timeout(var: &str, default: u64, min: u64, max: u64) -> u64 {
    env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .parse()
        .unwrap_or(default)
        .clamp(min, max)
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("ynab_api_token", &"[REDACTED]")
            .field("ynab_base_url", &self.ynab_base_url)
            .field("server_port", &self.server_port)
            .field("log_level", &self.log_level)
            .field("read_only_mode", &self.read_only_mode)
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ynab_api_token: YnabApiToken::new(String::new()),
            ynab_base_url: "https://api.ynab.com/v1".to_string(),
            server_port: 3000,
            log_level: "info".to_string(),
            read_only_mode: false,
            request_timeout_secs: 30,
            connect_timeout_secs: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port() {
        assert_eq!(parse_port("8080"), 8080);
        assert_eq!(parse_port("0"), 1); // Clamped to minimum
        assert_eq!(parse_port("70000"), 65535); // Clamped to maximum
        assert_eq!(parse_port("invalid"), 3000); // Falls back to default
        assert_eq!(parse_port(""), 3000); // Empty string falls back to default
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true"));
        assert!(parse_bool("True"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("on"));

        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("invalid"));
        assert!(!parse_bool(""));
    }

    #[test]
    fn test_parse_timeout() {
        assert_eq!(parse_timeout("NON_EXISTENT", 30, 5, 300), 30); // Default

        // Test with actual env var (simulate by direct parsing)
        let value = "45";
        let parsed: u64 = value.parse().unwrap_or(30).clamp(5, 300);
        assert_eq!(parsed, 45);

        let value = "1";
        let parsed: u64 = value.parse().unwrap_or(30).clamp(5, 300);
        assert_eq!(parsed, 5); // Clamped to minimum

        let value = "500";
        let parsed: u64 = value.parse().unwrap_or(30).clamp(5, 300);
        assert_eq!(parsed, 300); // Clamped to maximum
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.ynab_api_token.expose_secret(), "");
        assert_eq!(config.ynab_base_url, "https://api.ynab.com/v1");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "info");
        assert!(!config.read_only_mode);
        assert_eq!(config.request_timeout_secs, 30);
        assert_eq!(config.connect_timeout_secs, 10);
    }

    #[test]
    fn test_config_validation_invalid_token() {
        let config = Config {
            ynab_api_token: YnabApiToken::new("".to_string()),
            ..Config::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_config_validation_valid_token() {
        let config = Config {
            ynab_api_token: YnabApiToken::new("valid-token-123".to_string()),
            ..Config::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_port_edge_cases() {
        // Test boundary conditions
        assert_eq!(parse_port("1"), 1); // Minimum valid port
        assert_eq!(parse_port("65535"), 65535); // Maximum valid port
        assert_eq!(parse_port("0"), 1); // Below minimum, clamped up
        assert_eq!(parse_port("65536"), 65535); // Above maximum, clamped down

        // Test invalid inputs
        assert_eq!(parse_port("not_a_number"), 3000); // Non-numeric
        assert_eq!(parse_port("-1"), 3000); // Negative (parsing fails)
        assert_eq!(parse_port("3.14"), 3000); // Decimal (parsing fails)
        assert_eq!(parse_port("123abc"), 3000); // Mixed chars (parsing fails)
    }

    #[test]
    fn test_timeout_validation() {
        // Test parse_timeout helper function directly
        use std::env;

        // Set test environment variables
        env::set_var("TEST_TIMEOUT_VALID", "45");
        env::set_var("TEST_TIMEOUT_LOW", "1");
        env::set_var("TEST_TIMEOUT_HIGH", "500");
        env::set_var("TEST_TIMEOUT_INVALID", "not_a_number");

        assert_eq!(parse_timeout("TEST_TIMEOUT_VALID", 30, 5, 300), 45);
        assert_eq!(parse_timeout("TEST_TIMEOUT_LOW", 30, 5, 300), 5); // Clamped to min
        assert_eq!(parse_timeout("TEST_TIMEOUT_HIGH", 30, 5, 300), 300); // Clamped to max
        assert_eq!(parse_timeout("TEST_TIMEOUT_INVALID", 30, 5, 300), 30); // Default fallback
        assert_eq!(parse_timeout("NONEXISTENT_VAR", 30, 5, 300), 30); // Default fallback

        // Clean up test env vars
        env::remove_var("TEST_TIMEOUT_VALID");
        env::remove_var("TEST_TIMEOUT_LOW");
        env::remove_var("TEST_TIMEOUT_HIGH");
        env::remove_var("TEST_TIMEOUT_INVALID");
    }

    #[test]
    fn test_bool_parsing_comprehensive() {
        // All truthy values
        let truthy = [
            "true", "True", "TRUE", "1", "yes", "Yes", "YES", "on", "On", "ON",
        ];
        for value in &truthy {
            assert!(parse_bool(value), "Expected '{}' to parse as true", value);
        }

        // All falsy values
        let falsy = [
            "false",
            "False",
            "FALSE",
            "0",
            "no",
            "No",
            "NO",
            "off",
            "Off",
            "OFF",
            "",
            "invalid",
            "maybe",
            "2",
            "-1",
            "absolutely",
        ];
        for value in &falsy {
            assert!(!parse_bool(value), "Expected '{}' to parse as false", value);
        }
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Empty token should fail
        assert!(config.validate().is_err());

        // Valid token with HTTP should fail
        config.ynab_api_token = YnabApiToken::new("valid_token_12345".to_string());
        config.ynab_base_url = "http://api.ynab.com/v1".to_string();
        assert!(config.validate().is_err());

        // Valid configuration should pass
        config.ynab_base_url = "https://api.ynab.com/v1".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_from_env_with_secrets_success() {
        // Set test environment variables
        env::set_var("YNAB_API_TOKEN", "test-token-123");
        env::set_var("YNAB_BASE_URL", "https://test.api.com/v1");
        env::set_var("MCP_SERVER_PORT", "8080");
        env::set_var("LOG_LEVEL", "debug");
        env::set_var("READ_ONLY_MODE", "true");
        env::set_var("REQUEST_TIMEOUT_SECS", "45");
        env::set_var("CONNECT_TIMEOUT_SECS", "15");

        let secrets_config = crate::secrets::SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        let result = Config::from_env_with_secrets(secrets_config);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.ynab_base_url, "https://test.api.com/v1");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "debug");
        assert!(config.read_only_mode);
        assert_eq!(config.request_timeout_secs, 45);
        assert_eq!(config.connect_timeout_secs, 15);

        // Clean up
        env::remove_var("YNAB_API_TOKEN");
        env::remove_var("YNAB_BASE_URL");
        env::remove_var("MCP_SERVER_PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("READ_ONLY_MODE");
        env::remove_var("REQUEST_TIMEOUT_SECS");
        env::remove_var("CONNECT_TIMEOUT_SECS");
    }

    #[test]
    #[serial_test::serial]
    fn test_from_env_with_secrets_defaults() {
        // Store original values
        let orig_base_url = env::var("YNAB_BASE_URL").ok();
        let orig_port = env::var("MCP_SERVER_PORT").ok();
        let orig_log = env::var("LOG_LEVEL").ok();
        let orig_readonly = env::var("READ_ONLY_MODE").ok();
        let orig_req_timeout = env::var("REQUEST_TIMEOUT_SECS").ok();
        let orig_conn_timeout = env::var("CONNECT_TIMEOUT_SECS").ok();
        let orig_token = env::var("YNAB_API_TOKEN").ok();

        // Remove any existing env vars and ensure YNAB_API_TOKEN is set for the test
        env::remove_var("YNAB_BASE_URL");
        env::remove_var("MCP_SERVER_PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("READ_ONLY_MODE");
        env::remove_var("REQUEST_TIMEOUT_SECS");
        env::remove_var("CONNECT_TIMEOUT_SECS");
        env::set_var("YNAB_API_TOKEN", "test-token-default");

        let secrets_config = crate::secrets::SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        let result = Config::from_env_with_secrets(secrets_config);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.ynab_base_url, "https://api.ynab.com/v1");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "info");
        assert!(!config.read_only_mode);
        assert_eq!(config.request_timeout_secs, 30);
        assert_eq!(config.connect_timeout_secs, 10);

        // Restore original values
        if let Some(val) = orig_base_url {
            env::set_var("YNAB_BASE_URL", val);
        }
        if let Some(val) = orig_port {
            env::set_var("MCP_SERVER_PORT", val);
        }
        if let Some(val) = orig_log {
            env::set_var("LOG_LEVEL", val);
        }
        if let Some(val) = orig_readonly {
            env::set_var("READ_ONLY_MODE", val);
        }
        if let Some(val) = orig_req_timeout {
            env::set_var("REQUEST_TIMEOUT_SECS", val);
        }
        if let Some(val) = orig_conn_timeout {
            env::set_var("CONNECT_TIMEOUT_SECS", val);
        }
        if let Some(val) = orig_token {
            env::set_var("YNAB_API_TOKEN", val);
        } else {
            env::remove_var("YNAB_API_TOKEN");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_from_env_with_secrets_missing_token() {
        // Store original token
        let orig_token = env::var("YNAB_API_TOKEN").ok();

        // Ensure no token is available
        env::remove_var("YNAB_API_TOKEN");

        let secrets_config = crate::secrets::SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        let result = Config::from_env_with_secrets(secrets_config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Token loading will fail due to no token available
        assert!(error_msg.contains("token") || error_msg.contains("environment variable"));

        // Restore original token
        if let Some(val) = orig_token {
            env::set_var("YNAB_API_TOKEN", val);
        }
    }
}
