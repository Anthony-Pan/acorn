pub mod models;

use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::AppResult;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn initialize(db_path: &Path) -> AppResult<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        repair_migration_history(&pool).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

async fn repair_migration_history(pool: &SqlitePool) -> AppResult<()> {
    let table_exists: Option<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'")
            .fetch_optional(pool)
            .await?;
    if table_exists.is_none() {
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM _sqlx_migrations \
         WHERE version = 2 AND description = 'speech_providers'",
    )
    .execute(pool)
    .await?;

    Ok(())
}
