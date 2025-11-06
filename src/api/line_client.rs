use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PushMessage {
    to: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
pub struct ReplyMessage {
    #[serde(rename = "replyToken")]
    reply_token: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
pub struct BroadcastMessage {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        #[serde(rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(rename = "previewImageUrl")]
        preview_image_url: String,
    },
    #[serde(rename = "video")]
    Video {
        #[serde(rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(rename = "previewImageUrl")]
        preview_image_url: String,
    },
    #[serde(rename = "flex")]
    Flex {
        #[serde(rename = "altText")]
        alt_text: String,
        contents: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
pub struct LineApiResponse {
    pub message: Option<String>,
}

pub struct LineClient {
    client: Client,
    access_token: String,
}

impl LineClient {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    /// Push message to a specific user
    pub async fn push_message(&self, user_id: &str, messages: Vec<Message>) -> Result<(), anyhow::Error> {
        let payload = PushMessage {
            to: user_id.to_string(),
            messages,
        };

        let response = self
            .client
            .post("https://api.line.me/v2/bot/message/push")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("LINE API error: {}", error_text));
        }

        Ok(())
    }

    /// Reply to a message
    pub async fn reply_message(&self, reply_token: &str, messages: Vec<Message>) -> Result<(), anyhow::Error> {
        let payload = ReplyMessage {
            reply_token: reply_token.to_string(),
            messages,
        };

        let response = self
            .client
            .post("https://api.line.me/v2/bot/message/reply")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("LINE API error: {}", error_text));
        }

        Ok(())
    }

    /// Broadcast message to all users
    pub async fn broadcast_message(&self, messages: Vec<Message>) -> Result<(), anyhow::Error> {
        let payload = BroadcastMessage { messages };

        let response = self
            .client
            .post("https://api.line.me/v2/bot/message/broadcast")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("LINE API error: {}", error_text));
        }

        Ok(())
    }

    /// Get user profile
    pub async fn get_profile(&self, user_id: &str) -> Result<UserProfile, anyhow::Error> {
        let response = self
            .client
            .get(&format!("https://api.line.me/v2/bot/profile/{}", user_id))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("LINE API error: {}", error_text));
        }

        let profile = response.json::<UserProfile>().await?;
        Ok(profile)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserProfile {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "pictureUrl")]
    pub picture_url: Option<String>,
    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
}
