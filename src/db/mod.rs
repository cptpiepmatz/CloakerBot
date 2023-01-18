pub mod config;

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::future::Map;
use futures::FutureExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::method::Delete;
use surrealdb::sql::{Id, Output};
use surrealdb::Surreal;

use crate::STATE;

pub struct DataStore {
    db: Surreal<Db>
}

impl DataStore {
    pub async fn open() -> anyhow::Result<DataStore> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("cptpiepmatz").use_db("cloaker_bot").await?;
        Ok(DataStore { db })
    }
}

#[async_trait]
pub trait DataStoreEntry:
    Default + Clone + Send + Sync + Sized + Serialize + DeserializeOwned
{
    type Key: Send + Clone;

    fn cache_get(key: &Self::Key) -> Option<Self>;
    fn cache_set(key: Self::Key, value: Option<Self>);

    fn db_key(key: Self::Key) -> (String, Id);

    async fn fetch(key: Self::Key) -> anyhow::Result<Self> {
        if let Some(value) = Self::cache_get(&key) {
            return Ok(value);
        }
        let db = &STATE.ds.db;
        let from_db: Option<Self> = db.select(Self::db_key(key)).await?;
        Ok(from_db.unwrap_or_default())
    }

    async fn store(self, key: Self::Key) -> anyhow::Result<Option<Self>> {
        Self::cache_set(key.clone(), Some(self.clone()));
        let db = &STATE.ds.db;
        let from_db: Option<Self> = db.update(Self::db_key(key)).content(self).await?;
        Ok(from_db)
    }

    async fn delete(key: Self::Key) -> anyhow::Result<()> {
        Self::cache_set(key.clone(), None);
        let db = &STATE.ds.db;
        db.delete(Self::db_key(key)).await?;
        Ok(())
    }
}
