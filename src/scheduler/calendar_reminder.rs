use chrono::Utc;
use sqlx::SqlitePool;
use crate::db::models::{Calendar, Setting};
use crate::api::line_client::{LineClient, Message};

/// Check for upcoming calendar events and send reminders
pub async fn check_and_send_reminders(db: &SqlitePool) -> Result<(), anyhow::Error> {
    tracing::debug!("Checking for calendar reminders...");

    // Get all upcoming events in the next 24 hours that haven't had reminders sent
    let events = sqlx::query_as::<_, Calendar>(
        "SELECT * FROM calendars
         WHERE reminder_sent = 0
         AND event_time >= datetime('now')
         AND event_time <= datetime('now', '+24 hours')
         ORDER BY event_time ASC"
    )
    .fetch_all(db)
    .await?;

    if events.is_empty() {
        return Ok(());
    }

    tracing::info!("Found {} calendar events needing reminders", events.len());

    // Get LINE access token
    let access_token = match Setting::get(db, "line_channel_access_token").await? {
        Some(token) if !token.is_empty() => token,
        _ => {
            tracing::warn!("LINE channel access token not configured, skipping reminders");
            return Ok(());
        }
    };

    let line_client = LineClient::new(access_token);

    for event in events {
        match send_reminder(&line_client, db, &event).await {
            Ok(_) => {
                // Mark reminder as sent
                sqlx::query(
                    "UPDATE calendars SET reminder_sent = 1 WHERE id = ?"
                )
                .bind(event.id)
                .execute(db)
                .await?;

                tracing::info!("Sent reminder for event: {} to user {}", event.event_title, event.line_user_id);
            }
            Err(e) => {
                tracing::error!("Failed to send reminder for event {}: {}", event.id, e);
            }
        }
    }

    Ok(())
}

async fn send_reminder(
    line_client: &LineClient,
    _db: &SqlitePool,
    event: &Calendar,
) -> Result<(), anyhow::Error> {
    let event_time = chrono::DateTime::parse_from_rfc3339(&event.event_time)
        .map_err(|e| anyhow::anyhow!("Invalid event time: {}", e))?;

    let now = Utc::now();
    let time_until = event_time.signed_duration_since(now);

    let time_str = if time_until.num_hours() > 0 {
        format!("{}時間後", time_until.num_hours())
    } else {
        format!("{}分後", time_until.num_minutes())
    };

    let reminder_text = format!(
        "📅 イベントリマインダー\n\n「{}」が{}に開始されます。\n\n{}",
        event.event_title,
        time_str,
        event.event_description.as_deref().unwrap_or("詳細なし")
    );

    let messages = vec![Message::Text { text: reminder_text }];

    line_client.push_message(&event.line_user_id, messages).await?;

    Ok(())
}
