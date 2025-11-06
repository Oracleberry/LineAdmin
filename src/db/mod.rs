use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::PathBuf;

pub mod models;

pub async fn init_db(db_path: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    // Create database directory if it doesn't exist
    let path = PathBuf::from(db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| sqlx::Error::Io(e))?;
    }

    // Connect to database with create_if_missing option
    let db_url = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let migration_sql = include_str!("../../migrations/001_init.sql");

    sqlx::query(migration_sql)
        .execute(pool)
        .await?;

    Ok(())
}
