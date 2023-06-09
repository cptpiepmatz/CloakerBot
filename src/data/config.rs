use twilight_model::id::marker::{ChannelMarker, RoleMarker};
use twilight_model::id::Id;

#[derive(Debug, Default)]
pub struct GuildConfig {
    pub danger_zones: Vec<Id<ChannelMarker>>,
    pub immunity_role: Option<Id<RoleMarker>>,
    pub log_channel: Option<Id<ChannelMarker>>,
    pub random_enabled: Option<bool>,
    pub random_min: Option<f64>,
    pub random_max: Option<f64>,
    pub target_bots: Option<bool>
}
