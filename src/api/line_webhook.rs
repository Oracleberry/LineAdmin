use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::prelude::*;

use crate::api::AppState;
use crate::db::models::{User, Message};
use crate::notification;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct LineWebhook {
    destination: String,
    events: Vec<LineEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum LineEvent {
    #[serde(rename = "message")]
    Message {
        #[serde(rename = "replyToken")]
        reply_token: String,
        source: EventSource,
        message: LineMessage,
        timestamp: i64,
    },
    #[serde(rename = "follow")]
    Follow {
        #[serde(rename = "replyToken")]
        reply_token: String,
        source: EventSource,
        timestamp: i64,
    },
    #[serde(rename = "unfollow")]
    Unfollow {
        source: EventSource,
        timestamp: i64,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct EventSource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum LineMessage {
    #[serde(rename = "text")]
    Text {
        id: String,
        text: String,
    },
    #[serde(rename = "image")]
    Image {
        id: String,
    },
    #[serde(rename = "video")]
    Video {
        id: String,
    },
    #[serde(rename = "audio")]
    Audio {
        id: String,
    },
    #[serde(rename = "location")]
    Location {
        id: String,
        title: String,
        address: String,
        latitude: f64,
        longitude: f64,
    },
    #[serde(rename = "sticker")]
    Sticker {
        id: String,
        #[serde(rename = "packageId")]
        package_id: String,
        #[serde(rename = "stickerId")]
        sticker_id: String,
    },
    #[serde(other)]
    Other,
}

pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<LineWebhook>,
) -> impl IntoResponse {
    // Verify LINE signature
    let signature = headers.get("x-line-signature");
    if signature.is_none() {
        tracing::warn!("Missing LINE signature header");
        return StatusCode::UNAUTHORIZED;
    }

    // TODO: Implement signature verification with channel secret
    // For now, we'll skip verification in development

    tracing::info!("Received LINE webhook with {} events", payload.events.len());

    for event in payload.events {
        if let Err(e) = process_event(&state, event).await {
            tracing::error!("Failed to process event: {}", e);
        }
    }

    StatusCode::OK
}

async fn process_event(state: &AppState, event: LineEvent) -> Result<(), anyhow::Error> {
    match event {
        LineEvent::Message { source, message, .. } => {
            handle_message_event(state, &source.user_id, message).await?;
        }
        LineEvent::Follow { source, .. } => {
            tracing::info!("User followed: {}", source.user_id);
            User::create(&state.db, &source.user_id, None).await?;

            // Send notification to admin
            let msg = format!("New follower: {}", source.user_id);
            notification::send_notifications(&state.db, &msg).await;
        }
        LineEvent::Unfollow { source, .. } => {
            tracing::info!("User unfollowed: {}", source.user_id);

            // Send notification to admin
            let msg = format!("User unfollowed: {}", source.user_id);
            notification::send_notifications(&state.db, &msg).await;
        }
        LineEvent::Other => {
            tracing::debug!("Received other event type");
        }
    }

    Ok(())
}

async fn handle_message_event(
    state: &AppState,
    user_id: &str,
    message: LineMessage,
) -> Result<(), anyhow::Error> {
    // Ensure user exists in database
    User::create(&state.db, user_id, None).await?;

    match message {
        LineMessage::Text { text, .. } => {
            tracing::info!("Received text message from {}: {}", user_id, text);

            // Store message in database
            Message::create(&state.db, user_id, "text", Some(&text), None).await?;

            // Send notification to admin
            let notification_msg = format!("New message from {}: {}", user_id, text);
            notification::send_notifications(&state.db, &notification_msg).await;
        }
        LineMessage::Image { .. } => {
            tracing::info!("Received image message from {}", user_id);
            Message::create(&state.db, user_id, "image", None, None).await?;
        }
        LineMessage::Video { .. } => {
            tracing::info!("Received video message from {}", user_id);
            Message::create(&state.db, user_id, "video", None, None).await?;
        }
        LineMessage::Audio { .. } => {
            tracing::info!("Received audio message from {}", user_id);
            Message::create(&state.db, user_id, "audio", None, None).await?;
        }
        LineMessage::Location { title, address, latitude, longitude, .. } => {
            tracing::info!("Received location message from {}", user_id);
            let data = serde_json::json!({
                "title": title,
                "address": address,
                "latitude": latitude,
                "longitude": longitude,
            });
            Message::create(&state.db, user_id, "location", None, Some(&data.to_string())).await?;
        }
        LineMessage::Sticker { package_id, sticker_id, .. } => {
            tracing::info!("Received sticker message from {}", user_id);
            let data = serde_json::json!({
                "packageId": package_id,
                "stickerId": sticker_id,
            });
            Message::create(&state.db, user_id, "sticker", None, Some(&data.to_string())).await?;
        }
        LineMessage::Other => {
            tracing::debug!("Received other message type from {}", user_id);
        }
    }

    Ok(())
}
