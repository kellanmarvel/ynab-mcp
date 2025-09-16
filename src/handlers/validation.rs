use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

/// Input validation utilities for API endpoints
pub struct InputValidator;

impl InputValidator {
    /// Validate a required string field
    pub fn validate_required_string(args: &Value, field_name: &str) -> Result<String> {
        args.get(field_name)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing or empty required field: {}", field_name))
    }

    /// Validate an optional string field
    pub fn validate_optional_string(args: &Value, field_name: &str) -> Result<Option<String>> {
        match args.get(field_name) {
            Some(Value::String(s)) => {
                if s.is_empty() {
                    Ok(None)
                } else if s.len() > 1000 {
                    Err(anyhow::anyhow!(
                        "{} cannot exceed 1000 characters",
                        field_name
                    ))
                } else {
                    Ok(Some(s.clone()))
                }
            }
            Some(Value::Null) => Ok(None),
            None => Ok(None),
            _ => Err(anyhow::anyhow!(
                "Invalid {} format, must be a string",
                field_name
            )),
        }
    }

    /// Validate a UUID field
    pub fn validate_uuid(args: &Value, field_name: &str) -> Result<Uuid> {
        let uuid_str = Self::validate_required_string(args, field_name)?;
        Uuid::parse_str(&uuid_str)
            .with_context(|| format!("Invalid {} format, must be a valid UUID", field_name))
    }

    /// Validate an optional UUID field
    pub fn validate_optional_uuid(args: &Value, field_name: &str) -> Result<Option<Uuid>> {
        match Self::validate_optional_string(args, field_name)? {
            Some(uuid_str) => {
                let uuid = Uuid::parse_str(&uuid_str).with_context(|| {
                    format!("Invalid {} format, must be a valid UUID", field_name)
                })?;
                Ok(Some(uuid))
            }
            None => Ok(None),
        }
    }

    /// Validate a date field in YYYY-MM-DD format
    pub fn validate_date(args: &Value, field_name: &str) -> Result<NaiveDate> {
        let date_str = Self::validate_required_string(args, field_name)?;

        // Basic format validation
        if !date_str.matches('-').count() == 2 {
            return Err(anyhow::anyhow!(
                "Invalid {} format, expected YYYY-MM-DD",
                field_name
            ));
        }

        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!(
                "Invalid {} format, expected YYYY-MM-DD",
                field_name
            ));
        }

        // Validate year, month, day ranges
        let year: i32 = parts[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid year in {}", field_name))?;
        let month: u32 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid month in {}", field_name))?;
        let day: u32 = parts[2]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid day in {}", field_name))?;

        if !(1900..=2100).contains(&year) {
            return Err(anyhow::anyhow!(
                "Invalid year in {}, must be between 1900-2100",
                field_name
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(anyhow::anyhow!(
                "Invalid month in {}, must be between 1-12",
                field_name
            ));
        }
        if !(1..=31).contains(&day) {
            return Err(anyhow::anyhow!(
                "Invalid day in {}, must be between 1-31",
                field_name
            ));
        }

        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow::anyhow!("Invalid date: {}", date_str))
    }

    /// Validate an optional date field
    pub fn validate_optional_date(args: &Value, field_name: &str) -> Result<Option<NaiveDate>> {
        match Self::validate_optional_string(args, field_name)? {
            Some(date_str) => {
                let date = Self::validate_date_string(&date_str, field_name)?;
                Ok(Some(date))
            }
            None => Ok(None),
        }
    }

    /// Helper to validate a date string directly
    fn validate_date_string(date_str: &str, field_name: &str) -> Result<NaiveDate> {
        // Basic format validation
        if !date_str.matches('-').count() == 2 {
            return Err(anyhow::anyhow!(
                "Invalid {} format, expected YYYY-MM-DD",
                field_name
            ));
        }

        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!(
                "Invalid {} format, expected YYYY-MM-DD",
                field_name
            ));
        }

        let year: i32 = parts[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid year in {}", field_name))?;
        let month: u32 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid month in {}", field_name))?;
        let day: u32 = parts[2]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid day in {}", field_name))?;

        if !(1900..=2100).contains(&year) {
            return Err(anyhow::anyhow!(
                "Invalid year in {}, must be between 1900-2100",
                field_name
            ));
        }

        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow::anyhow!("Invalid date: {}", date_str))
    }

    /// Validate an amount field (supports both string and number formats)
    /// Deprecated: Use MoneyUtils::validate_amount directly
    pub fn validate_amount(args: &Value, field_name: &str) -> Result<i64> {
        crate::utils::MoneyUtils::validate_amount(args, field_name).map_err(|e| e.into())
    }

    /// Validate an optional amount field
    /// Deprecated: Use MoneyUtils::validate_optional_amount directly
    pub fn validate_optional_amount(args: &Value, field_name: &str) -> Result<Option<i64>> {
        crate::utils::MoneyUtils::validate_optional_amount(args, field_name).map_err(|e| e.into())
    }

    /// Validate an optional boolean field
    pub fn validate_optional_boolean(args: &Value, field_name: &str) -> Result<Option<bool>> {
        match args.get(field_name) {
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(Value::Null) => Ok(None),
            None => Ok(None),
            _ => Err(anyhow::anyhow!(
                "Invalid {} format, must be a boolean",
                field_name
            )),
        }
    }

    /// Validate an optional enum field
    pub fn validate_optional_enum(
        args: &Value,
        field_name: &str,
        allowed_values: &[&str],
    ) -> Result<Option<String>> {
        match Self::validate_optional_string(args, field_name)? {
            Some(value) => {
                if allowed_values.contains(&value.as_str()) {
                    Ok(Some(value))
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid {} value: '{}'. Allowed values: {}",
                        field_name,
                        value,
                        allowed_values.join(", ")
                    ))
                }
            }
            None => Ok(None),
        }
    }

    /// Validate budget_id parameter (required for most operations)
    pub fn validate_budget_id(args: &Value) -> Result<String> {
        let budget_id = Self::validate_required_string(args, "budget_id")?;

        // Allow "default" or valid UUID format
        if budget_id == "default" {
            Ok(budget_id)
        } else {
            // Validate as UUID
            Uuid::parse_str(&budget_id)
                .with_context(|| "Invalid budget_id format, must be 'default' or a valid UUID")?;
            Ok(budget_id)
        }
    }

    /// Validate that required fields are present
    pub fn validate_required_fields(args: &Value, required_fields: &[&str]) -> Result<()> {
        let mut missing_fields = Vec::new();

        for field in required_fields {
            if args.get(field).is_none() {
                missing_fields.push(*field);
            }
        }

        if !missing_fields.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing required fields: {}",
                missing_fields.join(", ")
            ));
        }

        Ok(())
    }

    /// Validate that no unexpected fields are present
    pub fn validate_no_unexpected_fields(args: &Value, allowed_fields: &[&str]) -> Result<()> {
        let allowed_set: HashSet<&str> = allowed_fields.iter().cloned().collect();

        if let Some(obj) = args.as_object() {
            let unexpected_fields: Vec<&String> = obj
                .keys()
                .filter(|key| !allowed_set.contains(key.as_str()))
                .collect();

            if !unexpected_fields.is_empty() {
                return Err(anyhow::anyhow!(
                    "Unexpected fields: {}. Allowed fields: {}",
                    unexpected_fields
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    allowed_fields.join(", ")
                ));
            }
        }

        Ok(())
    }

    /// Comprehensive validation for transaction creation
    pub fn validate_create_transaction(args: &Value) -> Result<()> {
        // Define allowed fields
        let allowed_fields = &[
            "budget_id",
            "account_id",
            "amount",
            "date",
            "memo",
            "payee_id",
            "category_id",
            "cleared",
            "flag_color",
            "flag_name",
        ];

        // Check for unexpected fields first
        Self::validate_no_unexpected_fields(args, allowed_fields)?;

        // Validate required fields
        Self::validate_required_fields(args, &["budget_id", "account_id", "amount", "date"])?;

        // Validate individual fields
        Self::validate_budget_id(args)?;
        Self::validate_uuid(args, "account_id")?;
        Self::validate_amount(args, "amount")?;
        Self::validate_date(args, "date")?;

        // Validate optional fields
        Self::validate_optional_string(args, "memo")?;
        Self::validate_optional_uuid(args, "payee_id")?;
        Self::validate_optional_uuid(args, "category_id")?;
        Self::validate_optional_boolean(args, "cleared")?;
        Self::validate_optional_enum(
            args,
            "flag_color",
            &["red", "orange", "yellow", "green", "blue", "purple"],
        )?;
        Self::validate_optional_string(args, "flag_name")?;

        Ok(())
    }

    /// Comprehensive validation for transaction updates
    pub fn validate_update_transaction(args: &Value) -> Result<()> {
        // Define allowed fields
        let allowed_fields = &[
            "budget_id",
            "transaction_id",
            "account_id",
            "amount",
            "date",
            "memo",
            "payee_id",
            "category_id",
            "cleared",
            "flag_color",
            "flag_name",
        ];

        // Check for unexpected fields first
        Self::validate_no_unexpected_fields(args, allowed_fields)?;

        // Validate required fields for update
        Self::validate_required_fields(args, &["budget_id", "transaction_id"])?;

        // Validate individual fields
        Self::validate_budget_id(args)?;
        Self::validate_uuid(args, "transaction_id")?;

        // Validate optional fields (all transaction fields are optional in updates)
        Self::validate_optional_uuid(args, "account_id")?;
        Self::validate_optional_amount(args, "amount")?;
        Self::validate_optional_date(args, "date")?;
        Self::validate_optional_string(args, "memo")?;
        Self::validate_optional_uuid(args, "payee_id")?;
        Self::validate_optional_uuid(args, "category_id")?;
        Self::validate_optional_boolean(args, "cleared")?;
        Self::validate_optional_enum(
            args,
            "flag_color",
            &["red", "orange", "yellow", "green", "blue", "purple"],
        )?;
        Self::validate_optional_string(args, "flag_name")?;

        Ok(())
    }

    /// Validation for approve transaction operation
    pub fn validate_approve_transaction(args: &Value) -> Result<()> {
        let allowed_fields = &["budget_id", "transaction_id"];
        Self::validate_no_unexpected_fields(args, allowed_fields)?;
        Self::validate_required_fields(args, &["budget_id", "transaction_id"])?;
        Self::validate_budget_id(args)?;
        Self::validate_uuid(args, "transaction_id")?;
        Ok(())
    }

    /// Validation for clear transaction operation
    pub fn validate_clear_transaction(args: &Value) -> Result<()> {
        let allowed_fields = &["budget_id", "transaction_id", "clear_state"];
        Self::validate_no_unexpected_fields(args, allowed_fields)?;
        Self::validate_required_fields(args, &["budget_id", "transaction_id"])?;
        Self::validate_budget_id(args)?;
        Self::validate_uuid(args, "transaction_id")?;
        Self::validate_optional_enum(args, "clear_state", &["cleared", "uncleared", "reconciled"])?;
        Ok(())
    }

    /// Validation for transfer creation
    pub fn validate_create_transfer(args: &Value) -> Result<()> {
        let allowed_fields = &[
            "budget_id",
            "from_account_id",
            "to_account_id",
            "amount",
            "date",
            "memo",
        ];
        Self::validate_no_unexpected_fields(args, allowed_fields)?;
        Self::validate_required_fields(
            args,
            &[
                "budget_id",
                "from_account_id",
                "to_account_id",
                "amount",
                "date",
            ],
        )?;
        Self::validate_budget_id(args)?;
        Self::validate_uuid(args, "from_account_id")?;
        Self::validate_uuid(args, "to_account_id")?;
        Self::validate_amount(args, "amount")?;
        Self::validate_date(args, "date")?;
        Self::validate_optional_string(args, "memo")?;
        Ok(())
    }

    /// Validation for budget assignment
    pub fn validate_assign_budget(args: &Value) -> Result<()> {
        let allowed_fields = &["budget_id", "category_id", "budgeted_amount", "month"];
        Self::validate_no_unexpected_fields(args, allowed_fields)?;
        Self::validate_required_fields(args, &["budget_id", "category_id", "budgeted_amount"])?;
        Self::validate_budget_id(args)?;
        Self::validate_uuid(args, "category_id")?;
        Self::validate_amount(args, "budgeted_amount")?;
        Self::validate_optional_date(args, "month")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_required_string() {
        let args = json!({"test": "value"});
        assert_eq!(
            InputValidator::validate_required_string(&args, "test").unwrap(),
            "value"
        );

        let args = json!({"test": ""});
        assert!(InputValidator::validate_required_string(&args, "test").is_err());

        let args = json!({});
        assert!(InputValidator::validate_required_string(&args, "test").is_err());
    }

    #[test]
    fn test_validate_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let args = json!({"test": valid_uuid});
        assert!(InputValidator::validate_uuid(&args, "test").is_ok());

        let args = json!({"test": "invalid-uuid"});
        assert!(InputValidator::validate_uuid(&args, "test").is_err());
    }

    #[test]
    fn test_validate_date() {
        let args = json!({"test": "2024-01-15"});
        assert!(InputValidator::validate_date(&args, "test").is_ok());

        let args = json!({"test": "invalid-date"});
        assert!(InputValidator::validate_date(&args, "test").is_err());

        let args = json!({"test": "2024-13-01"}); // Invalid month
        assert!(InputValidator::validate_date(&args, "test").is_err());
    }

    #[test]
    fn test_validate_amount() {
        let args = json!({"test": "123.45"});
        assert_eq!(
            InputValidator::validate_amount(&args, "test").unwrap(),
            123450
        );

        let args = json!({"test": 123.45});
        assert_eq!(
            InputValidator::validate_amount(&args, "test").unwrap(),
            123450
        );

        let args = json!({"test": "invalid"});
        assert!(InputValidator::validate_amount(&args, "test").is_err());

        // Test range validation
        let args = json!({"test": "1000000000"}); // Too large
        assert!(InputValidator::validate_amount(&args, "test").is_err());
    }

    #[test]
    fn test_validate_create_transaction() {
        let valid_args = json!({
            "budget_id": "default",
            "account_id": "550e8400-e29b-41d4-a716-446655440000",
            "amount": "123.45",
            "date": "2024-01-15"
        });
        assert!(InputValidator::validate_create_transaction(&valid_args).is_ok());

        let invalid_args = json!({
            "budget_id": "default",
            "account_id": "invalid-uuid",
            "amount": "123.45",
            "date": "2024-01-15"
        });
        assert!(InputValidator::validate_create_transaction(&invalid_args).is_err());

        let missing_fields = json!({
            "budget_id": "default"
        });
        assert!(InputValidator::validate_create_transaction(&missing_fields).is_err());
    }

    #[test]
    fn test_validate_update_transaction() {
        let valid_args = json!({
            "budget_id": "default",
            "transaction_id": "550e8400-e29b-41d4-a716-446655440000"
        });
        assert!(InputValidator::validate_update_transaction(&valid_args).is_ok());

        let invalid_args = json!({
            "budget_id": "default",
            "transaction_id": "invalid-uuid"
        });
        assert!(InputValidator::validate_update_transaction(&invalid_args).is_err());

        let missing_fields = json!({
            "budget_id": "default"
        });
        assert!(InputValidator::validate_update_transaction(&missing_fields).is_err());
    }

    #[test]
    fn test_validate_create_transfer() {
        let valid_args = json!({
            "budget_id": "default",
            "from_account_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_account_id": "550e8400-e29b-41d4-a716-446655440001",
            "amount": "100.00",
            "date": "2024-01-15"
        });
        assert!(InputValidator::validate_create_transfer(&valid_args).is_ok());

        let invalid_amount = json!({
            "budget_id": "default",
            "from_account_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_account_id": "550e8400-e29b-41d4-a716-446655440001",
            "amount": "invalid",
            "date": "2024-01-15"
        });
        assert!(InputValidator::validate_create_transfer(&invalid_amount).is_err());
    }

    #[test]
    fn test_validate_assign_budget() {
        let valid_args = json!({
            "budget_id": "default",
            "category_id": "550e8400-e29b-41d4-a716-446655440000",
            "budgeted_amount": "500.00"
        });
        assert!(InputValidator::validate_assign_budget(&valid_args).is_ok());

        let invalid_category = json!({
            "budget_id": "default",
            "category_id": "invalid-uuid",
            "budgeted_amount": "500.00"
        });
        assert!(InputValidator::validate_assign_budget(&invalid_category).is_err());
    }

    #[test]
    fn test_validate_required_fields_edge_cases() {
        // Test with null values - validate_required_fields only checks presence, not content
        let null_field = json!({"test": null});
        assert!(InputValidator::validate_required_fields(&null_field, &["test"]).is_ok());

        // Test with empty string - validate_required_fields only checks presence, not content
        let empty_string = json!({"test": ""});
        assert!(InputValidator::validate_required_fields(&empty_string, &["test"]).is_ok());

        // Test with whitespace only - validate_required_fields only checks presence, not content
        let whitespace = json!({"test": "   "});
        assert!(InputValidator::validate_required_fields(&whitespace, &["test"]).is_ok());

        // Test with valid value
        let valid = json!({"test": "value"});
        assert!(InputValidator::validate_required_fields(&valid, &["test"]).is_ok());

        // Test with missing field - this should error
        let missing = json!({});
        assert!(InputValidator::validate_required_fields(&missing, &["test"]).is_err());
    }

    #[test]
    fn test_validate_no_unexpected_fields() {
        let valid_args = json!({"allowed1": "value", "allowed2": "value"});
        assert!(InputValidator::validate_no_unexpected_fields(
            &valid_args,
            &["allowed1", "allowed2"]
        )
        .is_ok());

        let invalid_args = json!({"allowed1": "value", "unexpected": "value"});
        assert!(InputValidator::validate_no_unexpected_fields(
            &invalid_args,
            &["allowed1", "allowed2"]
        )
        .is_err());

        // Test with empty object
        let empty_args = json!({});
        assert!(InputValidator::validate_no_unexpected_fields(&empty_args, &["allowed"]).is_ok());
    }

    #[test]
    fn test_validate_optional_string() {
        // Valid optional string
        let valid_args = json!({"test": "value"});
        assert!(InputValidator::validate_optional_string(&valid_args, "test").is_ok());

        // Missing field should be ok for optional
        let missing_args = json!({});
        assert!(InputValidator::validate_optional_string(&missing_args, "test").is_ok());

        // Empty string should be ok for optional
        let empty_args = json!({"test": ""});
        assert!(InputValidator::validate_optional_string(&empty_args, "test").is_ok());

        // Null should be ok for optional
        let null_args = json!({"test": null});
        assert!(InputValidator::validate_optional_string(&null_args, "test").is_ok());
    }

    #[test]
    fn test_validate_optional_date() {
        // Valid optional date
        let valid_args = json!({"test": "2024-01-15"});
        assert!(InputValidator::validate_optional_date(&valid_args, "test").is_ok());

        // Missing field should be ok
        let missing_args = json!({});
        assert!(InputValidator::validate_optional_date(&missing_args, "test").is_ok());

        // Invalid date should error
        let invalid_args = json!({"test": "invalid-date"});
        assert!(InputValidator::validate_optional_date(&invalid_args, "test").is_err());
    }

    #[test]
    fn test_validate_amount_edge_cases() {
        // Test negative amounts
        let negative = json!({"test": "-100.00"});
        assert!(InputValidator::validate_amount(&negative, "test").is_ok());

        // Test zero amount
        let zero = json!({"test": "0.00"});
        assert!(InputValidator::validate_amount(&zero, "test").is_ok());

        // Test very small amount
        let small = json!({"test": "0.01"});
        assert!(InputValidator::validate_amount(&small, "test").is_ok());

        // Test integer amounts
        let integer = json!({"test": 100});
        assert!(InputValidator::validate_amount(&integer, "test").is_ok());

        // Test missing decimal
        let no_decimal = json!({"test": "100"});
        assert!(InputValidator::validate_amount(&no_decimal, "test").is_ok());
    }

    #[test]
    fn test_validate_uuid_edge_cases() {
        // Test "default" special case - validate_uuid doesn't handle this, only validate_budget_id does
        let default_args = json!({"test": "default"});
        assert!(InputValidator::validate_uuid(&default_args, "test").is_err());

        // Test empty string
        let empty_args = json!({"test": ""});
        assert!(InputValidator::validate_uuid(&empty_args, "test").is_err());

        // Test null
        let null_args = json!({"test": null});
        assert!(InputValidator::validate_uuid(&null_args, "test").is_err());

        // Test lowercase UUID
        let lowercase = json!({"test": "550e8400-e29b-41d4-a716-446655440000"});
        assert!(InputValidator::validate_uuid(&lowercase, "test").is_ok());

        // Test uppercase UUID
        let uppercase = json!({"test": "550E8400-E29B-41D4-A716-446655440000"});
        assert!(InputValidator::validate_uuid(&uppercase, "test").is_ok());
    }

    #[test]
    fn test_validate_budget_id_edge_cases() {
        // Test "default" special case - this method handles it
        let default_args = json!({"budget_id": "default"});
        assert!(InputValidator::validate_budget_id(&default_args).is_ok());

        // Test valid UUID
        let uuid_args = json!({"budget_id": "550e8400-e29b-41d4-a716-446655440000"});
        assert!(InputValidator::validate_budget_id(&uuid_args).is_ok());

        // Test invalid UUID
        let invalid_args = json!({"budget_id": "invalid-uuid"});
        assert!(InputValidator::validate_budget_id(&invalid_args).is_err());

        // Test missing budget_id
        let missing_args = json!({});
        assert!(InputValidator::validate_budget_id(&missing_args).is_err());
    }

    #[test]
    fn test_validate_date_edge_cases() {
        // Test leap year date
        let leap_year = json!({"test": "2024-02-29"});
        assert!(InputValidator::validate_date(&leap_year, "test").is_ok());

        // Test invalid leap year date
        let invalid_leap = json!({"test": "2023-02-29"});
        assert!(InputValidator::validate_date(&invalid_leap, "test").is_err());

        // Test end of year
        let end_year = json!({"test": "2024-12-31"});
        assert!(InputValidator::validate_date(&end_year, "test").is_ok());

        // Test start of year
        let start_year = json!({"test": "2024-01-01"});
        assert!(InputValidator::validate_date(&start_year, "test").is_ok());
    }
}
