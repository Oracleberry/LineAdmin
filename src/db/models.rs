use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub line_user_id: String,
    pub display_name: Option<String>,
    pub picture_url: Option<String>,
    pub status_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub line_user_id: String,
    pub message_type: String,
    pub message_text: Option<String>,
    pub message_data: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledMessage {
    pub id: i64,
    pub line_user_id: Option<String>,
    pub message_text: String,
    pub schedule_time: String,
    pub cron_expression: Option<String>,
    pub status: String,
    pub sent_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Calendar {
    pub id: i64,
    pub line_user_id: String,
    pub event_title: String,
    pub event_description: Option<String>,
    pub event_time: String,
    pub reminder_sent: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationLog {
    pub id: i64,
    pub notification_type: String,
    pub recipient: String,
    pub message: String,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: String,
}

// Database operations for User
impl User {
    pub async fn create(pool: &SqlitePool, line_user_id: &str, display_name: Option<&str>) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO users (line_user_id, display_name) VALUES (?, ?)
             ON CONFLICT(line_user_id) DO UPDATE SET
             display_name = excluded.display_name,
             updated_at = CURRENT_TIMESTAMP"
        )
        .bind(line_user_id)
        .bind(display_name)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn find_by_line_id(pool: &SqlitePool, line_user_id: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE line_user_id = ?"
        )
        .bind(line_user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
    }
}

// Database operations for Message
impl Message {
    pub async fn create(
        pool: &SqlitePool,
        line_user_id: &str,
        message_type: &str,
        message_text: Option<&str>,
        message_data: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO messages (line_user_id, message_type, message_text, message_data)
             VALUES (?, ?, ?, ?)"
        )
        .bind(line_user_id)
        .bind(message_type)
        .bind(message_text)
        .bind(message_data)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn list_by_user(pool: &SqlitePool, line_user_id: &str, limit: i32) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE line_user_id = ? ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(line_user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn list_all(pool: &SqlitePool, limit: i32) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

// Database operations for ScheduledMessage
impl ScheduledMessage {
    pub async fn create(
        pool: &SqlitePool,
        line_user_id: Option<&str>,
        message_text: &str,
        schedule_time: &str,
        cron_expression: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO scheduled_messages (line_user_id, message_text, schedule_time, cron_expression)
             VALUES (?, ?, ?, ?)"
        )
        .bind(line_user_id)
        .bind(message_text)
        .bind(schedule_time)
        .bind(cron_expression)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn list_pending(pool: &SqlitePool) -> Result<Vec<ScheduledMessage>, sqlx::Error> {
        sqlx::query_as::<_, ScheduledMessage>(
            "SELECT * FROM scheduled_messages WHERE status = 'pending' ORDER BY schedule_time ASC"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE scheduled_messages SET status = ?, error_message = ?,
             sent_at = CASE WHEN ? = 'sent' THEN CURRENT_TIMESTAMP ELSE sent_at END,
             updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(status)
        .bind(error_message)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Database operations for Calendar
impl Calendar {
    pub async fn create(
        pool: &SqlitePool,
        line_user_id: &str,
        event_title: &str,
        event_description: Option<&str>,
        event_time: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO calendars (line_user_id, event_title, event_description, event_time)
             VALUES (?, ?, ?, ?)"
        )
        .bind(line_user_id)
        .bind(event_title)
        .bind(event_description)
        .bind(event_time)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn list_by_user(pool: &SqlitePool, line_user_id: &str) -> Result<Vec<Calendar>, sqlx::Error> {
        sqlx::query_as::<_, Calendar>(
            "SELECT * FROM calendars WHERE line_user_id = ? ORDER BY event_time ASC"
        )
        .bind(line_user_id)
        .fetch_all(pool)
        .await
    }
}

// Database operations for Setting
impl Setting {
    pub async fn set(pool: &SqlitePool, key: &str, value: &str, description: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO settings (key, value, description) VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP"
        )
        .bind(key)
        .bind(value)
        .bind(description)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
        let setting: Option<Setting> = sqlx::query_as::<_, Setting>(
            "SELECT * FROM settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;

        Ok(setting.map(|s| s.value))
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Setting>, sqlx::Error> {
        sqlx::query_as::<_, Setting>(
            "SELECT * FROM settings ORDER BY key ASC"
        )
        .fetch_all(pool)
        .await
    }
}
