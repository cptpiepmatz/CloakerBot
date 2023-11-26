use std::collections::HashMap;

use chrono::{DateTime, Local};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::data::persist::Persist;

impl Persist for HashMap<Id<GuildMarker>, DateTime<Local>> {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/random_takedown_schedule.json";
}
