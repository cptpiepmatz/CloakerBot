use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::{cmp, u64};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

use crate::data::persist::Persist;
use crate::util::log::warn;

/// Statistics per guild.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuildStats {
    /// Statistics about the takedowns within a guild.
    pub takedowns: GuildTakedowns,
}

/// Statistics about takedowns in a guild.
///
/// This structure stores takedown statistics for each member of the guild.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuildTakedowns(HashMap<Id<UserMarker>, MemberTakedowns>);

/// Statistics about the takedowns of a member of a guild
///
/// Across multiple guilds, a user might have multiple of these entries.
/// Random takedowns are stored by the user id of the bot.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemberTakedowns {
    /// Stores successful takedowns.
    takedowns: HashMap<Id<UserMarker>, u64>,
    /// Stores successful counters.
    ///
    /// A counter is a successful takedown action in response to an opponent's takedown.
    counters: HashMap<Id<UserMarker>, u64>,
    /// Stores the number of successful parries.
    ///
    /// A parry is a successful defense against a takedown where the return action did not result
    /// in a takedown.
    parries: u64,
}

impl GuildTakedowns {
    /// Increments the takedown count for a caller targeting a specific victim.
    ///
    /// This method records a successful takedown initiated by the `caller_id` against the 
    /// `target_id`. 
    /// The takedown count for the target is incremented by 1.
    ///
    /// # Parameters
    /// - `caller_id`: The ID of the user initiating the takedown.
    /// - `target_id`: The ID of the user being taken down.
    ///
    /// # Returns
    /// The updated number of successful takedowns initiated by the `caller_id` against the 
    /// `target_id`.
    pub fn increment_takedown(
        &mut self,
        caller_id: Id<UserMarker>,
        target_id: Id<UserMarker>,
    ) -> u64 {
        let takedowns =
            self.0.entry(caller_id).or_default().takedowns.entry(target_id).or_default();
        *takedowns += 1;
        *takedowns
    }

    /// Increments the counter count for a member responding to an attack.
    ///
    /// This method records a successful counter performed by the `counterattacker_id` against the 
    /// `attacker_id`. 
    /// The counter count for the counterattacker is incremented by 1.
    ///
    /// # Parameters
    /// - `counterattacker_id`: The ID of the user performing the counter.
    /// - `attacker_id`: The ID of the user being countered.
    ///
    /// # Returns
    /// The updated number of successful counters performed by the `counterattacker_id` against the 
    /// `attacker_id`.
    pub fn increment_counter(
        &mut self,
        counterattacker_id: Id<UserMarker>,
        attacker_id: Id<UserMarker>,
    ) -> u64 {
        let counters =
            self.0.entry(counterattacker_id).or_default().counters.entry(attacker_id).or_default();
        *counters += 1;
        *counters
    }

    /// Increments the parry count for a member successfully defending a takedown.
    ///
    /// This method records a successful parry performed by the `parrier_id`. 
    /// The parry count for the member is incremented by 1.
    ///
    /// # Parameters
    /// - `parrier_id`: The ID of the user performing the parry.
    ///
    /// # Returns
    /// The updated number of successful parries performed by the `parrier_id`.
    pub fn increment_parry(&mut self, parrier_id: Id<UserMarker>) -> u64 {
        let parries = &mut self.0.entry(parrier_id).or_default().parries;
        *parries += 1;
        *parries
    }

    /// Returns an iterator over the punisher rank positions.
    ///
    /// The punisher rank indicates the number of successful takedowns initiated by each member. 
    /// This iterator provides the rank, user ID, and the number of successful takedowns initiated 
    /// by the user.
    ///
    /// The rank positions may have duplicates if multiple members share the same number of 
    /// successful takedowns.
    ///
    /// # Returns
    /// An iterator of tuples, where each tuple contains:
    /// - The rank position
    /// - The user ID
    /// - The score (number of successful takedowns initiated)
    pub fn punisher_ranks(&self) -> impl Iterator<Item = (u64, (Id<UserMarker>, u64))> {
        let mut takedown_counts: Vec<(Id<UserMarker>, u64)> = self
            .0
            .iter()
            .map(|(uid, MemberTakedowns { takedowns, .. })| (*uid, takedowns.values().sum()))
            .collect();
        takedown_counts.sort_by(|(_, a), (_, b)| b.cmp(a));

        RankIter::new(takedown_counts.into_iter(), |(_, score)| *score)
    }

    /// Returns an iterator over the victim rank positions.
    ///
    /// The victim rank indicates the number of times a user has been taken down, including both 
    /// direct takedowns and counters. 
    /// This iterator provides the rank, user ID, and the number of times the user has been taken 
    /// down.
    ///
    /// The rank positions may have duplicates if multiple users share the same number of takedowns.
    ///
    /// # Returns
    /// An iterator of tuples, where each tuple contains:
    /// - The rank position
    /// - The user ID
    /// - The score (number of times the user was taken down)
    pub fn victim_ranks(&self) -> impl Iterator<Item = (u64, (Id<UserMarker>, u64))> {
        let mut takedown_counts = HashMap::<Id<UserMarker>, u64>::with_capacity(self.0.len());
        for (_, member_takedowns) in self.0.iter() {
            for (&victim_id, &value) in member_takedowns.takedowns.iter() {
                let count = takedown_counts.entry(victim_id).or_default();
                *count += value;
            }

            for (&countered_id, &value) in member_takedowns.counters.iter() {
                let count = takedown_counts.entry(countered_id).or_default();
                *count += value;
            }
        }

        let mut takedown_counts = Vec::from_iter(takedown_counts);
        takedown_counts.sort_by(|(_, a), (_, b)| b.cmp(a));

        RankIter::new(takedown_counts.into_iter(), |(_, score)| *score)
    }

    /// Returns an iterator over the counter rank positions.
    ///
    /// The counter rank indicates the number of times a user successfully countered or parried an 
    /// incoming takedown. 
    /// This iterator provides the rank, user ID, and the number of successful counters and parries 
    /// performed by the user.
    ///
    /// The rank positions may have duplicates if multiple users share the same number of successful 
    /// counters and parries.
    ///
    /// # Returns
    /// An iterator of tuples, where each tuple contains:
    /// - The rank position
    /// - The user ID
    /// - The score (number of successful counters and parries)
    pub fn counter_ranks(&self) -> impl Iterator<Item = (u64, (Id<UserMarker>, u64))> {
        let mut counter_counts: Vec<(Id<UserMarker>, u64)> = self
            .0
            .iter()
            .map(|(&uid, MemberTakedowns { counters, parries, .. })| {
                (uid, counters.values().sum::<u64>() + parries)
            })
            .collect();
        counter_counts.sort_by(|(_, a), (_, b)| b.cmp(a));

        RankIter::new(counter_counts.into_iter(), |(_, score)| *score)
    }
}

pub type PersistentGuildStats = HashMap<Id<GuildMarker>, GuildStats>;
impl Persist for PersistentGuildStats {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/guild_stats.json";
}

struct RankIter<T, I, F>
where
    I: Iterator<Item = T>,
    F: Fn(&T) -> u64,
{
    iter: I,
    score_fn: F,

    prev_score: u64,
    curr_rank: u64,
}

impl<T, I, F> RankIter<T, I, F>
where
    I: Iterator<Item = T>,
    F: Fn(&T) -> u64,
{
    pub fn new(iter: I, score_fn: F) -> Self {
        Self { iter, score_fn, prev_score: u64::MAX, curr_rank: 0 }
    }
}

impl<T, I, F> Iterator for RankIter<T, I, F>
where
    I: Iterator<Item = T>,
    F: Fn(&T) -> u64,
{
    type Item = (u64, T);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;

        let score = (self.score_fn)(&next);
        match self.prev_score.cmp(&score) {
            cmp::Ordering::Less => warn!(format!(
                "received a higher score ({}) after a lower one ({}); input may not be sorted",
                score, self.prev_score
            )),
            cmp::Ordering::Greater => {
                // strictly lower score → move to next rank
                self.prev_score = score;
                self.curr_rank += 1;
            }
            cmp::Ordering::Equal => {
                // tie → same rank, nothing changes
            }
        }

        // now updated if the new value is smaller
        let rank = self.curr_rank;
        Some((rank, next))
    }
}

#[cfg(test)]
mod rank_iter_tests {
    use super::*;

    #[test]
    fn basic_descending_scores() {
        let scores = vec![100, 90, 90, 80];
        let result: Vec<_> = RankIter::new(scores.into_iter(), |&x| x).collect();
        let expected = vec![(1, 100), (2, 90), (2, 90), (3, 80)];
        assert_eq!(result, expected);
    }

    #[test]
    fn all_ties() {
        let scores = vec![50, 50, 50];
        let result: Vec<_> = RankIter::new(scores.into_iter(), |&x| x).collect();
        let expected = vec![(1, 50), (1, 50), (1, 50)];
        assert_eq!(result, expected);
    }

    #[test]
    fn unsorted_input_keeps_first_rank() {
        // warns on higher scores after lower ones, but still returns rank=1 always
        let scores = vec![1, 2, 3];
        let result: Vec<_> = RankIter::new(scores.into_iter(), |&x| x).collect();
        let expected = vec![(1, 1), (1, 2), (1, 3)];
        assert_eq!(result, expected);
    }

    #[test]
    fn empty_iterator() {
        let scores: Vec<u64> = vec![];
        let result: Vec<_> = RankIter::new(scores.into_iter(), |&x| x).collect();
        let expected: Vec<(u64, u64)> = vec![];
        assert_eq!(result, expected);
    }
}
