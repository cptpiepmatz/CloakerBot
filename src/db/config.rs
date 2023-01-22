use dashmap::DashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::db::{DataStoreEntry, OptionalDataStoreEntry};

pub const TABLE_NAME: &str = "config";

lazy_static! {
    static ref CACHE: DashMap<Id<GuildMarker>, GuildConfig> = DashMap::new();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    #[serde(skip_serializing)]
    #[serde(deserialize_with = "super::deserialize_guild_id")]
    pub id: Id<GuildMarker>,
    pub log_channel: Option<Id<ChannelMarker>>,
    pub danger_zones: Vec<Id<ChannelMarker>>,
    pub target_bots: bool,
    pub random: GuildConfigRandom,
    pub immunity: GuildConfigImmunity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfigRandom {
    pub enabled: bool,
    pub min: f64,
    pub max: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfigImmunity {
    pub general: Option<Id<RoleMarker>>,
    pub random: Option<Id<RoleMarker>>
}

impl OptionalDataStoreEntry for GuildConfig {
    type Key = Id<GuildMarker>;

    fn cache_get(key: &Self::Key) -> Option<Self> {
        CACHE.get(key).map(|r| r.clone())
    }

    fn cache_set(key: Self::Key, value: Option<Self>) {
        match value {
            Some(v) => {
                CACHE.insert(key, v);
            }
            None => {
                CACHE.remove(&key);
            }
        }
    }

    fn table_name() -> String {
        TABLE_NAME.into()
    }

    fn key(key: Self::Key) -> surrealdb::sql::Id {
        key.get().into()
    }
}

impl DataStoreEntry for GuildConfig {}

impl Default for GuildConfig {
    fn default() -> Self {
        GuildConfig {
            id: Id::new(1),
            log_channel: None,
            danger_zones: Vec::new(),
            target_bots: true,
            random: GuildConfigRandom {
                enabled: true,
                min: 4.0,
                max: 24.0
            },
            immunity: GuildConfigImmunity {
                general: None,
                random: None
            }
        }
    }
}
