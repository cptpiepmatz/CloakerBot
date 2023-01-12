use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{ChannelMarker};
use twilight_model::id::Id;

pub const TABLE_NAME: &str = "config";

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildConfig {
    pub log_channel: Option<Id<ChannelMarker>>,
    pub danger_zones: Vec<Id<ChannelMarker>>,
    pub target_bots: bool,
    pub random: GuildConfigRandom
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildConfigRandom {
    pub enabled: bool,
    pub min: f64,
    pub max: f64
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
            }
        }
    }
}
