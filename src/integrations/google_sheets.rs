use super::{ExternalIntegration, ExternalRecord};
use reqwest::Client;

pub struct GoogleSheetsClient {
    client: Client,
    credentials: String,
    spreadsheet_id: String,
}

impl GoogleSheetsClient {
    pub fn new(credentials: String, spreadsheet_id: String) -> Self {
        Self {
            client: Client::new(),
            credentials,
            spreadsheet_id,
        }
    }
}

#[async_trait::async_trait]
impl ExternalIntegration for GoogleSheetsClient {
    async fn connect(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Connecting to Google Sheets: {}", self.spreadsheet_id);
        // Stub: Would authenticate with Google OAuth2
        Ok(())
    }

    async fn sync_users(&self, users: Vec<crate::db::models::User>) -> Result<(), anyhow::Error> {
        tracing::info!("Syncing {} users to Google Sheets", users.len());

        // Stub: Would append rows to Google Sheets
        for user in users {
            tracing::debug!("Would sync user: {} to Google Sheets", user.line_user_id);
        }

        Ok(())
    }

    async fn sync_messages(&self, messages: Vec<crate::db::models::Message>) -> Result<(), anyhow::Error> {
        tracing::info!("Syncing {} messages to Google Sheets", messages.len());

        // Stub: Would append rows to Google Sheets
        for msg in messages {
            tracing::debug!("Would sync message from {} to Google Sheets", msg.line_user_id);
        }

        Ok(())
    }

    async fn fetch_records(&self) -> Result<Vec<ExternalRecord>, anyhow::Error> {
        tracing::info!("Fetching records from Google Sheets");

        // Stub: Would fetch rows from Google Sheets API
        Ok(vec![])
    }
}
