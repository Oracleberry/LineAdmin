// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod commands;
mod db;
mod notification;
mod scheduler;
mod analytics;
mod integrations;

use std::net::SocketAddr;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "line_admin_app=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting LINE Admin App");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database path from environment or use default
    let db_path = std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| {
            let mut path = dirs::data_local_dir().unwrap_or_default();
            path.push("line_admin_app");
            std::fs::create_dir_all(&path).ok();
            path.push("database.db");
            path.to_string_lossy().to_string()
        });

    tracing::info!("Database path: {}", db_path);

    // Initialize database
    let db = db::init_db(&db_path).await?;
    tracing::info!("Database initialized");

    // Clone db for different uses
    let db_for_tauri = db.clone();
    let db_for_server = db.clone();
    let db_for_scheduler = db.clone();

    // Start scheduler
    let scheduler = scheduler::init_scheduler(db_for_scheduler).await?;
    tracing::info!("Scheduler initialized");

    // Start web server in background
    let server_port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    tokio::spawn(async move {
        start_web_server(db_for_server, server_port).await;
    });

    // Build Tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(commands::AppState { db: db_for_tauri })
        .invoke_handler(tauri::generate_handler![
            // User commands
            commands::get_users,
            commands::get_user_by_line_id,
            commands::delete_user,
            // Message commands
            commands::get_messages,
            commands::get_messages_by_user,
            commands::delete_message,
            // Scheduled message commands
            commands::create_scheduled_message,
            commands::get_scheduled_messages,
            commands::cancel_scheduled_message,
            // Calendar commands
            commands::create_calendar_event,
            commands::get_calendar_events,
            // Settings commands
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            // Analytics commands
            commands::get_dashboard_stats,
            commands::get_user_stats,
            // LINE messaging commands
            commands::send_message_to_user,
            commands::broadcast_message,
            // External integration commands
            commands::sync_to_notion,
            commands::sync_to_airtable,
            commands::sync_to_google_sheets,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

async fn start_web_server(db: sqlx::SqlitePool, port: u16) {
    let app = api::create_router(db);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Web server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind server");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
