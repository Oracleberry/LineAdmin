pub mod calendar_reminder;

use chrono::Utc;
use sqlx::SqlitePool;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::db::models::{ScheduledMessage, Setting};

/// Initialize and start the scheduler
pub async fn init_scheduler(db: SqlitePool) -> Result<JobScheduler, anyhow::Error> {
    let scheduler = JobScheduler::new().await?;

    // Job to check and send scheduled messages every minute
    let db_clone = db.clone();
    let scheduled_job = Job::new_async("0 * * * * *", move |_uuid, _lock| {
        let db = db_clone.clone();
        Box::pin(async move {
            if let Err(e) = check_and_send_scheduled_messages(&db).await {
                tracing::error!("Failed to check scheduled messages: {}", e);
            }
        })
    })?;

    // Job to check calendar reminders every 5 minutes
    let db_clone2 = db.clone();
    let reminder_job = Job::new_async("0 */5 * * * *", move |_uuid, _lock| {
        let db = db_clone2.clone();
        Box::pin(async move {
            if let Err(e) = calendar_reminder::check_and_send_reminders(&db).await {
                tracing::error!("Failed to check calendar reminders: {}", e);
            }
        })
    })?;

    scheduler.add(scheduled_job).await?;
    scheduler.add(reminder_job).await?;
    scheduler.start().await?;

    tracing::info!("Scheduler started with scheduled messages and calendar reminders");

    Ok(scheduler)
}

/// Check for pending scheduled messages and send them
async fn check_and_send_scheduled_messages(db: &SqlitePool) -> Result<(), anyhow::Error> {
    let pending_messages = ScheduledMessage::list_pending(db).await?;
    let now = Utc::now();

    for message in pending_messages {
        // Parse schedule time
        let schedule_time = match chrono::DateTime::parse_from_rfc3339(&message.schedule_time) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => {
                tracing::warn!("Invalid schedule time format for message {}: {}", message.id, message.schedule_time);
                continue;
            }
        };

        // Check if it's time to send
        if schedule_time <= now {
            if let Err(e) = send_scheduled_message(db, &message).await {
                tracing::error!("Failed to send scheduled message {}: {}", message.id, e);
                ScheduledMessage::update_status(db, message.id, "failed", Some(&e.to_string())).await?;
            } else {
                ScheduledMessage::update_status(db, message.id, "sent", None).await?;
            }
        }
    }

    Ok(())
}

/// Send a scheduled message via LINE Messaging API
async fn send_scheduled_message(db: &SqlitePool, message: &ScheduledMessage) -> Result<(), anyhow::Error> {
    // Get LINE channel access token from settings
    let access_token = match Setting::get(db, "line_channel_access_token").await? {
        Some(token) => token,
        None => {
            return Err(anyhow::anyhow!("LINE channel access token not configured"));
        }
    };

    if access_token.is_empty() {
        return Err(anyhow::anyhow!("LINE channel access token is empty"));
    }

    let client = reqwest::Client::new();

    // Prepare message payload
    let payload = if let Some(user_id) = &message.line_user_id {
        // Send to specific user (push message)
        serde_json::json!({
            "to": user_id,
            "messages": [{
                "type": "text",
                "text": message.message_text
            }]
        })
    } else {
        // Broadcast message to all followers
        serde_json::json!({
            "messages": [{
                "type": "text",
                "text": message.message_text
            }]
        })
    };

    let endpoint = if message.line_user_id.is_some() {
        "https://api.line.me/v2/bot/message/push"
    } else {
        "https://api.line.me/v2/bot/message/broadcast"
    };

    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        tracing::info!("Scheduled message {} sent successfully", message.id);
    } else {
        let error_text = response.text().await?;
        tracing::error!("Failed to send scheduled message {}: {}", message.id, error_text);
        return Err(anyhow::anyhow!("LINE API error: {}", error_text));
    }

    Ok(())
}
