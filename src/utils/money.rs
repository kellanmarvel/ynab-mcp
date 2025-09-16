use crate::error::Result;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde_json::Value;
use std::str::FromStr;

/// Centralized money handling utilities for YNAB operations
pub struct MoneyUtils;

impl MoneyUtils {
    /// Convert amount from various formats to YNAB milliunits (1000 milliunits = 1 currency unit)
    pub fn parse_amount(value: &Value) -> Result<i64> {
        match value {
            Value::Number(n) => {
                if let Some(float_val) = n.as_f64() {
                    // Convert from currency units to milliunits
                    Ok((float_val * 1000.0).round() as i64)
                } else if let Some(int_val) = n.as_i64() {
                    // Assume it's already in milliunits if it's an integer
                    Ok(int_val)
                } else {
                    Err(anyhow::anyhow!("Invalid number format").into())
                }
            }
            Value::String(s) => {
                // Try to parse as decimal
                let decimal = Decimal::from_str(s.trim())
                    .map_err(|_| anyhow::anyhow!("Invalid decimal string: {}", s))?;

                // Convert to milliunits
                let milliunits = (decimal * Decimal::from(1000)).round();
                milliunits
                    .to_i64()
                    .ok_or_else(|| anyhow::anyhow!("Amount too large or small").into())
            }
            _ => Err(anyhow::anyhow!("Amount must be a number or string").into()),
        }
    }

    /// Validate amount and convert to milliunits with comprehensive error checking
    pub fn validate_amount(args: &Value, field_name: &str) -> Result<i64> {
        let amount_value = args
            .get(field_name)
            .ok_or_else(|| anyhow::anyhow!("Missing required field: {}", field_name))?;

        let milliunits = Self::parse_amount(amount_value)?;

        // Validate range (YNAB supports amounts from -$999,999,999.999 to $999,999,999.999)
        const MAX_MILLIUNITS: i64 = 999_999_999_999;
        const MIN_MILLIUNITS: i64 = -999_999_999_999;

        if !(MIN_MILLIUNITS..=MAX_MILLIUNITS).contains(&milliunits) {
            return Err(anyhow::anyhow!(
                "Amount {} is outside the valid range (±$999,999,999.999)",
                Self::format_milliunits(milliunits)
            )
            .into());
        }

        Ok(milliunits)
    }

    /// Validate optional amount field
    pub fn validate_optional_amount(args: &Value, field_name: &str) -> Result<Option<i64>> {
        match args.get(field_name) {
            None | Some(Value::Null) => Ok(None),
            Some(_) => Ok(Some(Self::validate_amount(args, field_name)?)),
        }
    }

    /// Format milliunits to human-readable currency string
    pub fn format_milliunits(milliunits: i64) -> String {
        let amount = milliunits as f64 / 1000.0;
        if amount >= 0.0 {
            format!("£{:.2}", amount)
        } else {
            format!("-£{:.2}", amount.abs())
        }
    }

    /// Validate that amount is positive (for transfers, etc.)
    pub fn validate_positive_amount(milliunits: i64, context: &str) -> Result<()> {
        if milliunits <= 0 {
            return Err(anyhow::anyhow!("{} amount must be positive", context).into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_amount_float() {
        let value = json!(123.45);
        assert_eq!(MoneyUtils::parse_amount(&value).unwrap(), 123450);
    }

    #[test]
    fn test_parse_amount_negative() {
        let value = json!(-50.0);
        assert_eq!(MoneyUtils::parse_amount(&value).unwrap(), -50000);
    }

    #[test]
    fn test_parse_amount_string() {
        let value = json!("123.45");
        assert_eq!(MoneyUtils::parse_amount(&value).unwrap(), 123450);
    }

    #[test]
    fn test_format_milliunits() {
        assert_eq!(MoneyUtils::format_milliunits(123450), "£123.45");
        assert_eq!(MoneyUtils::format_milliunits(-50000), "-£50.00");
        assert_eq!(MoneyUtils::format_milliunits(0), "£0.00");
    }

    #[test]
    fn test_validate_amount() {
        let args = json!({"amount": 123.45});
        assert_eq!(
            MoneyUtils::validate_amount(&args, "amount").unwrap(),
            123450
        );
    }
}
