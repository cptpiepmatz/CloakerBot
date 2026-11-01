use thiserror::Error;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

use crate::STATE;
use crate::util::iterator_ext::IteratorExt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Ranks {
    pub victim_rank: u32,
    pub punisher_rank: u32,
    pub counter_rank: u32
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Times {
    pub times_smacked: u32,
    pub times_called: u32,
    pub times_countered: u32
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub global_ranks: Ranks,
    pub here_ranks: Ranks,
    pub global_times: Times,
    pub here_times: Times
}

bitflags::bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Badges: u8 {
        const TOP_STRIKER      = 1 << 0;
        const TOP_VICTIM       = 1 << 1;
        const NEVER_INITIATED  = 1 << 2;
        const NEVER_TAKEN_DOWN = 1 << 3;
        const IMMUNITY         = 1 << 4;
        const RANDOM_IMMUNITY  = 1 << 5;
        const TOP_COUNTERER    = 1 << 6;
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum CalculateRanksError {
    Database(#[from] sqlx::Error),
    Join(#[from] tokio::task::JoinError)
}

pub(crate) async fn calculate_ranks(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<Stats, CalculateRanksError> {
    let stats = STATE.database.stats().await?;

    // cpu-heavy rank calculations on a separate thread
    // keep the tokio worker free for other tasks
    let span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        span.in_scope(|| {
            let is_user = |(_, (uid, _)): &(u32, (Id<UserMarker>, u32))| *uid == user_id;
            type RankEntry = (u32, (Id<UserMarker>, u32));
            let match_rank = |rank: (Option<RankEntry>, Option<RankEntry>)| match rank {
                (None, None) => (1, 0),
                (None, Some((last_rank, _))) => (last_rank + 1, 0),
                (Some((rank, (_, count))), _) => (rank, count)
            };

            let global = {
                let takedowns = &stats.takedowns;

                let mut victim_ranks = takedowns.victim_ranks();
                let victim_rank = victim_ranks.find_and_last(is_user);
                let (victim_rank, times_smacked) = match_rank(victim_rank);

                let mut punisher_ranks = takedowns.punisher_ranks();
                let punisher_rank = punisher_ranks.find_and_last(is_user);
                let (punisher_rank, times_called) = match_rank(punisher_rank);

                let mut counter_ranks = takedowns.counter_ranks();
                let counter_rank = counter_ranks.find_and_last(is_user);
                let (counter_rank, times_countered) = match_rank(counter_rank);

                let ranks = Ranks { victim_rank, punisher_rank, counter_rank };
                let times = Times { times_smacked, times_called, times_countered };
                (ranks, times)
            };

            let here = match stats.takedowns.get(&guild_id) {
                None => {
                    let ranks = Ranks { victim_rank: 1, punisher_rank: 1, counter_rank: 1 };
                    let times = Times { times_smacked: 0, times_called: 0, times_countered: 0 };
                    (ranks, times)
                }
                Some(takedowns) => {
                    let mut victim_ranks = takedowns.victim_ranks();
                    let victim_rank = victim_ranks.find_and_last(is_user);
                    let (victim_rank, times_smacked) = match_rank(victim_rank);

                    let mut punisher_ranks = takedowns.punisher_ranks();
                    let punisher_rank = punisher_ranks.find_and_last(is_user);
                    let (punisher_rank, times_called) = match_rank(punisher_rank);

                    let mut counter_ranks = takedowns.counter_ranks();
                    let counter_rank = counter_ranks.find_and_last(is_user);
                    let (counter_rank, times_countered) = match_rank(counter_rank);

                    let ranks = Ranks { victim_rank, punisher_rank, counter_rank };
                    let times = Times { times_smacked, times_called, times_countered };
                    (ranks, times)
                }
            };

            Stats {
                global_ranks: global.0,
                here_ranks: here.0,
                global_times: global.1,
                here_times: here.1
            }
        })
    })
    .await
    .map_err(Into::into)
}

pub(crate) struct Immunity {
    general: bool,
    random: bool
}

pub(crate) async fn has_immunity(
    guild_id: Id<GuildMarker>,
    user_roles: &[Id<RoleMarker>]
) -> Result<Immunity, sqlx::Error> {
    let guild_config = STATE.database.guild_config(guild_id).await?;

    let contains_any = |immunity_roles: &[Id<RoleMarker>]| {
        user_roles.into_iter().any(|user_role| immunity_roles.contains(user_role))
    };

    let immunity = &guild_config.immunity;
    let general_immunity = contains_any(&immunity.general);
    let general_silent_immunity = contains_any(&immunity.general_silent);
    let random_immunity = contains_any(&immunity.random);
    let random_silent_immunity = contains_any(&immunity.random_silent);

    Ok(Immunity {
        general: general_immunity || general_silent_immunity,
        random: random_immunity || random_silent_immunity
    })
}

pub(crate) fn determine_badges(stats: &Stats, immunity: &Immunity) -> Badges {
    let mut badges = Badges::empty();

    if stats.here_ranks.punisher_rank == 1 && stats.here_times.times_called > 0 {
        badges.insert(Badges::TOP_STRIKER);
    }

    if stats.here_ranks.victim_rank == 1 && stats.here_times.times_smacked > 0 {
        badges.insert(Badges::TOP_VICTIM);
    }

    if stats.here_times.times_called == 0 {
        badges.insert(Badges::NEVER_INITIATED);
    }

    if stats.here_times.times_smacked == 0 {
        badges.insert(Badges::NEVER_TAKEN_DOWN);
    }

    if immunity.general {
        badges.insert(Badges::IMMUNITY);
    }

    if immunity.random {
        badges.insert(Badges::RANDOM_IMMUNITY);
    }

    if stats.here_ranks.counter_rank == 1 && stats.here_times.times_countered > 0 {
        badges.insert(Badges::TOP_COUNTERER);
    }

    badges
}
