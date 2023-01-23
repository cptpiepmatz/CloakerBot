pub mod config;
pub mod random;
pub mod takedown;

use async_trait::async_trait;
use serde::de::{DeserializeOwned, Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::sql::Id;
use surrealdb::Surreal;
use twilight_mention::ParseMention;
use twilight_model::id::marker::GuildMarker;

use crate::STATE;

pub const NAMESPACE: &str = "cptpiepmatz";
pub const NAME: &str = "cloaker_bot";

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
pub trait OptionalDataStoreEntry:
    Clone + Send + Sync + Sized + Serialize + DeserializeOwned
{
    type Key: Send + Clone;

    fn cache_get(key: &Self::Key) -> Option<Self>;
    fn cache_set(key: Self::Key, value: Option<Self>);

    fn table_name() -> String;
    fn key(key: Self::Key) -> Id;

    async fn fetch(key: Self::Key) -> anyhow::Result<Option<Self>> {
        if let Some(value) = Self::cache_get(&key) {
            return Ok(Some(value));
        }
        let db = &STATE.ds.db;
        let from_db: Option<Self> = db.select((Self::table_name(), Self::key(key))).await?;
        Ok(from_db)
    }

    async fn fetch_all() -> anyhow::Result<Vec<Self>> {
        let db = &STATE.ds.db;
        let from_db: Vec<Self> = db.select(Self::table_name()).await?;
        Ok(from_db)
    }

    async fn store(self, key: Self::Key) -> anyhow::Result<Option<Self>> {
        Self::cache_set(key.clone(), Some(self.clone()));
        let db = &STATE.ds.db;
        let from_db: Option<Self> = db
            .update((Self::table_name(), Self::key(key)))
            .content(self)
            .await?;
        Ok(from_db)
    }

    async fn delete(key: Self::Key) -> anyhow::Result<()> {
        Self::cache_set(key.clone(), None);
        let db = &STATE.ds.db;
        db.delete((Self::table_name(), Self::key(key))).await?;
        Ok(())
    }
}

#[async_trait]
pub trait DataStoreEntry: Default + OptionalDataStoreEntry {
    async fn fetch_or_default(key: Self::Key) -> anyhow::Result<Self> {
        Ok(Self::fetch(key).await?.unwrap_or_default())
    }
}

fn deserialize_guild_id<'de, D>(
    deserializer: D
) -> Result<twilight_model::id::Id<GuildMarker>, D::Error>
where
    D: Deserializer<'de>
{
    let concat_id = String::deserialize(deserializer)?;
    let split = concat_id.split(':').rev().next().ok_or_else(|| {
        D::Error::invalid_value(Unexpected::Str(&concat_id), &"with ':' split snowflake")
    })?;

    let id_num: u64 = split
        .parse()
        .map_err(|_| D::Error::invalid_value(Unexpected::Str(split), &"u64 parseable string"))?;

    let id: twilight_model::id::Id<GuildMarker> = twilight_model::id::Id::new_checked(id_num)
        .ok_or_else(|| D::Error::invalid_value(Unexpected::Unsigned(0), &"non-zero value"))?;

    Ok(id)
}
