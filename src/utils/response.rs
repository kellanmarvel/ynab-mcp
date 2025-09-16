use crate::utils::money::MoneyUtils;
use serde_json::{json, Value};

/// Minimal response utilities for MCP server compliance
pub struct ResponseUtils;

impl ResponseUtils {
    /// Create a standard success response with amount formatting
    pub fn amount_success(
        operation: &str,
        amount: i64,
        entity: &str,
        additional_data: Option<Value>,
    ) -> Value {
        let formatted_amount = MoneyUtils::format_milliunits(amount);

        let mut response = json!({
            "success": true,
            "conversational_summary": format!(
                "✅ Successfully {} {} with amount {}",
                operation, entity, formatted_amount
            ),
            "message": format!(
                "💰 {} {} - Amount: {}",
                operation.chars().next()
                    .map(|c| c.to_uppercase().collect::<String>() + &operation[1..])
                    .unwrap_or_else(|| operation.to_string()),
                entity,
                formatted_amount
            ),
            "amount_details": {
                "formatted_amount": formatted_amount,
                "raw_milliunits": amount
            }
        });

        // Merge additional data if provided
        if let Some(data) = additional_data {
            if let (Value::Object(response_map), Value::Object(data_map)) = (&mut response, data) {
                for (key, value) in data_map {
                    response_map.insert(key, value);
                }
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_success_response() {
        let response = ResponseUtils::amount_success(
            "transferred",
            5000, // $5.00 in milliunits
            "test entity",
            None,
        );

        assert_eq!(response["success"], true);
        assert!(response["conversational_summary"]
            .as_str()
            .unwrap()
            .contains("5.00"));
        assert_eq!(response["amount_details"]["raw_milliunits"], 5000);
        assert!(!response["amount_details"]["formatted_amount"]
            .as_str()
            .unwrap()
            .is_empty());
    }
}
