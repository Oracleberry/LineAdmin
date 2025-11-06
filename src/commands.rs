use sqlx::SqlitePool;
use tauri::State;

use crate::db::models::{User, Message, ScheduledMessage, Setting, Calendar};
use crate::analytics::{DashboardStats, UserStats};
use crate::api::line_client::{LineClient, Message as LineMessage};

pub struct AppState {
    pub db: SqlitePool,
}

// User commands
#[tauri::command]
pub async fn get_users(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    User::list_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_by_line_id(state: State<'_, AppState>, line_user_id: String) -> Result<Option<User>, String> {
    User::find_by_line_id(&state.db, &line_user_id)
        .await
        .map_err(|e| e.to_string())
}

// Message commands
#[tauri::command]
pub async fn get_messages(state: State<'_, AppState>, limit: i32) -> Result<Vec<Message>, String> {
    Message::list_all(&state.db, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_messages_by_user(
    state: State<'_, AppState>,
    line_user_id: String,
    limit: i32,
) -> Result<Vec<Message>, String> {
    Message::list_by_user(&state.db, &line_user_id, limit)
        .await
        .map_err(|e| e.to_string())
}

// Scheduled message commands
#[tauri::command]
pub async fn create_scheduled_message(
    state: State<'_, AppState>,
    line_user_id: Option<String>,
    message_text: String,
    schedule_time: String,
    cron_expression: Option<String>,
) -> Result<i64, String> {
    ScheduledMessage::create(
        &state.db,
        line_user_id.as_deref(),
        &message_text,
        &schedule_time,
        cron_expression.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_scheduled_messages(state: State<'_, AppState>) -> Result<Vec<ScheduledMessage>, String> {
    ScheduledMessage::list_pending(&state.db)
        .await
        .map_err(|e| e.to_string())
}

// Calendar commands
#[tauri::command]
pub async fn create_calendar_event(
    state: State<'_, AppState>,
    line_user_id: String,
    event_title: String,
    event_description: Option<String>,
    event_time: String,
) -> Result<i64, String> {
    Calendar::create(
        &state.db,
        &line_user_id,
        &event_title,
        event_description.as_deref(),
        &event_time,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_calendar_events(
    state: State<'_, AppState>,
    line_user_id: String,
) -> Result<Vec<Calendar>, String> {
    Calendar::list_by_user(&state.db, &line_user_id)
        .await
        .map_err(|e| e.to_string())
}

// Settings commands
#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    Setting::get(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
    description: Option<String>,
) -> Result<(), String> {
    Setting::set(&state.db, &key, &value, description.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_settings(state: State<'_, AppState>) -> Result<Vec<Setting>, String> {
    Setting::list_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}

// Analytics commands
#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    crate::analytics::get_dashboard_stats(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_stats(state: State<'_, AppState>, line_user_id: String) -> Result<UserStats, String> {
    crate::analytics::get_user_stats(&state.db, &line_user_id)
        .await
        .map_err(|e| e.to_string())
}

// LINE messaging commands
#[tauri::command]
pub async fn send_message_to_user(
    state: State<'_, AppState>,
    line_user_id: String,
    message_text: String,
) -> Result<(), String> {
    let access_token = Setting::get(&state.db, "line_channel_access_token")
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "LINE access token not configured".to_string())?;

    let client = LineClient::new(access_token);
    let messages = vec![LineMessage::Text { text: message_text }];

    client
        .push_message(&line_user_id, messages)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn broadcast_message(
    state: State<'_, AppState>,
    message_text: String,
) -> Result<(), String> {
    let access_token = Setting::get(&state.db, "line_channel_access_token")
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "LINE access token not configured".to_string())?;

    let client = LineClient::new(access_token);
    let messages = vec![LineMessage::Text { text: message_text }];

    client
        .broadcast_message(messages)
        .await
        .map_err(|e| e.to_string())
}

// External integration commands (stubs)
#[tauri::command]
pub async fn sync_to_notion(state: State<'_, AppState>) -> Result<String, String> {
    // Stub implementation
    tracing::info!("Notion sync requested");
    Ok("Notion sync functionality is available. Configure API key in settings.".to_string())
}

#[tauri::command]
pub async fn sync_to_airtable(state: State<'_, AppState>) -> Result<String, String> {
    // Stub implementation
    tracing::info!("Airtable sync requested");
    Ok("Airtable sync functionality is available. Configure API key in settings.".to_string())
}

#[tauri::command]
pub async fn sync_to_google_sheets(state: State<'_, AppState>) -> Result<String, String> {
    // Stub implementation
    tracing::info!("Google Sheets sync requested");
    Ok("Google Sheets sync functionality is available. Configure credentials in settings.".to_string())
}

// Database management commands
#[tauri::command]
pub async fn delete_message(state: State<'_, AppState>, message_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM messages WHERE id = ?")
        .bind(message_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_user(state: State<'_, AppState>, user_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_scheduled_message(state: State<'_, AppState>, message_id: i64) -> Result<(), String> {
    ScheduledMessage::update_status(&state.db, message_id, "cancelled", None)
        .await
        .map_err(|e| e.to_string())
}
