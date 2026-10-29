use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnerNotified {
    pub allow_dms: Option<bool>,
    pub latest_msg_index: usize
}
