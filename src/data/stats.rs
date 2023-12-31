use std::collections::HashMap;
use std::process::Output;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::persist::Persist;

/// Statistics per guild.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GuildStats {
    /// Statistics about the takedowns on a guild.
    pub takedowns: GuildTakedowns
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GuildTakedowns(HashMap<Id<UserMarker>, MemberTakedowns>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MemberTakedowns(HashMap<Id<UserMarker>, u64>);

impl GuildTakedowns {
    /// Increment the takedown counter for the pair of caller and target.
    ///
    /// Returns the counter after incrementing it.
    pub fn increment(&mut self, caller_id: Id<UserMarker>, target_id: Id<UserMarker>) -> u64 {
        self.0.entry(caller_id).or_default().increment(target_id)
    }

    pub fn count(&self, caller_id: Id<UserMarker>) -> u64 {
        self.0
            .get(&caller_id)
            .map(|entry| entry.count())
            .unwrap_or_default()
    }

    pub fn get(&self, caller_id: Id<UserMarker>, target_id: Id<UserMarker>) -> u64 {
        self.0
            .get(&caller_id)
            .map(|entry| entry.get(target_id))
            .unwrap_or_default()
    }

    pub fn ranked(
        &self
    ) -> impl Iterator<Item = (Id<UserMarker>, impl Iterator<Item = (Id<UserMarker>, u64)>)> {
        self.0
            .iter()
            .map(|(id, takedowns)| (*id, takedowns.ranked(), takedowns.count()))
            .sorted_by(|(_, _, a), (_, _, b)| a.cmp(b))
            .map(|(id, iter, _)| (id, iter))
    }
}

impl MemberTakedowns {
    pub fn increment(&mut self, target_id: Id<UserMarker>) -> u64 {
        let entry = self.0.entry(target_id).or_default();
        *entry = *entry + 1;
        *entry
    }

    pub fn count(&self) -> u64 {
        self.0.values().sum()
    }

    pub fn get(&self, target_id: Id<UserMarker>) -> u64 {
        self.0.get(&target_id).copied().unwrap_or_default()
    }

    /// An iterator that ranks the takedown targets by their takedown count.
    pub fn ranked(&self) -> impl Iterator<Item = (Id<UserMarker>, u64)> {
        self.0
            .iter()
            .map(|(id, count)| (*id, *count))
            .sorted_by(|(_, a), (_, b)| a.cmp(b))
    }
}

pub type PersistentGuildStats = HashMap<Id<GuildMarker>, GuildStats>;
impl Persist for PersistentGuildStats {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/guild_stats.json";
}

#[cfg(test)]
mod tests {
    use twilight_model::id::marker::UserMarker;

    use super::*;

    const TARGET_ID: Id<UserMarker> = Id::new(123);
    const CALLER_ID: Id<UserMarker> = Id::new(456);

    #[test]
    fn increment_works() {
        let mut stats = GuildStats::default();
        assert!(stats.takedowns.0.get(&CALLER_ID).is_none());
        assert_eq!(stats.takedowns.increment(CALLER_ID, TARGET_ID), 1);
        assert_eq!(
            stats
                .takedowns
                .0
                .get(&CALLER_ID)
                .unwrap()
                .0
                .get(&TARGET_ID)
                .unwrap(),
            &1
        );
    }
}
