use super::{ExternalIntegration, ExternalRecord};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct NotionClient {
    client: Client,
    api_key: String,
    database_id: String,
}

impl NotionClient {
    pub fn new(api_key: String, database_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            database_id,
        }
    }
}

#[async_trait::async_trait]
impl ExternalIntegration for NotionClient {
    async fn connect(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Connecting to Notion...");
        // Stub: Test connection
        Ok(())
    }

    async fn sync_users(&self, users: Vec<crate::db::models::User>) -> Result<(), anyhow::Error> {
        tracing::info!("Syncing {} users to Notion database {}", users.len(), self.database_id);

        // Stub: Would send users to Notion database
        for user in users {
            tracing::debug!("Would sync user: {} to Notion", user.line_user_id);
        }

        Ok(())
    }

    async fn sync_messages(&self, messages: Vec<crate::db::models::Message>) -> Result<(), anyhow::Error> {
        tracing::info!("Syncing {} messages to Notion", messages.len());

        // Stub: Would send messages to Notion
        for msg in messages {
            tracing::debug!("Would sync message from user {} to Notion", msg.line_user_id);
        }

        Ok(())
    }

    async fn fetch_records(&self) -> Result<Vec<ExternalRecord>, anyhow::Error> {
        tracing::info!("Fetching records from Notion database {}", self.database_id);

        // Stub: Would fetch from Notion API
        Ok(vec![])
    }
}

#[derive(Debug, Serialize)]
struct NotionPage {
    parent: NotionParent,
    properties: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct NotionParent {
    database_id: String,
}
