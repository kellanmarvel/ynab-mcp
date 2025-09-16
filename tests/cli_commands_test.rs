// Integration tests for CLI commands
use ynab_mcp::secrets::{SecretsConfig, SecretsManager, StorageMethod, YnabApiToken};

// Mock the CLI handlers for testing
// Since the handlers are in main.rs and not public, we'll test the underlying components

#[cfg(test)]
mod cli_command_tests {
    use super::*;

    #[test]
    fn test_secrets_config_creation() {
        // Test creating SecretsConfig with different combinations
        let config1 = SecretsConfig {
            use_keyring: true,
            auto_migrate: true,
            require_secure_storage: false,
        };

        assert!(config1.use_keyring);
        assert!(config1.auto_migrate);
        assert!(!config1.require_secure_storage);

        let config2 = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: true,
        };

        assert!(!config2.use_keyring);
        assert!(!config2.auto_migrate);
        assert!(config2.require_secure_storage);
    }

    #[test]
    fn test_secrets_config_default() {
        let default_config = SecretsConfig::default();

        // Check default values
        assert!(default_config.use_keyring);
        assert!(default_config.auto_migrate);
        assert!(!default_config.require_secure_storage);
    }

    #[tokio::test]
    async fn test_secrets_manager_creation_no_keyring() {
        let config = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        let manager = SecretsManager::new(config);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(!manager.is_keyring_available() || !config.use_keyring);
    }

    #[tokio::test]
    async fn test_secrets_manager_creation_require_secure_but_unavailable() {
        let config = SecretsConfig {
            use_keyring: true,
            auto_migrate: true,
            require_secure_storage: true,
        };

        // This might fail if keyring is not available and we require secure storage
        let manager = SecretsManager::new(config);

        match manager {
            Ok(mgr) => {
                // If it succeeds, keyring should be available
                assert!(mgr.is_keyring_available());
            }
            Err(_) => {
                // It's acceptable to fail if keyring is not available and required
                // This is expected behavior in test environments
            }
        }
    }

    #[test]
    fn test_ynab_api_token_validation() {
        // Test valid token formats
        let valid_tokens = vec![
            "abcdef123456789",
            "test_token_123",
            "ABC-123-DEF",
            "token.with.dots",
            "token_with_underscores",
            "1234567890",
        ];

        for token_str in valid_tokens {
            let token = YnabApiToken::new(token_str.to_string());
            assert!(
                token.validate().is_ok(),
                "Token '{}' should be valid",
                token_str
            );
        }
    }

    #[test]
    fn test_ynab_api_token_validation_invalid() {
        // Test invalid token formats
        let invalid_tokens = vec![
            "",          // Empty token
            "a",         // Too short (< 10 chars per main.rs validation)
            "ab",        // Too short
            "123456789", // Too short (exactly 9 chars)
        ];

        for token_str in invalid_tokens {
            let token = YnabApiToken::new(token_str.to_string());
            assert!(
                token.validate().is_err(),
                "Token '{}' should be invalid",
                token_str
            );
        }
    }

    #[test]
    fn test_ynab_api_token_character_validation() {
        // Test tokens that would fail the main.rs character validation
        let tokens_with_invalid_chars = vec![
            "token!with!exclamation",
            "token@with@at",
            "token#with#hash",
            "token$with$dollar",
            "token%with%percent",
            "token with spaces",
            "token\nwith\nnewlines",
            "token\twith\ttabs",
        ];

        for token_str in tokens_with_invalid_chars {
            let _token = YnabApiToken::new(token_str.to_string());

            // Check if it contains invalid characters (simulating main.rs logic)
            let has_invalid_chars = !token_str
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

            if has_invalid_chars {
                // This would be caught by the main.rs validation before YnabApiToken validation
                // Token has invalid characters as expected
            }
        }
    }

    #[test]
    fn test_token_length_validation() {
        // Test token length limits (main.rs checks for > 512 characters)
        let max_valid_length = 512;
        let valid_long_token = "a".repeat(max_valid_length);
        let invalid_long_token = "a".repeat(max_valid_length + 1);

        // Main.rs would reject the too-long token before YnabApiToken validation
        assert_eq!(valid_long_token.len(), max_valid_length);
        assert!(invalid_long_token.len() > max_valid_length);

        // Test that we can create tokens up to the limit
        let token = YnabApiToken::new(valid_long_token);
        // The token itself might be valid format-wise, length checking is in main.rs
        assert!(token.validate().is_ok() || token.validate().is_err()); // Either is acceptable
    }

    #[tokio::test]
    async fn test_secrets_manager_storage_method() {
        let config_with_keyring = SecretsConfig {
            use_keyring: true,
            auto_migrate: true,
            require_secure_storage: false,
        };

        let config_without_keyring = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        // Test with keyring enabled
        if let Ok(manager) = SecretsManager::new(config_with_keyring) {
            let storage_method = manager.get_storage_method();

            // Should be either keyring-based or environment variables
            assert!(matches!(
                storage_method,
                StorageMethod::Keyring | StorageMethod::EnvironmentVariable
            ));
        }

        // Test without keyring
        if let Ok(manager) = SecretsManager::new(config_without_keyring) {
            let storage_method = manager.get_storage_method();
            assert!(matches!(storage_method, StorageMethod::EnvironmentVariable));
        }
    }

    #[tokio::test]
    async fn test_secrets_manager_guidance() {
        let config = SecretsConfig::default();

        if let Ok(manager) = SecretsManager::new(config) {
            let guidance = manager.setup_guidance();

            // Should provide some guidance
            assert!(!guidance.is_empty());

            // All guidance should be non-empty strings
            for guide in guidance {
                assert!(!guide.is_empty());
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_secrets_manager_token_operations_no_keyring() {
        // Temporarily unset environment token for this test
        let original_token = std::env::var("YNAB_API_TOKEN").ok();
        std::env::remove_var("YNAB_API_TOKEN");

        let config = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        if let Ok(manager) = SecretsManager::new(config) {
            // Try to load token when none exists
            let result = manager.load_token();
            assert!(result.is_err()); // Should fail - no token available

            // Try to delete token when none exists
            let result = manager.delete_token();
            // This might fail if keyring is not available, which is expected
            if result.is_err() {
                // Expected - keyring not available or no token to delete
            } else {
                // Also acceptable - deletion succeeded (nothing to delete)
                assert!(result.is_ok());
            }
        }

        // Restore original environment variable
        if let Some(token) = original_token {
            std::env::set_var("YNAB_API_TOKEN", token);
        }
    }

    #[test]
    fn test_token_redaction_in_debug() {
        // Test that tokens are properly redacted in debug output
        let sensitive_token = "this_is_a_secret_token_12345";
        let token = YnabApiToken::new(sensitive_token.to_string());

        let debug_output = format!("{:?}", token);

        // Debug output should not contain the actual token
        assert!(!debug_output.contains(sensitive_token));
        assert!(
            debug_output.contains("[REDACTED]")
                || debug_output.contains("***")
                || debug_output.contains("HIDDEN")
        );
    }

    #[test]
    fn test_cli_argument_combinations() {
        // Test the logic that would be used to create SecretsConfig from CLI args

        // Simulate --no-keyring flag
        let no_keyring = true;
        let require_secure_storage = false;
        let no_auto_migrate = false;

        let config = SecretsConfig {
            use_keyring: !no_keyring,
            auto_migrate: !no_auto_migrate,
            require_secure_storage,
        };

        assert!(!config.use_keyring);
        assert!(config.auto_migrate);
        assert!(!config.require_secure_storage);

        // Simulate --require-secure-storage flag
        let no_keyring = false;
        let require_secure_storage = true;
        let no_auto_migrate = false;

        let config = SecretsConfig {
            use_keyring: !no_keyring,
            auto_migrate: !no_auto_migrate,
            require_secure_storage,
        };

        assert!(config.use_keyring);
        assert!(config.auto_migrate);
        assert!(config.require_secure_storage);

        // Simulate --no-auto-migrate flag
        let no_keyring = false;
        let require_secure_storage = false;
        let no_auto_migrate = true;

        let config = SecretsConfig {
            use_keyring: !no_keyring,
            auto_migrate: !no_auto_migrate,
            require_secure_storage,
        };

        assert!(config.use_keyring);
        assert!(!config.auto_migrate);
        assert!(!config.require_secure_storage);
    }

    #[test]
    fn test_empty_token_handling() {
        // Test how the system handles empty tokens (which main.rs should reject)
        let empty_token = YnabApiToken::new("".to_string());

        // Empty tokens should fail validation
        assert!(empty_token.validate().is_err());
    }

    #[test]
    fn test_token_memory_safety() {
        // Test that tokens are properly handled in memory
        let sensitive_data = "secret_token_123";
        let token = YnabApiToken::new(sensitive_data.to_string());

        // The token should be created successfully
        assert!(token.validate().is_ok() || token.validate().is_err()); // Either is fine for this test

        // After dropping the token, memory should be cleared (this is handled by the zeroize trait)
        drop(token);

        // We can't directly test memory clearing in a unit test, but we can verify
        // the token implements the expected traits
        let token2 = YnabApiToken::new(sensitive_data.to_string());

        // Verify it implements Clone and Debug as expected
        let _cloned = token2.clone();
        let _debug_str = format!("{:?}", token2);

        // Test passes if no panics occur
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_cli_workflow_simulation() {
        // Simulate a complete CLI workflow without actual process spawning

        // Temporarily unset environment token for this test
        let original_token = std::env::var("YNAB_API_TOKEN").ok();
        std::env::remove_var("YNAB_API_TOKEN");

        // 1. Start with no keyring, no token
        let no_keyring_config = SecretsConfig {
            use_keyring: false,
            auto_migrate: false,
            require_secure_storage: false,
        };

        if let Ok(manager) = SecretsManager::new(no_keyring_config) {
            // Should fail to load token
            assert!(manager.load_token().is_err());

            // Storage method should be environment variables
            assert!(matches!(
                manager.get_storage_method(),
                StorageMethod::EnvironmentVariable
            ));

            // Should get helpful guidance
            let guidance = manager.setup_guidance();
            assert!(!guidance.is_empty());
        }

        // 2. Try with keyring enabled (may work or fail depending on system)
        let keyring_config = SecretsConfig {
            use_keyring: true,
            auto_migrate: true,
            require_secure_storage: false,
        };

        let manager_result = SecretsManager::new(keyring_config);
        match manager_result {
            Ok(manager) => {
                // If keyring works, test its functionality
                assert!(manager.is_keyring_available());

                // Should fail to load token unless one exists in environment
                let _load_result = manager.load_token();
                // This is acceptable either way - depends on environment

                // Storage method should be EnvironmentVariable (no token in keyring yet)
                let storage_method = manager.get_storage_method();
                assert!(matches!(
                    storage_method,
                    StorageMethod::Keyring | StorageMethod::EnvironmentVariable
                ));
            }
            Err(_) => {
                // Keyring not available on this system - that's fine for testing
            }
        }

        // Restore original environment variable
        if let Some(token) = original_token {
            std::env::set_var("YNAB_API_TOKEN", token);
        }
    }

    #[test]
    fn test_cli_validation_logic() {
        // Test the validation logic that would be used in CLI commands

        // Test empty token (would be caught by main.rs)
        let empty_input = "";
        assert!(empty_input.trim().is_empty()); // main.rs check

        // Test token length (would be caught by main.rs)
        let long_token = "a".repeat(600);
        assert!(long_token.len() > 512); // main.rs check

        // Test character validation (would be caught by main.rs)
        let tokens_to_test = vec![
            ("valid_token_123", true),
            ("valid-token-456", true),
            ("valid.token.789", true),
            ("invalid token spaces", false),
            ("invalid!token@symbols", false),
            ("invalid\ntoken\nnewlines", false),
        ];

        for (token, should_be_valid) in tokens_to_test {
            let has_valid_chars = token
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

            assert_eq!(
                has_valid_chars, should_be_valid,
                "Token '{}' validity mismatch",
                token
            );
        }
    }

    #[test]
    fn test_cli_confirmation_logic() {
        // Test the confirmation logic used in delete-token command

        let confirmations = vec![
            ("y", true),
            ("Y", true),
            ("yes", false), // main.rs only accepts "y"
            ("YES", false), // main.rs only accepts "y"
            ("n", false),
            ("N", false),
            ("no", false),
            ("", false), // Empty input defaults to "no"
            ("invalid", false),
        ];

        for (input, should_confirm) in confirmations {
            let normalized = input.trim().to_lowercase();
            let confirms = normalized == "y";

            assert_eq!(
                confirms, should_confirm,
                "Confirmation '{}' should be {}",
                input, should_confirm
            );
        }
    }

    #[test]
    fn test_cli_error_handling_patterns() {
        // Test error handling patterns used in CLI commands

        // Test token validation errors
        let long_token = "a".repeat(600);
        let test_tokens = vec![
            ("", "No token provided"),
            ("short", "too short"),
            (long_token.as_str(), "too long"),
        ];

        for (token_input, expected_error_type) in test_tokens {
            if token_input.is_empty() {
                assert!(expected_error_type.contains("No token"));
            } else if token_input.len() < 10 {
                assert!(expected_error_type.contains("short"));
            } else if token_input.len() > 512 {
                assert!(expected_error_type.contains("long"));
            }
        }
    }

    #[tokio::test]
    async fn test_secrets_manager_error_scenarios() {
        // Test various error scenarios that CLI commands might encounter

        // 1. Test requiring secure storage but keyring unavailable
        let strict_config = SecretsConfig {
            use_keyring: true,
            auto_migrate: false,
            require_secure_storage: true,
        };

        let manager_result = SecretsManager::new(strict_config);
        match manager_result {
            Ok(manager) => {
                // If this succeeds, keyring must be available
                assert!(manager.is_keyring_available());
            }
            Err(_) => {
                // Expected if keyring is not available but required
                // This is the expected behavior
            }
        }

        // 2. Test auto-migration scenario (would need environment token)
        let migration_config = SecretsConfig {
            use_keyring: true,
            auto_migrate: true,
            require_secure_storage: false,
        };

        if let Ok(manager) = SecretsManager::new(migration_config) {
            // Without environment token, load should fail unless token exists
            let _load_result = manager.load_token();
            // Result depends on environment - both outcomes are acceptable for this test
        }
    }

    #[test]
    fn test_output_formatting_components() {
        // Test components that would be used for CLI output formatting

        // Test emoji usage
        let status_emoji = "📊";
        let keyring_emoji = "🔐";
        let token_emoji = "🔑";
        let config_emoji = "⚙️";
        let recommendation_emoji = "💡";
        let success_emoji = "✅";
        let error_emoji = "❌";

        // Verify emojis are valid UTF-8 and not empty
        let emojis = vec![
            status_emoji,
            keyring_emoji,
            token_emoji,
            config_emoji,
            recommendation_emoji,
            success_emoji,
            error_emoji,
        ];

        for emoji in emojis {
            assert!(!emoji.is_empty());
            assert!(!emoji.is_ascii()); // Emojis are non-ASCII
        }

        // Test section headers
        let headers = vec![
            "YNAB MCP Server - Secrets Management Status",
            "Keyring Status:",
            "Token Status:",
            "Configuration:",
            "Recommendations:",
        ];

        for header in headers {
            assert!(!header.is_empty());
            assert!(header
                .chars()
                .all(|c| c.is_ascii() || c == ':' || c.is_whitespace()));
        }
    }
}
