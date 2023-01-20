use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::db::DataStoreEntry;

pub const TABLE_NAME: &str = "random";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRandom {
    #[serde(skip_serializing)]
    #[serde(deserialize_with = "super::deserialize_guild_id")]
    pub id: Id<GuildMarker>,
    pub next: DateTime<Utc>
}

impl DataStoreEntry for GuildRandom {
    type Key = Id<GuildMarker>;

    fn cache_get(_: &Self::Key) -> Option<Self> {
        None
    }

    fn cache_set(_: Self::Key, _: Option<Self>) {}

    fn table_name() -> String {
        TABLE_NAME.into()
    }

    fn key(key: Self::Key) -> surrealdb::sql::Id {
        key.get().into()
    }
}

impl Default for GuildRandom {
    fn default() -> Self {
        Self {
            id: Id::new(1),
            next: DateTime::from(SystemTime::now()) + chrono::Duration::hours(4)
        }
    }
}
