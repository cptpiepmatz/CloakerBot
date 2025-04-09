use std::collections::HashMap;
use std::ops::DerefMut;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::persist::Persist;

/// Statistics per guild.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuildStats {
    /// Statistics about the takedowns on a guild.
    pub takedowns: GuildTakedowns
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuildTakedowns(HashMap<Id<UserMarker>, MemberTakedowns>);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemberTakedowns(HashMap<Id<UserMarker>, u64>);

impl GuildTakedowns {
    /// Increment the takedown counter for the pair of caller and target.
    ///
    /// Returns the counter after incrementing it.
    pub fn increment(&mut self, caller_id: Id<UserMarker>, target_id: Id<UserMarker>) -> u64 {
        self.0.entry(caller_id).or_default().increment(target_id)
    }

    pub fn count(&self, caller_id: Id<UserMarker>) -> u64 {
        self.0.get(&caller_id).map(|entry| entry.count()).unwrap_or_default()
    }

    pub fn get(&self, caller_id: Id<UserMarker>, target_id: Id<UserMarker>) -> u64 {
        self.0.get(&caller_id).map(|entry| entry.get(target_id)).unwrap_or_default()
    }

    pub fn ranked(
        &self
    ) -> impl Iterator<Item = (Id<UserMarker>, impl Iterator<Item = (Id<UserMarker>, u64)>)> {
        self.0
            .iter()
            .map(|(id, takedowns)| (*id, takedowns.ranked(), takedowns.count()))
            .sorted_by(|(_, _, a), (_, _, b)| b.cmp(a))
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
        self.0.iter().map(|(id, count)| (*id, *count)).sorted_by(|(_, a), (_, b)| b.cmp(a))
    }
}

pub type PersistentGuildStats = HashMap<Id<GuildMarker>, GuildStats>;
impl Persist for PersistentGuildStats {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/guild_stats.json";
}

pub fn global_takedown_count<'s, I>(stats: I) -> HashMap<Id<UserMarker>, u64>
where
    I: Iterator<Item = &'s GuildStats>
{
    let mut takedown_count: HashMap<Id<UserMarker>, u64> = HashMap::new();

    for stats in stats {
        let guild_takedowns = &stats.takedowns;
        for &caller_id in guild_takedowns.0.keys() {
            *takedown_count.entry(caller_id).or_default().deref_mut() +=
                guild_takedowns.count(caller_id);
        }
    }

    takedown_count
}

#[cfg(test)]
mod tests {
    use twilight_model::id::marker::UserMarker;

    use super::*;

    const TARGET_ID: Id<UserMarker> = Id::new(123);
    const CALLER_ID: Id<UserMarker> = Id::new(456);

    const USER_1: Id<UserMarker> = Id::new(1);
    const USER_2: Id<UserMarker> = Id::new(2);
    const USER_3: Id<UserMarker> = Id::new(3);
    const USER_4: Id<UserMarker> = Id::new(4);
    const USER_5: Id<UserMarker> = Id::new(5);

    #[test]
    fn increment_works() {
        let mut stats = GuildStats::default();
        assert!(stats.takedowns.0.get(&CALLER_ID).is_none());
        assert_eq!(stats.takedowns.increment(CALLER_ID, TARGET_ID), 1);
        assert_eq!(stats.takedowns.0.get(&CALLER_ID).unwrap().0.get(&TARGET_ID).unwrap(), &1);
    }

    fn test_data() -> GuildTakedowns {
        let mut guild_takedowns = GuildTakedowns::default();

        macro_rules! takedowns {
            [$($caller:ident -> $target:ident = $count:literal),+] => {$(
                for _ in 0..$count {
                    guild_takedowns.increment($caller, $target);
                }
            )+};
        }

        takedowns![
            USER_1 -> USER_2 = 4,
            USER_1 -> USER_3 = 7,
            USER_2 -> USER_1 = 5,
            USER_2 -> USER_3 = 2,
            USER_3 -> USER_4 = 3,
            USER_4 -> USER_5 = 8,
            USER_5 -> USER_2 = 1
        ];

        guild_takedowns
    }

    #[test]
    fn count_works() {
        let guild_takedowns = test_data();
        assert_eq!(guild_takedowns.count(Id::new(999)), 0);
        assert_eq!(guild_takedowns.count(USER_1), 4 + 7);
        assert_eq!(guild_takedowns.count(USER_2), 5 + 2);
        assert_eq!(guild_takedowns.count(USER_3), 3);
        assert_eq!(guild_takedowns.count(USER_4), 8);
        assert_eq!(guild_takedowns.count(USER_5), 1);
    }

    #[test]
    fn ranked_works() {
        let guild_takedowns = test_data();
        let mut ranked = guild_takedowns.ranked().flat_map(|(caller_id, targets)| {
            targets.map(move |(target_id, count)| (caller_id, target_id, count))
        });

        macro_rules! ranked {
            [$($caller:ident -> $target:ident = $count:literal),+] => {$(
                assert_eq!(ranked.next(), Some(($caller, $target, $count)));
            )+}
        }

        ranked![
            USER_1 -> USER_3 = 7,
            USER_1 -> USER_2 = 4,
            USER_4 -> USER_5 = 8,
            USER_2 -> USER_1 = 5,
            USER_2 -> USER_3 = 2,
            USER_3 -> USER_4 = 3,
            USER_5 -> USER_2 = 1
        ];

        assert!(ranked.next().is_none());
    }
}
