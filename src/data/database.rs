use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use thiserror::Error;
use twilight_model::id::Id;

#[derive(Debug)]
pub struct Database {
    pub(super) pool: SqlitePool,
    path: PathBuf,
    pub(super) backup_lock: tokio::sync::Mutex<()>
}

#[derive(Debug, Error)]
pub enum InitDatabaseError {
    #[error("creating the database directory")]
    Directory(#[from] std::io::Error),

    #[error("initializing the database schema")]
    Schema(#[from] sqlx::migrate::MigrateError)
}

impl Database {
    pub async fn init(path: impl AsRef<Path>) -> Result<Self, InitDatabaseError> {
        let path = path.as_ref().to_owned();
        tracing::info!(path = %path.display(), "initializing database");
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5))
            .disable_statement_logging();
        let pool = SqlitePoolOptions::new().max_connections(4).connect_lazy_with(options);
        if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tracing::debug!("applying database migrations");
        sqlx::migrate!().run(&pool).await?;
        tracing::info!("database initialized; migrations complete");
        Ok(Self { pool, path, backup_lock: Default::default() })
    }
}

impl Database {
    pub(super) fn backup_path(&self) -> PathBuf {
        self.path.with_file_name("backup").join("cloaker.sqlite")
    }
}

pub(super) fn discord_id<M>(value: String) -> Result<Id<M>, sqlx::Error> {
    value.parse().map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

pub(super) fn decode_error(message: &'static str) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, message)))
}
