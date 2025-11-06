use reqwest::Client;
use serde::Serialize;
use sqlx::SqlitePool;
use crate::db::models::Setting;

#[derive(Debug, Serialize)]
struct LineNotifyPayload {
    message: String,
}

#[derive(Debug, Serialize)]
struct SlackPayload {
    text: String,
}

/// Send notifications to both LINE Notify and Slack
pub async fn send_notifications(db: &SqlitePool, message: &str) {
    // Send to LINE Notify
    if let Err(e) = send_line_notify(db, message).await {
        tracing::error!("Failed to send LINE Notify: {}", e);
    }

    // Send to Slack
    if let Err(e) = send_slack(db, message).await {
        tracing::error!("Failed to send Slack notification: {}", e);
    }
}

/// Send notification via LINE Notify
pub async fn send_line_notify(db: &SqlitePool, message: &str) -> Result<(), anyhow::Error> {
    // Get LINE Notify token from settings
    let token = match Setting::get(db, "line_notify_token").await? {
        Some(t) => t,
        None => {
            tracing::warn!("LINE Notify token not configured");
            return Ok(());
        }
    };

    if token.is_empty() {
        tracing::warn!("LINE Notify token is empty");
        return Ok(());
    }

    let client = Client::new();
    let response = client
        .post("https://notify-api.line.me/api/notify")
        .header("Authorization", format!("Bearer {}", token))
        .form(&LineNotifyPayload {
            message: message.to_string(),
        })
        .send()
        .await?;

    if response.status().is_success() {
        tracing::info!("LINE Notify sent successfully");
        log_notification(db, "line_notify", &token[..10], message, "success", None).await;
    } else {
        let error = format!("LINE Notify failed with status: {}", response.status());
        tracing::error!("{}", error);
        log_notification(db, "line_notify", &token[..10], message, "failed", Some(&error)).await;
        return Err(anyhow::anyhow!(error));
    }

    Ok(())
}

/// Send notification via Slack
pub async fn send_slack(db: &SqlitePool, message: &str) -> Result<(), anyhow::Error> {
    // Get Slack webhook URL from settings
    let webhook_url = match Setting::get(db, "slack_webhook_url").await? {
        Some(url) => url,
        None => {
            tracing::warn!("Slack webhook URL not configured");
            return Ok(());
        }
    };

    if webhook_url.is_empty() {
        tracing::warn!("Slack webhook URL is empty");
        return Ok(());
    }

    let client = Client::new();
    let response = client
        .post(&webhook_url)
        .json(&SlackPayload {
            text: message.to_string(),
        })
        .send()
        .await?;

    if response.status().is_success() {
        tracing::info!("Slack notification sent successfully");
        log_notification(db, "slack", "webhook", message, "success", None).await;
    } else {
        let error = format!("Slack notification failed with status: {}", response.status());
        tracing::error!("{}", error);
        log_notification(db, "slack", "webhook", message, "failed", Some(&error)).await;
        return Err(anyhow::anyhow!(error));
    }

    Ok(())
}

async fn log_notification(
    db: &SqlitePool,
    notification_type: &str,
    recipient: &str,
    message: &str,
    status: &str,
    error_message: Option<&str>,
) {
    let result = sqlx::query(
        "INSERT INTO notification_logs (notification_type, recipient, message, status, error_message)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(notification_type)
    .bind(recipient)
    .bind(message)
    .bind(status)
    .bind(error_message)
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::error!("Failed to log notification: {}", e);
    }
}
