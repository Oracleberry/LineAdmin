pub mod notion;
pub mod airtable;
pub mod google_sheets;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRecord {
    pub id: String,
    pub fields: serde_json::Value,
}

#[async_trait::async_trait]
pub trait ExternalIntegration {
    async fn connect(&self) -> Result<(), anyhow::Error>;
    async fn sync_users(&self, users: Vec<crate::db::models::User>) -> Result<(), anyhow::Error>;
    async fn sync_messages(&self, messages: Vec<crate::db::models::Message>) -> Result<(), anyhow::Error>;
    async fn fetch_records(&self) -> Result<Vec<ExternalRecord>, anyhow::Error>;
}
