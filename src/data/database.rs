use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use twilight_model::id::Id;

#[derive(Debug)]
pub struct Database {
    pub(super) pool: SqlitePool,
    path: PathBuf,
    pub(super) backup_lock: tokio::sync::Mutex<()>
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_owned();
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5))
            .disable_statement_logging();
        let pool = SqlitePoolOptions::new().max_connections(4).connect_lazy_with(options);
        Self { pool, path, backup_lock: Default::default() }
    }

    pub async fn initialize(&self) -> miette::Result<()> {
        use miette::{Context, IntoDiagnostic};

        if let Some(parent) = self.path.parent().filter(|path| !path.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent)
                .await
                .into_diagnostic()
                .wrap_err("creating the database directory")?;
        }
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .into_diagnostic()
            .wrap_err("initializing the database schema")?;
        Ok(())
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
