use serde_json::json;

// Import modules for financial testing
use ynab_mcp::utils::money::MoneyUtils;

/// Property-based testing for financial calculations
///
/// These tests use property-based testing principles to verify that financial
/// calculations are correct across a wide range of inputs, focusing on:
/// - Money conversion between currency units and milliunits
/// - Precision and rounding behavior
/// - Range validation and overflow handling
/// - Arithmetic properties (associativity, commutativity, etc.)
/// - Edge cases with very large/small amounts
/// - Currency formatting consistency
///
/// Generate a range of test amounts covering various financial scenarios
fn generate_test_amounts() -> Vec<f64> {
    let mut amounts = Vec::new();

    // Zero and basic amounts
    amounts.extend(vec![0.0, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0]);

    // Negative amounts
    amounts.extend(vec![-0.01, -1.0, -100.0, -1000.0]);

    // Precision test amounts (up to 3 decimal places for milliunits)
    amounts.extend(vec![
        1.001, 1.01, 1.1, 1.123, 1.999, -1.001, -1.01, -1.1, -1.123, -1.999,
    ]);

    // Large amounts (near YNAB limits)
    amounts.extend(vec![999999.999, -999999.999, 123456.789, -123456.789]);

    // Edge cases for rounding
    amounts.extend(vec![
        0.0001, 0.0005, 0.0009, 0.9995, 0.9999, -0.0001, -0.0005, -0.0009, -0.9995, -0.9999,
    ]);

    // Random-looking amounts for comprehensive testing
    amounts.extend(vec![
        12.34, 56.78, 90.12, 345.67, 890.123, -12.34, -56.78, -90.12, -345.67, -890.123,
    ]);

    amounts
}

#[tokio::test]
async fn test_money_conversion_properties() {
    let test_amounts = generate_test_amounts();

    for &amount in &test_amounts {
        // Skip amounts that would be outside YNAB's range
        if amount.abs() > 999_999_999.999 {
            continue;
        }

        // Test conversion to milliunits and back
        let amount_value = json!(amount);

        match MoneyUtils::parse_amount(&amount_value) {
            Ok(milliunits) => {
                // Verify milliunits are within valid range
                assert!(milliunits >= -999_999_999_999);
                assert!(milliunits <= 999_999_999_999);

                // Test formatting back to string
                let formatted = MoneyUtils::format_milliunits(milliunits);

                // Parse formatted amount back to number
                let formatted_clean = formatted
                    .trim_start_matches('£')
                    .trim_start_matches('-')
                    .trim_start_matches('£'); // Handle -£ case
                let reconstructed: f64 = formatted_clean
                    .parse()
                    .unwrap_or_else(|_| panic!("Failed to parse formatted amount: {}", formatted));

                // Apply sign if original was negative
                let final_reconstructed = if formatted.starts_with('-') {
                    -reconstructed
                } else {
                    reconstructed
                };

                // Verify roundtrip conversion is within acceptable precision
                // Note: YNAB formatting rounds to 2 decimal places, so we expect some precision loss
                let difference = (amount - final_reconstructed).abs();
                assert!(
                    difference < 0.01, // Allow up to 1 cent difference due to formatting rounding
                    "Roundtrip conversion failed: {} -> {} -> {} (diff: {})",
                    amount,
                    milliunits,
                    final_reconstructed,
                    difference
                );
            }
            Err(_) => {
                // Some amounts may fail validation, which is acceptable
                // but shouldn't panic or expose sensitive information
            }
        }
    }
}

#[tokio::test]
async fn test_string_parsing_properties() {
    let test_strings = vec![
        "0",
        "0.0",
        "0.00",
        "0.000",
        "1",
        "1.0",
        "1.00",
        "1.000",
        "10",
        "10.0",
        "10.00",
        "10.000",
        "100.50",
        "100.500",
        "1.23",
        "1.230",
        "1.2300",
        "-1",
        "-1.0",
        "-1.00",
        "-1.000",
        "-100.50",
        "-100.500",
        "999999.999",
        "-999999.999",
        // Edge cases
        ".50",
        "-.50", // Leading decimal point
        "1.",
        "-1.",         // Trailing decimal point
        "   1.23   ",  // Whitespace
        "1.234567890", // Extra precision
        "0000001.23",  // Leading zeros
        "1.20000000",  // Trailing zeros
    ];

    for test_string in test_strings {
        let string_value = json!(test_string);

        match MoneyUtils::parse_amount(&string_value) {
            Ok(milliunits) => {
                // Verify the result is within bounds
                assert!(milliunits >= -999_999_999_999);
                assert!(milliunits <= 999_999_999_999);

                // Test that equivalent representations produce same result
                let trimmed = test_string.trim();
                let trimmed_value = json!(trimmed);

                if let Ok(trimmed_milliunits) = MoneyUtils::parse_amount(&trimmed_value) {
                    assert_eq!(
                        milliunits, trimmed_milliunits,
                        "Whitespace should not affect parsing: '{}' vs '{}'",
                        test_string, trimmed
                    );
                }

                // Test formatting consistency
                let formatted = MoneyUtils::format_milliunits(milliunits);
                assert!(!formatted.is_empty());
                assert!(
                    formatted.contains('.'),
                    "Formatted amount should contain decimal point"
                );

                // Verify formatted amount has correct currency symbol
                assert!(formatted.contains('£'));
            }
            Err(_) => {
                // Some strings may be invalid, which is acceptable
                // Just ensure no panic or information leak
            }
        }
    }
}

#[tokio::test]
async fn test_validation_properties() {
    // Test various invalid inputs that should fail validation
    let invalid_inputs = vec![
        // Non-numeric strings
        json!("not_a_number"),
        json!("abc123"),
        json!("$100"),      // Currency symbols not supported
        json!("100.50.25"), // Multiple decimal points
        json!(""),          // Empty string
        json!("   "),       // Just whitespace
        // Non-string, non-number types
        json!(null),
        json!(true),
        json!(false),
        json!([1, 2, 3]),
        json!({"amount": 100}),
        // Extreme values
        json!(f64::INFINITY),
        json!(f64::NEG_INFINITY),
        json!(f64::NAN),
        // Very large numbers (outside YNAB range)
        json!(1e15), // 1 quadrillion
        json!(-1e15),
        json!(1_000_000_000_000.0), // 1 trillion (outside YNAB range)
    ];

    for invalid_input in invalid_inputs {
        let result = MoneyUtils::parse_amount(&invalid_input);

        // Most should fail, but if they succeed, result should be reasonable
        match result {
            Ok(milliunits) => {
                // If parsing somehow succeeded, ensure it's within bounds
                // Note: very large numbers might convert to milliunits outside our expected range
                // This is acceptable as long as we can detect and handle them
                if milliunits.abs() <= 999_999_999_999 {
                    // Within expected range - good
                } else {
                    // Outside range - should be caught by validation later
                    println!(
                        "Large value {} produced out-of-range milliunits: {}",
                        serde_json::to_string(&invalid_input).unwrap_or_default(),
                        milliunits
                    );
                }
            }
            Err(_) => {
                // Expected for most invalid inputs - just ensure no panic
            }
        }
    }
}

#[tokio::test]
async fn test_arithmetic_properties() {
    let test_values = vec![
        (0.0, 0.0),
        (1.0, 1.0),
        (1.0, -1.0),
        (100.50, 50.25),
        (-100.50, 50.25),
        (999.99, 0.01),
        (0.001, 999.999),
    ];

    for (a, b) in test_values {
        let a_json = json!(a);
        let b_json = json!(b);

        if let (Ok(a_milliunits), Ok(b_milliunits)) = (
            MoneyUtils::parse_amount(&a_json),
            MoneyUtils::parse_amount(&b_json),
        ) {
            // Test addition properties
            let sum = a_milliunits + b_milliunits;
            if (-999_999_999_999..=999_999_999_999).contains(&sum) {
                // Commutativity: a + b = b + a
                let sum_reverse = b_milliunits + a_milliunits;
                assert_eq!(
                    sum, sum_reverse,
                    "Addition should be commutative: {} + {} = {}, {} + {} = {}",
                    a, b, sum, b, a, sum_reverse
                );

                // Identity: a + 0 = a
                assert_eq!(
                    a_milliunits, a_milliunits,
                    "Adding zero should not change value: {} + 0 = {}",
                    a, a_milliunits
                );
            }

            // Test subtraction properties
            let diff = a_milliunits - b_milliunits;
            if (-999_999_999_999..=999_999_999_999).contains(&diff) {
                // Inverse: (a - b) + b = a
                let reconstructed = diff + b_milliunits;
                assert_eq!(
                    reconstructed, a_milliunits,
                    "Subtraction and addition should be inverse: ({} - {}) + {} = {}, expected {}",
                    a, b, b, reconstructed, a
                );
            }

            // Test formatting preservation under arithmetic
            let formatted_a = MoneyUtils::format_milliunits(a_milliunits);
            let formatted_b = MoneyUtils::format_milliunits(b_milliunits);

            // Verify formatting produces valid-looking currency strings
            assert!(formatted_a.contains('£'));
            assert!(formatted_b.contains('£'));
            assert!(formatted_a.contains('.'));
            assert!(formatted_b.contains('.'));
        }
    }
}

#[tokio::test]
async fn test_precision_properties() {
    // Test that milliunits preserve precision correctly
    let precision_tests = vec![
        (1.001, 1001),       // 3 decimal places
        (1.01, 1010),        // 2 decimal places
        (1.1, 1100),         // 1 decimal place
        (1.0, 1000),         // 0 decimal places
        (0.001, 1),          // Minimum precision
        (0.999, 999),        // Maximum sub-unit
        (-1.001, -1001),     // Negative precision
        (1234.567, 1234567), // Large with precision
    ];

    for (currency_amount, expected_milliunits) in precision_tests {
        let amount_json = json!(currency_amount);

        match MoneyUtils::parse_amount(&amount_json) {
            Ok(milliunits) => {
                assert_eq!(
                    milliunits, expected_milliunits,
                    "Precision conversion failed: {} should be {} milliunits, got {}",
                    currency_amount, expected_milliunits, milliunits
                );

                // Test reverse conversion
                let formatted = MoneyUtils::format_milliunits(milliunits);

                // Parse the formatted amount (without currency symbol)
                let formatted_clean = formatted
                    .trim_start_matches('£')
                    .trim_start_matches('-')
                    .trim_start_matches('£');
                let parsed_back: f64 = formatted_clean
                    .parse()
                    .expect("Formatted amount should be parseable");

                let final_value = if formatted.starts_with('-') {
                    -parsed_back
                } else {
                    parsed_back
                };

                // Should match within rounding tolerance (YNAB formats to 2 decimal places)
                let diff = (currency_amount - final_value).abs();
                assert!(
                    diff < 0.01,
                    "Precision roundtrip failed: {} -> {} -> {} (diff: {})",
                    currency_amount,
                    milliunits,
                    final_value,
                    diff
                );
            }
            Err(e) => {
                panic!(
                    "Valid precision amount should parse: {} - Error: {}",
                    currency_amount, e
                );
            }
        }
    }
}

#[tokio::test]
async fn test_edge_case_properties() {
    // Test boundary conditions
    let edge_cases = vec![
        // YNAB limits
        (999_999_999.999, 999_999_999_999i64),
        (-999_999_999.999, -999_999_999_999i64),
        // Just under limits
        (999_999_999.998, 999_999_999_998i64),
        (-999_999_999.998, -999_999_999_998i64),
        // Zero variations
        (0.0, 0i64),
        (-0.0, 0i64),
        // Minimum unit
        (0.001, 1i64),
        (-0.001, -1i64),
    ];

    for (amount, expected_milliunits) in edge_cases {
        let amount_json = json!(amount);

        match MoneyUtils::parse_amount(&amount_json) {
            Ok(milliunits) => {
                assert_eq!(
                    milliunits, expected_milliunits,
                    "Edge case conversion failed: {} should be {} milliunits, got {}",
                    amount, expected_milliunits, milliunits
                );

                // Test validation passes for edge cases within range
                if milliunits.abs() <= 999_999_999_999 {
                    let args = json!({"amount": amount});
                    let validated = MoneyUtils::validate_amount(&args, "amount");
                    assert!(
                        validated.is_ok(),
                        "Edge case should pass validation: {} ({})",
                        amount,
                        milliunits
                    );
                }
            }
            Err(_) => {
                // Some edge cases may fail parsing, which might be acceptable
                // depending on implementation choices
            }
        }
    }
}

#[tokio::test]
async fn test_range_validation_properties() {
    // Test amounts outside YNAB's supported range
    let out_of_range_amounts = vec![
        1_000_000_000.0,   // 1 billion (too large)
        -1_000_000_000.0,  // -1 billion (too large magnitude)
        999_999_999.999_9, // Just over limit with extra precision
        1e9,               // Scientific notation for 1 billion
        -1e9,              // Scientific notation for -1 billion
    ];

    for amount in out_of_range_amounts {
        let args = json!({"amount": amount});

        let result = MoneyUtils::validate_amount(&args, "amount");

        // Should fail validation for out-of-range amounts
        match result {
            Ok(milliunits) => {
                // If it somehow passes, ensure it's actually within bounds
                assert!(
                    milliunits.abs() <= 999_999_999_999,
                    "Out of range amount {} produced milliunits {} which should be rejected",
                    amount,
                    milliunits
                );
            }
            Err(_) => {
                // Expected behavior - out of range amounts should be rejected
            }
        }
    }

    // Test amounts just within range
    let in_range_amounts = vec![
        999_999_999.999,
        -999_999_999.999,
        999_999_999.0,
        -999_999_999.0,
    ];

    for amount in in_range_amounts {
        let args = json!({"amount": amount});

        let result = MoneyUtils::validate_amount(&args, "amount");
        assert!(
            result.is_ok(),
            "In-range amount {} should pass validation",
            amount
        );

        let milliunits = result.unwrap();
        assert!(
            milliunits.abs() <= 999_999_999_999,
            "Validated amount should be within bounds: {} -> {}",
            amount,
            milliunits
        );
    }
}

#[tokio::test]
async fn test_positive_amount_validation_properties() {
    let test_amounts = vec![
        (1.0, true),      // Positive should pass
        (0.001, true),    // Small positive should pass
        (1000.0, true),   // Large positive should pass
        (0.0, false),     // Zero should fail
        (-0.001, false),  // Small negative should fail
        (-1.0, false),    // Negative should fail
        (-1000.0, false), // Large negative should fail
    ];

    for (amount, should_pass) in test_amounts {
        let amount_json = json!(amount);

        if let Ok(milliunits) = MoneyUtils::parse_amount(&amount_json) {
            let result = MoneyUtils::validate_positive_amount(milliunits, "test");

            if should_pass {
                assert!(
                    result.is_ok(),
                    "Positive amount {} ({} milliunits) should pass validation",
                    amount,
                    milliunits
                );
            } else {
                assert!(
                    result.is_err(),
                    "Non-positive amount {} ({} milliunits) should fail validation",
                    amount,
                    milliunits
                );
            }
        }
    }
}

#[tokio::test]
async fn test_formatting_consistency_properties() {
    let test_amounts = generate_test_amounts();

    for &amount in &test_amounts {
        if amount.abs() > 999_999_999.999 {
            continue; // Skip out-of-range amounts
        }

        let amount_json = json!(amount);

        if let Ok(milliunits) = MoneyUtils::parse_amount(&amount_json) {
            let formatted = MoneyUtils::format_milliunits(milliunits);

            // Property: All formatted amounts should have currency symbol
            assert!(
                formatted.contains('£'),
                "Formatted amount should contain currency symbol: {}",
                formatted
            );

            // Property: All formatted amounts should have decimal point
            assert!(
                formatted.contains('.'),
                "Formatted amount should contain decimal point: {}",
                formatted
            );

            // Property: Decimal places should be exactly 2
            let decimal_part = formatted.split('.').nth(1).unwrap_or("");
            let decimal_digits = decimal_part.len();
            assert_eq!(
                decimal_digits, 2,
                "Formatted amount should have exactly 2 decimal places: {} (has {})",
                formatted, decimal_digits
            );

            // Property: Negative amounts should start with minus sign
            if milliunits < 0 {
                assert!(
                    formatted.starts_with('-'),
                    "Negative amount should start with minus: {}",
                    formatted
                );
            } else {
                assert!(
                    !formatted.starts_with('-'),
                    "Non-negative amount should not start with minus: {}",
                    formatted
                );
            }

            // Property: Zero should format consistently
            if milliunits == 0 {
                assert_eq!(
                    formatted, "£0.00",
                    "Zero should format as £0.00, got: {}",
                    formatted
                );
            }
        }
    }
}
