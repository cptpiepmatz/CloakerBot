use std::collections::HashMap;

use chrono::{DateTime, Local};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::data::persist::Persist;

pub type PersistentTakedownSchedule = HashMap<Id<GuildMarker>, DateTime<Local>>;
impl Persist for PersistentTakedownSchedule {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/random_takedown_schedule.json";
}
