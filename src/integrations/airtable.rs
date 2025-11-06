use super::{ExternalIntegration, ExternalRecord};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AirtableClient {
    client: Client,
    api_key: String,
    base_id: String,
    table_name: String,
}

impl AirtableClient {
    pub fn new(api_key: String, base_id: String, table_name: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_id,
            table_name,
        }
    }
}

#[async_trait::async_trait]
impl ExternalIntegration for AirtableClient {
    async fn connect(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Connecting to Airtable base: {}", self.base_id);
        // Stub: Test connection
        Ok(())
    }

    async fn sync_users(&self, users: Vec<crate::db::models::User>) -> Result<(), anyhow::Error> {
        tracing::info!("Syncing {} users to Airtable table '{}'", users.len(), self.table_name);

        // Stub: Would create/update records in Airtable
        for user in users {
            tracing::debug!("Would sync user: {} to Airtable", user.line_user_id);
        }

        Ok(())
    }

    async fn sync_messages(&self, messages: Vec<crate::db::models::Message>) -> Result<(), anyhow::Error> {
        tracing::info!("Syncing {} messages to Airtable", messages.len());

        // Stub: Would create records in Airtable
        for msg in messages {
            tracing::debug!("Would sync message from {} to Airtable", msg.line_user_id);
        }

        Ok(())
    }

    async fn fetch_records(&self) -> Result<Vec<ExternalRecord>, anyhow::Error> {
        tracing::info!("Fetching records from Airtable table '{}'", self.table_name);

        // Stub: Would fetch from Airtable API
        Ok(vec![])
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AirtableRecord {
    id: Option<String>,
    fields: serde_json::Value,
}
