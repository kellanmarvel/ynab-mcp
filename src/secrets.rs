use crate::error::{Result, YnabMcpError};
use keyring::{Entry, Error as KeyringError};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::fmt;
use tracing::{debug, info, warn};
use zeroize::Zeroize;

/// Service name for the keyring entry
const SERVICE_NAME: &str = "ynab-mcp";

/// User name for the keyring entry  
const USER_NAME: &str = "api-token";

/// Configuration for secrets management
#[derive(Debug, Clone, Copy)]
pub struct SecretsConfig {
    /// Whether to use keyring storage (OS-native secure storage)
    pub use_keyring: bool,
    /// Whether to automatically migrate from env vars to keyring
    pub auto_migrate: bool,
    /// Whether to require secure storage (fail if keyring unavailable)
    pub require_secure_storage: bool,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            use_keyring: true,
            auto_migrate: true,
            require_secure_storage: false, // Don't break existing deployments
        }
    }
}

/// Secure wrapper for the YNAB API token
#[derive(Clone, Zeroize)]
pub struct YnabApiToken {
    #[zeroize(skip)]
    inner: SecretString,
}

impl YnabApiToken {
    /// Create a new token from a string
    pub fn new(token: String) -> Self {
        Self {
            inner: SecretString::new(token.into()),
        }
    }

    /// Expose the secret for API calls (use sparingly!)
    pub fn expose_secret(&self) -> &str {
        self.inner.expose_secret()
    }

    /// Check if the token is empty
    pub fn is_empty(&self) -> bool {
        self.inner.expose_secret().is_empty()
    }

    /// Validate the token format
    pub fn validate(&self) -> Result<()> {
        let token = self.inner.expose_secret();

        if token.is_empty() {
            return Err(YnabMcpError::authentication(
                "YNAB API token is missing or empty",
                "Set your YNAB API token using: 1) Environment variable YNAB_API_TOKEN, 2) Run 'ynab-mcp store-token' to store securely in keyring, or 3) Get a token from https://app.youneedabudget.com/settings/developer"
            ));
        }

        // YNAB tokens typically start with a pattern, but we'll be permissive
        if token.len() < 10 {
            return Err(YnabMcpError::validation(
                &format!("YNAB API token is too short ({} characters)", token.len()),
                "ynab_api_token",
                "at least 10 characters (typically 43+ characters for real YNAB tokens)",
                "Check your token at https://app.youneedabudget.com/settings/developer and ensure you copied the complete token"
            ));
        }

        // Check for common test/placeholder tokens
        let is_test_token = Self::is_test_token(token);
        if is_test_token {
            let env_block_test = std::env::var("YNAB_BLOCK_TEST_TOKENS")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            if env_block_test {
                return Err(YnabMcpError::authentication(
                    "Test/placeholder token detected and blocked in production mode",
                    "Replace with a real YNAB API token from https://app.youneedabudget.com/settings/developer or set YNAB_BLOCK_TEST_TOKENS=false for testing"
                ));
            } else {
                warn!("Token appears to be a test/placeholder token - set YNAB_BLOCK_TEST_TOKENS=true to block in production");
            }
        }

        Ok(())
    }

    /// Check if a token appears to be a test/placeholder token
    pub fn is_test_token(token: &str) -> bool {
        let test_patterns = [
            "test",
            "dummy",
            "placeholder",
            "example",
            "fake",
            "demo",
            "sample",
            "mock",
            "ynab_api_token",
            "your_token_here",
            "xxx",
        ];
        let lower_token = token.to_lowercase();

        // Check for test patterns
        for pattern in &test_patterns {
            if lower_token.contains(pattern) {
                return true;
            }
        }

        // Check for repeated characters (like "aaaa" or "1111")
        if token.len() > 4 {
            if let Some(first_char) = token.chars().next() {
                if token.chars().all(|c| c == first_char) {
                    return true;
                }
            }
        }

        // Check for sequential patterns
        if token == "abcdefghijklmnopqrstuvwxyz" || token == "123456789" {
            return true;
        }

        false
    }
}

impl fmt::Debug for YnabApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YnabApiToken")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Display for YnabApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED TOKEN]")
    }
}

// Custom serde implementation to prevent accidental serialization
impl Serialize for YnabApiToken {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

impl<'de> Deserialize<'de> for YnabApiToken {
    fn deserialize<D>(_deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Never deserialize tokens - force explicit construction
        Err(serde::de::Error::custom(
            "YnabApiToken cannot be deserialized - must be constructed explicitly",
        ))
    }
}

/// Secrets manager for YNAB API tokens
pub struct SecretsManager {
    config: SecretsConfig,
    keyring_entry: Option<Entry>,
}

impl SecretsManager {
    /// Create a new secrets manager
    pub fn new(config: SecretsConfig) -> Result<Self> {
        let keyring_entry = if config.use_keyring {
            match Entry::new(SERVICE_NAME, USER_NAME) {
                Ok(entry) => {
                    info!("Keyring available: {}", Self::get_keyring_info());
                    Some(entry)
                }
                Err(e) => {
                    if config.require_secure_storage {
                        return Err(YnabMcpError::config(
                            &format!("Keyring required but unavailable: {}", e),
                            "Install and configure a keyring service on your system, or set REQUIRE_SECURE_STORAGE=false to allow environment variable storage"
                        ));
                    } else {
                        warn!(
                            "Keyring unavailable, falling back to environment variables: {}",
                            e
                        );
                        None
                    }
                }
            }
        } else {
            debug!("Keyring disabled by configuration");
            None
        };

        Ok(Self {
            config,
            keyring_entry,
        })
    }

    /// Get information about the available keyring backend
    fn get_keyring_info() -> String {
        // This would be platform-specific, but keyring crate doesn't expose this easily
        if cfg!(target_os = "macos") {
            "macOS Keychain".to_string()
        } else if cfg!(target_os = "windows") {
            "Windows Credential Manager".to_string()
        } else if cfg!(target_os = "linux") {
            "Linux Kernel Keyring (keyutils)".to_string()
        } else {
            "Generic keyring backend".to_string()
        }
    }

    /// Load the YNAB API token from the most secure available source
    pub fn load_token(&self) -> Result<YnabApiToken> {
        // Try keyring first if available
        if let Some(ref entry) = self.keyring_entry {
            match entry.get_password() {
                Ok(token) => {
                    debug!("Token loaded from keyring");
                    return Ok(YnabApiToken::new(token));
                }
                Err(KeyringError::NoEntry) => {
                    debug!("No token found in keyring");
                    // Continue to environment variables
                }
                Err(e) => {
                    warn!("Failed to read from keyring: {}", e);
                    if self.config.require_secure_storage {
                        return Err(YnabMcpError::authentication(
                            &format!("Failed to read from keyring: {}", e),
                            "Check that keyring service is running and accessible. On Linux, ensure gnome-keyring or kwallet is available. Set REQUIRE_SECURE_STORAGE=false to use environment variables instead."
                        ));
                    }
                    // Continue to environment variables
                }
            }
        }

        // Fall back to environment variables
        let token = env::var("YNAB_API_TOKEN")
            .or_else(|_| env::var("YNAB_ACCESS_TOKEN"))
            .map_err(|_| YnabMcpError::authentication(
                "YNAB API token not found",
                "Set your YNAB API token using environment variable YNAB_API_TOKEN or run 'ynab-mcp store-token' to store it securely. Get a token from https://app.youneedabudget.com/settings/developer"
            ))?;

        debug!("Token loaded from environment variables");

        let ynab_token = YnabApiToken::new(token);

        // Auto-migrate to keyring if enabled
        if self.config.auto_migrate && self.keyring_entry.is_some() {
            if let Err(e) = self.store_token(&ynab_token) {
                warn!(
                    operation = "token_migration",
                    result = "failed",
                    error = %e,
                    "Failed to migrate token to keyring"
                );
            } else {
                info!(
                    operation = "token_migration",
                    result = "success",
                    "Token automatically migrated to keyring for enhanced security"
                );
                info!(
                    operation = "token_migration",
                    cleanup_hint = "env_var_removal",
                    "You can now remove YNAB_API_TOKEN from environment variables"
                );
            }
        }

        Ok(ynab_token)
    }

    /// Store a token in the keyring
    pub fn store_token(&self, token: &YnabApiToken) -> Result<()> {
        let entry = self
            .keyring_entry
            .as_ref()
            .ok_or_else(|| YnabMcpError::config(
                "Keyring is not available on this system",
                "Install and configure a keyring service (gnome-keyring, kwallet, or Windows Credential Manager) or set USE_KEYRING=false to use environment variables"
            ))?;

        entry
            .set_password(token.expose_secret())
            .map_err(|e| YnabMcpError::authentication(
                &format!("Failed to write to keyring: {}", e),
                "Ensure keyring service is running and has write permissions. Check that the keyring is unlocked and accessible."
            ))?;

        info!("Token stored in keyring successfully");
        Ok(())
    }

    /// Delete the token from keyring
    pub fn delete_token(&self) -> Result<()> {
        let entry = self
            .keyring_entry
            .as_ref()
            .ok_or_else(|| YnabMcpError::config(
                "Keyring is not available on this system",
                "Install and configure a keyring service (gnome-keyring, kwallet, or Windows Credential Manager) or set USE_KEYRING=false to use environment variables"
            ))?;

        match entry.delete_credential() {
            Ok(()) => {
                info!("Token deleted from keyring");
                Ok(())
            }
            Err(KeyringError::NoEntry) => {
                debug!("No token found in keyring to delete");
                Ok(())
            }
            Err(e) => Err(YnabMcpError::authentication(
                &format!("Failed to delete token from keyring: {}", e),
                "Check that keyring service is accessible and the token exists. The keyring may be locked or the service may be unavailable."
            )),
        }
    }

    /// Check if keyring is available
    pub fn is_keyring_available(&self) -> bool {
        self.keyring_entry.is_some()
    }

    /// Get the current storage method being used
    pub fn get_storage_method(&self) -> StorageMethod {
        if self.keyring_entry.is_some() {
            // Check if there's actually a token in the keyring
            if let Some(ref entry) = self.keyring_entry {
                match entry.get_password() {
                    Ok(_) => StorageMethod::Keyring,
                    Err(_) => StorageMethod::EnvironmentVariable,
                }
            } else {
                StorageMethod::EnvironmentVariable
            }
        } else {
            StorageMethod::EnvironmentVariable
        }
    }

    /// Provide setup guidance for users
    pub fn setup_guidance(&self) -> Vec<String> {
        let mut guidance = Vec::new();

        if !self.is_keyring_available() {
            guidance.push("⚠️  Keyring unavailable - using environment variables".to_string());
            guidance.push(
                "   For enhanced security, ensure your system supports keyring storage".to_string(),
            );
        } else if self.get_storage_method() == StorageMethod::EnvironmentVariable {
            guidance.push(
                "💡 You can enhance security by storing your token in the system keyring:"
                    .to_string(),
            );
            guidance.push("   1. Run: ynab-mcp --store-token".to_string());
            guidance.push("   2. Remove YNAB_API_TOKEN from environment variables".to_string());
        } else {
            guidance.push("✅ Token is securely stored in system keyring".to_string());
        }

        guidance
    }
}

/// Represents the method used to store the token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageMethod {
    /// Token stored in OS-native keyring (most secure)
    Keyring,
    /// Token stored in environment variables (less secure)
    EnvironmentVariable,
}

impl fmt::Display for StorageMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageMethod::Keyring => write!(f, "System Keyring"),
            StorageMethod::EnvironmentVariable => write!(f, "Environment Variables"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = YnabApiToken::new("test-token-123".to_string());
        assert_eq!(token.expose_secret(), "test-token-123");
        assert!(!token.is_empty());
    }

    #[test]
    fn test_empty_token() {
        let token = YnabApiToken::new("".to_string());
        assert!(token.is_empty());
        assert!(token.validate().is_err());
    }

    #[test]
    fn test_token_validation() {
        let valid_token = YnabApiToken::new("sk_1234567890abcdef".to_string());
        assert!(valid_token.validate().is_ok());

        let short_token = YnabApiToken::new("short".to_string());
        assert!(short_token.validate().is_err());
    }

    #[test]
    fn test_token_debug_format() {
        let token = YnabApiToken::new("secret-token".to_string());
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret-token"));
    }

    #[test]
    fn test_token_display_format() {
        let token = YnabApiToken::new("secret-token".to_string());
        let display_str = format!("{}", token);
        assert_eq!(display_str, "[REDACTED TOKEN]");
    }

    #[test]
    fn test_token_serialization() {
        let token = YnabApiToken::new("secret-token".to_string());
        let json = serde_json::to_string(&token).unwrap();
        assert_eq!(json, "\"[REDACTED]\"");
    }

    #[test]
    fn test_secrets_config_default() {
        let config = SecretsConfig::default();
        assert!(config.use_keyring);
        assert!(config.auto_migrate);
        assert!(!config.require_secure_storage);
    }

    #[test]
    fn test_storage_method_display() {
        assert_eq!(format!("{}", StorageMethod::Keyring), "System Keyring");
        assert_eq!(
            format!("{}", StorageMethod::EnvironmentVariable),
            "Environment Variables"
        );
    }

    #[test]
    fn test_is_test_token() {
        // Test various test patterns
        assert!(YnabApiToken::is_test_token("test-token"));
        assert!(YnabApiToken::is_test_token("dummy_token"));
        assert!(YnabApiToken::is_test_token("PLACEHOLDER"));
        assert!(YnabApiToken::is_test_token("ExampleToken"));
        assert!(YnabApiToken::is_test_token("fake-key"));
        assert!(YnabApiToken::is_test_token("demo123"));
        assert!(YnabApiToken::is_test_token("sample_api_key"));
        assert!(YnabApiToken::is_test_token("mock-token"));
        assert!(YnabApiToken::is_test_token("ynab_api_token"));
        assert!(YnabApiToken::is_test_token("your_token_here"));
        assert!(YnabApiToken::is_test_token("xxxxx"));

        // Test repeated characters
        assert!(YnabApiToken::is_test_token("aaaaaaa"));
        assert!(YnabApiToken::is_test_token("1111111"));
        assert!(YnabApiToken::is_test_token("00000"));

        // Test sequential patterns
        assert!(YnabApiToken::is_test_token("abcdefghijklmnopqrstuvwxyz"));
        assert!(YnabApiToken::is_test_token("123456789"));

        // Test valid-looking tokens (should not be test tokens)
        assert!(!YnabApiToken::is_test_token("sk_1a2b3c4d5e6f7890abcdef"));
        assert!(!YnabApiToken::is_test_token(
            "real_api_key_with_actual_randomness"
        ));
        assert!(!YnabApiToken::is_test_token("abc123xyz789"));
    }

    #[test]
    fn test_test_token_validation_blocked() {
        std::env::set_var("YNAB_BLOCK_TEST_TOKENS", "true");

        let test_token = YnabApiToken::new("test-token".to_string());
        let result = test_token.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Test/placeholder token detected"));

        std::env::remove_var("YNAB_BLOCK_TEST_TOKENS");
    }

    #[test]
    fn test_test_token_validation_allowed() {
        std::env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");

        let test_token = YnabApiToken::new("test-token-long-enough".to_string());
        let result = test_token.validate();
        assert!(result.is_ok()); // Should be allowed but warn

        std::env::remove_var("YNAB_BLOCK_TEST_TOKENS");
    }

    #[test]
    fn test_test_token_validation_no_env() {
        std::env::remove_var("YNAB_BLOCK_TEST_TOKENS");

        let test_token = YnabApiToken::new("test-token-long-enough".to_string());
        let result = test_token.validate();
        assert!(result.is_ok()); // Should be allowed with warning
    }

    #[test]
    fn test_secrets_config_custom() {
        let config = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: true,
        };

        assert!(!config.use_keyring);
        assert!(!config.auto_migrate);
        assert!(config.require_secure_storage);
    }

    #[test]
    fn test_get_keyring_info() {
        let info = SecretsManager::get_keyring_info();
        assert!(!info.is_empty());

        // Test platform-specific behavior
        if cfg!(target_os = "macos") {
            assert!(info.contains("macOS Keychain"));
        } else if cfg!(target_os = "windows") {
            assert!(info.contains("Windows Credential Manager"));
        } else if cfg!(target_os = "linux") {
            assert!(info.contains("Linux Kernel Keyring"));
        } else {
            assert!(info.contains("Generic keyring backend"));
        }
    }

    #[test]
    fn test_secrets_manager_creation_with_keyring_disabled() {
        let config = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        let manager = SecretsManager::new(config);
        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert!(!manager.is_keyring_available());
        assert_eq!(
            manager.get_storage_method(),
            StorageMethod::EnvironmentVariable
        );
    }

    #[test]
    fn test_secrets_manager_creation_with_required_secure_storage() {
        let config = SecretsConfig {
            use_keyring: true,
            auto_migrate: false,
            require_secure_storage: true,
        };

        // This may succeed or fail depending on system keyring availability
        let manager_result = SecretsManager::new(config);
        // Don't assert success/failure as it's system dependent
        let _ = manager_result;
    }

    #[test]
    fn test_token_deserialization_blocked() {
        // Test that deserialization is blocked
        let json = "\"some-token\"";
        let result: std::result::Result<YnabApiToken, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be deserialized"));
    }

    #[test]
    fn test_token_short_validation_error() {
        let short_token = YnabApiToken::new("short".to_string());
        let result = short_token.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("too short"));
        assert!(error_msg.contains("5 characters"));
    }

    #[test]
    fn test_token_empty_validation_error() {
        let empty_token = YnabApiToken::new("".to_string());
        let result = empty_token.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("missing or empty"));
    }

    #[test]
    fn test_storage_method_equality() {
        assert_eq!(StorageMethod::Keyring, StorageMethod::Keyring);
        assert_eq!(
            StorageMethod::EnvironmentVariable,
            StorageMethod::EnvironmentVariable
        );
        assert_ne!(StorageMethod::Keyring, StorageMethod::EnvironmentVariable);
    }

    #[test]
    fn test_secrets_config_debug() {
        let config = SecretsConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SecretsConfig"));
        assert!(debug_str.contains("use_keyring"));
    }

    #[test]
    fn test_token_clone() {
        let token = YnabApiToken::new("test-token".to_string());
        let cloned = token.clone();
        assert_eq!(token.expose_secret(), cloned.expose_secret());
    }

    #[test]
    fn test_repeated_char_detection_edge_cases() {
        // Test short strings that shouldn't trigger repeated char detection
        assert!(!YnabApiToken::is_test_token("aa"));
        assert!(!YnabApiToken::is_test_token("aaa"));
        assert!(!YnabApiToken::is_test_token("aaaa"));

        // Test exactly 5 chars (should trigger)
        assert!(YnabApiToken::is_test_token("aaaaa"));

        // Test empty string
        assert!(!YnabApiToken::is_test_token(""));
    }

    #[test]
    fn test_env_var_parsing_edge_cases() {
        // Test different env var values for blocking
        std::env::set_var("YNAB_BLOCK_TEST_TOKENS", "TRUE");
        let token = YnabApiToken::new("test-token-long".to_string());
        assert!(token.validate().is_err());

        std::env::set_var("YNAB_BLOCK_TEST_TOKENS", "1");
        assert!(token.validate().is_err());

        std::env::set_var("YNAB_BLOCK_TEST_TOKENS", "false");
        assert!(token.validate().is_ok());

        std::env::set_var("YNAB_BLOCK_TEST_TOKENS", "random_value");
        assert!(token.validate().is_ok());

        std::env::remove_var("YNAB_BLOCK_TEST_TOKENS");
    }
}
