use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::data::Persist;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GuildConfig {
    pub danger_zones: Vec<Id<ChannelMarker>>,
    pub immunity: GuildConfigImmunity,
    pub log_channel: Option<Id<ChannelMarker>>,
    pub random: GuildConfigRandom,
    pub target_bots: Option<bool>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GuildConfigImmunity {
    pub general: Vec<Id<RoleMarker>>,
    pub random: Vec<Id<RoleMarker>>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GuildConfigRandom {
    pub enabled: Option<bool>,
    pub min: Option<f64>,
    pub max: Option<f64>
}

impl Persist for HashMap<Id<GuildMarker>, GuildConfig> {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/guild_configs.json";
}
