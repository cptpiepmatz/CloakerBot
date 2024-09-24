use std::io;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};

use derive_more::From;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod config;
pub mod owner_notifications;
pub mod random;
pub mod stats;

#[derive(Debug, Default)]
pub struct PersistentData<T> {
    dirty: AtomicBool,
    value: RwLock<T>
}

use persist::Persist;

use crate::{util, STATE};

mod persist {
    // public interfaces require public types, this makes the `Persist` trait public
    // but still unavailable from outside this module

    use std::path::Path;
    use std::{fs, io};

    use async_trait::async_trait;
    use serde::de::DeserializeOwned;
    use serde::ser::Serialize;
    #[cfg(not(debug_assertions))]
    use serde_json::to_string as json_to_string;
    #[cfg(debug_assertions)]
    use serde_json::to_string_pretty as json_to_string;

    use crate::data::LoadPersistentDataError;

    #[async_trait]
    pub trait Persist: Serialize + DeserializeOwned {
        type Item: Persist;
        const DATA_PATH: &'static str;

        async fn load() -> Result<Self::Item, LoadPersistentDataError> {
            let file_content = tokio::fs::read_to_string(Self::DATA_PATH).await?;
            Ok(serde_json::from_str(&file_content)?)
        }

        async fn store(&self) -> io::Result<()> {
            let data_path = Path::new(Self::DATA_PATH);
            if let Some(parent_path) = data_path.parent() {
                fs::create_dir_all(parent_path)?;
            }
            fs::write(
                Self::DATA_PATH,
                json_to_string(self).expect("data should always be serializable")
            )
        }
    }
}

impl<T: Persist<Item = T> + Send + Sync> PersistentData<T> {
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.value.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        let guard = self.value.write().await;
        self.dirty.swap(true, Ordering::Release);
        guard
    }

    pub async fn update(&self, update: impl FnOnce(&mut T) -> bool) {
        let mut guard = self.value.write().await;
        let modified = update(guard.deref_mut());
        if modified {
            // this ensures no accidental false in dirty here
            self.dirty.swap(true, Ordering::Release);
        }
    }

    pub async fn load() -> Result<Option<PersistentData<T>>, LoadPersistentDataError> {
        let value = match T::load().await {
            Ok(value) => value,
            Err(LoadPersistentDataError::SerdeJson(err)) => return Err(err.into()),
            Err(LoadPersistentDataError::Io(err)) => {
                return match err.kind() {
                    io::ErrorKind::NotFound => Ok(None),
                    _ => Err(err.into())
                }
            }
        };
        Ok(Some(PersistentData {
            dirty: AtomicBool::new(false),
            value: RwLock::new(value)
        }))
    }

    pub async fn store(&self) -> io::Result<()> {
        // do not store data if the data is not dirty
        util::require!(self.dirty.load(Ordering::Relaxed) => Ok(()));

        // get exclusive guard, therefore write
        let guard = self.value.write().await;
        self.dirty.store(false, Ordering::Relaxed);
        guard.store().await
    }
}

#[derive(Debug, From)]
pub enum LoadPersistentDataError {
    SerdeJson(serde_json::Error),
    Io(io::Error)
}

pub async fn handle_persist_data() -> io::Result<()> {
    STATE.guild_configs.store().await?;
    STATE.guild_stats.store().await?;
    STATE.random_takedown_schedule.store().await?;
    STATE.owner_notifications.store().await?;

    Ok(())
}
