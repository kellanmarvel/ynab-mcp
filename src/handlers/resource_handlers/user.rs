use anyhow::Result;
use serde_json::{json, Value};

use crate::handlers::common::CommonUtils;

pub struct UserResourceHandler<'a> {
    pub common: &'a CommonUtils,
}

impl<'a> UserResourceHandler<'a> {
    pub fn new(common: &'a CommonUtils) -> Self {
        Self { common }
    }

    pub async fn get_user(&self) -> Result<Value> {
        let user = self.common.ynab_client.get_user().await?;
        let data = json!({
            "type": "user",
            "data": user
        });
        Ok(self.common.create_mcp_response(
            "ynab://user",
            "user.json",
            "YNAB User Information",
            data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::common::CommonUtils;
    use crate::secrets::YnabApiToken;
    use crate::ynab::YnabClient;

    fn create_test_common() -> CommonUtils {
        let token = YnabApiToken::new("test-token".to_string());
        let client =
            YnabClient::new_for_test("https://api.ynab.com/v1".to_string(), token).unwrap();
        CommonUtils::new(client, true)
    }

    #[tokio::test]
    async fn test_user_resource_handler_creation() {
        let common = create_test_common();
        let handler = UserResourceHandler::new(&common);

        assert!(handler.common.read_only_mode);
    }

    #[tokio::test]
    async fn test_user_handler_struct() {
        let common = create_test_common();
        let handler = UserResourceHandler::new(&common);

        // Test the handler structure
        assert_eq!(handler.common.read_only_mode, common.read_only_mode);
    }
}
