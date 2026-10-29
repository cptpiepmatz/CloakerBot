use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

use crate::data::persist::Persist;
use crate::util::ref_access_map::RefAccessMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnerNotified {
    pub allow_dms: Option<bool>,
    pub latest_msg_index: usize
}

pub type PersistentOwnerNotifications =
    RefAccessMap<Id<GuildMarker>, Id<UserMarker>, OwnerNotified>;
impl Persist for PersistentOwnerNotifications {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/owner_notifications.json";
}
