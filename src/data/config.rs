use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::data::Persist;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GuildConfig {
    pub danger_zones: Vec<Id<ChannelMarker>>,
    pub immunity_role: Option<Id<RoleMarker>>,
    pub log_channel: Option<Id<ChannelMarker>>,
    pub random_enabled: Option<bool>,
    pub random_min: Option<f64>,
    pub random_max: Option<f64>,
    pub target_bots: Option<bool>
}

impl Persist for HashMap<Id<GuildMarker>, GuildConfig> {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/guild_configs.json";
}
