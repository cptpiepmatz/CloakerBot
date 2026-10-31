use std::time::Duration;

use thiserror::Error;

use super::Database;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error)
}

impl Database {
    /// Publishes a complete standalone snapshot, leaving the previous one
    /// intact if generation or replacement fails.
    #[tracing::instrument(skip_all)]
    pub async fn backup(&self) -> Result<(), BackupError> {
        let _lock = self.backup_lock.lock().await;
        let destination = self.backup_path();
        let temporary = destination.with_extension("sqlite.tmp");
        tokio::fs::create_dir_all(destination.parent().expect("backup has a directory")).await?;
        match tokio::fs::remove_file(&temporary).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into())
        }
        let path = temporary.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "backup path is not UTF-8")
        })?;
        // maintenance statement with no schema-dependent result columns
        sqlx::query("VACUUM main INTO ?").bind(path).execute(&self.pool).await?;
        let file = tokio::fs::OpenOptions::new().write(true).open(&temporary).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&temporary, &destination).await?;
        tracing::debug!(path = %destination.display(), "Database snapshot published");
        Ok(())
    }

    pub async fn maintain_backups(&self, every: Duration) {
        let mut interval = tokio::time::interval(every);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Err(error) = self.backup().await {
                tracing::warn!(%error, "Database snapshot failed; retaining the previous backup");
            }
        }
    }
}
