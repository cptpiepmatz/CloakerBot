use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::db::{DataStore, DataStoreEntry};
use crate::STATE;

pub const TABLE_NAME: &str = "config";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
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

impl DataStoreEntry for GuildConfig {
    type Key = Id<GuildMarker>;
    type Output = GuildConfig;

    fn db_key(key: Self::Key) -> (String, surrealdb::sql::Id) {
        (TABLE_NAME.into(), key.get().into())
    }
}

impl Default for GuildConfig {
    fn default() -> Self {
        GuildConfig {
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
