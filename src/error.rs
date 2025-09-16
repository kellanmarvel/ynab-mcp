use thiserror::Error;

/// Enhanced error type for the YNAB MCP server with actionable context
///
/// Each error variant includes specific remediation guidance to help
/// users and AI assistants resolve issues effectively.
#[derive(Error, Debug)]
pub enum YnabMcpError {
    /// Authentication errors with specific remediation steps
    #[error("Authentication failed: {message}. {remediation}")]
    Authentication {
        message: String,
        remediation: String,
    },

    /// Rate limiting errors with timing information
    #[error("Rate limit exceeded: {message}. {next_timing} {remediation}")]
    RateLimit {
        message: String,
        next_timing: String,
        remediation: String,
    },

    /// Configuration errors with specific fix instructions
    #[error("Configuration error: {message}. {remediation}")]
    Config {
        message: String,
        remediation: String,
    },

    /// Input validation errors with parameter details
    #[error("Validation error: {message}. Parameter: '{parameter}', Expected: {expected}. {remediation}")]
    Validation {
        message: String,
        parameter: String,
        expected: String,
        remediation: String,
    },

    /// YNAB API errors with specific response details
    #[error("YNAB API error: {message} (Status: {status_code}). {remediation}")]
    YnabApi {
        message: String,
        status_code: u16,
        remediation: String,
    },

    /// Network connectivity errors with troubleshooting steps
    #[error("Network error: {message}. {remediation}")]
    Network {
        message: String,
        remediation: String,
    },

    /// Generic application errors (fallback)
    #[error("{0}")]
    Generic(String),
}

impl YnabMcpError {
    /// Create an authentication error with remediation steps
    pub fn authentication(message: &str, remediation: &str) -> Self {
        Self::Authentication {
            message: message.to_string(),
            remediation: remediation.to_string(),
        }
    }

    /// Create a rate limit error with timing information
    pub fn rate_limit(message: &str, next_timing: &str, remediation: &str) -> Self {
        Self::RateLimit {
            message: message.to_string(),
            next_timing: next_timing.to_string(),
            remediation: remediation.to_string(),
        }
    }

    /// Create a configuration error with remediation steps
    pub fn config(message: &str, remediation: &str) -> Self {
        Self::Config {
            message: message.to_string(),
            remediation: remediation.to_string(),
        }
    }

    /// Create a validation error with parameter details
    pub fn validation(message: &str, parameter: &str, expected: &str, remediation: &str) -> Self {
        Self::Validation {
            message: message.to_string(),
            parameter: parameter.to_string(),
            expected: expected.to_string(),
            remediation: remediation.to_string(),
        }
    }

    /// Create a YNAB API error with status code and remediation
    pub fn ynab_api(message: &str, status_code: u16, remediation: &str) -> Self {
        Self::YnabApi {
            message: message.to_string(),
            status_code,
            remediation: remediation.to_string(),
        }
    }

    /// Create a network error with troubleshooting steps
    pub fn network(message: &str, remediation: &str) -> Self {
        Self::Network {
            message: message.to_string(),
            remediation: remediation.to_string(),
        }
    }

    /// Create a generic error (fallback for compatibility)
    pub fn generic(msg: &str) -> Self {
        Self::Generic(msg.to_string())
    }
}

// Convert from external error types with enhanced context
impl From<std::io::Error> for YnabMcpError {
    fn from(err: std::io::Error) -> Self {
        Self::network(
            &format!("I/O operation failed: {}", err),
            "Check file permissions, disk space, and network connectivity. Verify the file or network resource is accessible."
        )
    }
}

impl From<reqwest::Error> for YnabMcpError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::network(
                &format!("Request timeout: {}", err),
                "The YNAB API request timed out. Check your internet connection and try again. Consider increasing timeout values if the issue persists."
            )
        } else if err.is_connect() {
            Self::network(
                &format!("Connection failed: {}", err),
                "Could not connect to YNAB API. Verify your internet connection and check if api.ynab.com is accessible."
            )
        } else {
            Self::network(
                &format!("HTTP request failed: {}", err),
                "The HTTP request to YNAB API failed. Check your internet connection and verify the API endpoint is correct."
            )
        }
    }
}

impl From<serde_json::Error> for YnabMcpError {
    fn from(err: serde_json::Error) -> Self {
        Self::ynab_api(
            &format!("JSON parsing failed: {}", err),
            0,
            "The response from YNAB API was not valid JSON. This may indicate an API issue or network problem. Try the request again."
        )
    }
}

impl From<uuid::Error> for YnabMcpError {
    fn from(err: uuid::Error) -> Self {
        Self::validation(
            &format!("Invalid UUID format: {}", err),
            "id",
            "valid UUID format (e.g., '12345678-1234-1234-1234-123456789abc')",
            "Ensure the ID parameter is a valid UUID. You can get valid IDs from the list_resources or list budgets/accounts operations."
        )
    }
}

impl From<anyhow::Error> for YnabMcpError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic(format!(
            "Internal error: {}. Please report this issue if it persists.",
            err
        ))
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, YnabMcpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction_and_display() {
        // Test authentication error
        let auth_error = YnabMcpError::authentication("Test auth error", "Check your token");
        assert!(auth_error.to_string().contains("Test auth error"));
        assert!(auth_error.to_string().contains("Check your token"));

        // Test validation error
        let validation_error = YnabMcpError::validation(
            "Invalid UUID",
            "budget_id",
            "valid UUID format",
            "Use a valid UUID",
        );
        assert!(validation_error.to_string().contains("Invalid UUID"));
        assert!(validation_error.to_string().contains("budget_id"));

        // Test rate limit error
        let rate_limit_error = YnabMcpError::rate_limit(
            "Rate limit exceeded",
            "Next reset in 1 hour",
            "Wait before making more requests",
        );
        assert!(rate_limit_error.to_string().contains("Rate limit exceeded"));
        assert!(rate_limit_error
            .to_string()
            .contains("Next reset in 1 hour"));

        // Test config error
        let config_error = YnabMcpError::config("Missing token", "Set YNAB_API_TOKEN");
        assert!(config_error.to_string().contains("Missing token"));
        assert!(config_error.to_string().contains("Set YNAB_API_TOKEN"));
    }

    #[test]
    fn test_error_debug_format() {
        let auth_error = YnabMcpError::authentication("Debug test", "Fix it");
        let debug_str = format!("{:?}", auth_error);
        assert!(debug_str.contains("Authentication"));
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_error_conversion_from_serde() {
        // Create a JSON parsing error by trying to parse invalid JSON
        let json_str = "{invalid json";
        let serde_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let ynab_error: YnabMcpError = serde_error.into();
        assert!(ynab_error.to_string().contains("JSON"));
    }

    #[test]
    fn test_error_conversion_from_anyhow() {
        let anyhow_error = anyhow::anyhow!("test anyhow error");
        let ynab_error: YnabMcpError = anyhow_error.into();
        assert!(ynab_error.to_string().contains("test anyhow error"));
    }

    #[test]
    fn test_error_conversion_from_std_io() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let ynab_error: YnabMcpError = io_error.into();
        assert!(ynab_error.to_string().contains("connection refused"));
    }
}
