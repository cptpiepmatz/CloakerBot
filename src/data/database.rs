use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use twilight_model::id::Id;

#[derive(Debug)]
pub struct Database {
    pub(super) pool: SqlitePool,
    path: PathBuf
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_owned();
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5))
            .disable_statement_logging();
        let pool = SqlitePoolOptions::new().max_connections(4).connect_lazy_with(options);
        Self { pool, path }
    }

    pub async fn initialize(&self) -> miette::Result<()> {
        use miette::{Context, IntoDiagnostic};

        if let Some(parent) = self.path.parent().filter(|path| !path.as_os_str().is_empty()) {
            let mut directory = tokio::fs::DirBuilder::new();
            directory.recursive(true);
            #[cfg(unix)]
            directory.mode(0o700);
            directory.create(parent).await.into_diagnostic().wrap_err("creating the database directory")?;
        }
        let mut file = tokio::fs::OpenOptions::new();
        file.write(true).create_new(true);
        #[cfg(unix)]
        file.mode(0o600);
        match file.open(&self.path).await {
            Ok(file) => drop(file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error).into_diagnostic().wrap_err("creating the database")
        }
        sqlx::migrate!().run(&self.pool).await.into_diagnostic().wrap_err("initializing the database schema")?;
        Ok(())
    }
}

pub(super) fn discord_id<M>(value: String) -> Result<Id<M>, sqlx::Error> {
    value.parse().map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

pub(super) fn decode_error(message: &'static str) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, message)))
}
