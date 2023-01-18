use std::sync::Arc;

use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::State;

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

impl GuildConfig {
    pub async fn fetch(guild_id: Id<GuildMarker>, com: Arc<State>) -> anyhow::Result<GuildConfig> {
        let db_key = (TABLE_NAME, guild_id.get());
        let config_db: Option<GuildConfig> = com.db.select(db_key).await?;
        Ok(config_db.unwrap_or_default())
    }

    pub async fn save(self, guild_id: Id<GuildMarker>, com: Arc<State>) -> anyhow::Result<()> {
        let _: Option<GuildConfig> = com
            .db
            .update((TABLE_NAME, guild_id.get()))
            .content(self)
            .await?;

        Ok(())
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
