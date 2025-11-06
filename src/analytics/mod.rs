use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_messages: i64,
    pub messages_today: i64,
    pub new_users_this_week: i64,
    pub pending_scheduled_messages: i64,
    pub upcoming_calendar_events: i64,
    pub message_types: Vec<MessageTypeCount>,
    pub hourly_activity: Vec<HourlyActivity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageTypeCount {
    pub message_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HourlyActivity {
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub display_name: Option<String>,
    pub message_count: i64,
    pub first_message: Option<String>,
    pub last_message: Option<String>,
    pub most_used_message_type: Option<String>,
}

pub async fn get_dashboard_stats(db: &SqlitePool) -> Result<DashboardStats, sqlx::Error> {
    // Total users
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await?;

    // Total messages
    let total_messages: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
        .fetch_one(db)
        .await?;

    // Messages today
    let messages_today: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE DATE(timestamp) = DATE('now')"
    )
    .fetch_one(db)
    .await?;

    // New users this week
    let new_users_this_week: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE created_at >= datetime('now', '-7 days')"
    )
    .fetch_one(db)
    .await?;

    // Pending scheduled messages
    let pending_scheduled: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scheduled_messages WHERE status = 'pending'"
    )
    .fetch_one(db)
    .await?;

    // Upcoming calendar events (next 7 days)
    let upcoming_events: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM calendars
         WHERE event_time >= datetime('now')
         AND event_time <= datetime('now', '+7 days')
         AND reminder_sent = 0"
    )
    .fetch_one(db)
    .await?;

    // Message type distribution
    let message_types = sqlx::query_as::<_, (String, i64)>(
        "SELECT message_type, COUNT(*) as count
         FROM messages
         GROUP BY message_type
         ORDER BY count DESC"
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|(message_type, count)| MessageTypeCount { message_type, count })
    .collect();

    // Hourly activity (messages by hour)
    let hourly_activity = sqlx::query_as::<_, (i32, i64)>(
        "SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count
         FROM messages
         WHERE DATE(timestamp) >= DATE('now', '-7 days')
         GROUP BY hour
         ORDER BY hour"
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|(hour, count)| HourlyActivity { hour, count })
    .collect();

    Ok(DashboardStats {
        total_users: total_users.0,
        total_messages: total_messages.0,
        messages_today: messages_today.0,
        new_users_this_week: new_users_this_week.0,
        pending_scheduled_messages: pending_scheduled.0,
        upcoming_calendar_events: upcoming_events.0,
        message_types,
        hourly_activity,
    })
}

pub async fn get_user_stats(db: &SqlitePool, user_id: &str) -> Result<UserStats, sqlx::Error> {
    let message_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE line_user_id = ?"
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    let first_message: Option<(String,)> = sqlx::query_as(
        "SELECT timestamp FROM messages WHERE line_user_id = ? ORDER BY timestamp ASC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    let last_message: Option<(String,)> = sqlx::query_as(
        "SELECT timestamp FROM messages WHERE line_user_id = ? ORDER BY timestamp DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    let most_used_type: Option<(String,)> = sqlx::query_as(
        "SELECT message_type FROM messages
         WHERE line_user_id = ?
         GROUP BY message_type
         ORDER BY COUNT(*) DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    let user = crate::db::models::User::find_by_line_id(db, user_id).await?;

    Ok(UserStats {
        user_id: user_id.to_string(),
        display_name: user.and_then(|u| u.display_name),
        message_count: message_count.0,
        first_message: first_message.map(|m| m.0),
        last_message: last_message.map(|m| m.0),
        most_used_message_type: most_used_type.map(|t| t.0),
    })
}
