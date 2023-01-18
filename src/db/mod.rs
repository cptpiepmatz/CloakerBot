pub mod config;

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use async_trait::async_trait;
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

pub struct DataStoreFuture<T: Default + Send + Sync + DeserializeOwned> {
    phantom: PhantomData<T>
}

impl DataStore {
    pub async fn open() -> anyhow::Result<DataStore> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("cptpiepmatz").use_db("cloaker_bot").await?;
        Ok(DataStore { db })
    }
}

#[async_trait]
pub trait DataStoreEntry: Sized + Serialize {
    type Key: Send;
    type Output: Default + Send + Sync + DeserializeOwned;

    fn db_key(key: Self::Key) -> (String, Id);

    async fn select(key: Self::Key) -> anyhow::Result<Self::Output> {
        let db = &STATE.ds.db;
        let from_db: Option<Self::Output> = db.select(Self::db_key(key)).await?;
        Ok(from_db.unwrap_or_default())
    }

    async fn update(self, key: Self::Key) -> anyhow::Result<Option<Self::Output>> {
        let db = &STATE.ds.db;
        let from_db: Option<Self::Output> = db.update(Self::db_key(key)).content(self).await?;
        Ok(from_db)
    }

    async fn delete(key: Self::Key) -> anyhow::Result<()> {
        let db = &STATE.ds.db;
        db.delete(Self::db_key(key)).await?;
        Ok(())
    }
}
