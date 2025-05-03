use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

use crate::STATE;

pub fn counter(
    guild_id: Id<GuildMarker>,
    countered_by_id: Id<UserMarker>
) -> Result<Id<UserMarker>, ()> {
    STATE
        .planned_takedowns
        .apply(guild_id, |plan| {
            match plan.counter_tx.take().map(|tx| tx.send(countered_by_id)).and_then(|res| res.ok())
            {
                Some(_) => Some(plan.target_id),
                None => None
            }
        })
        .flatten()
        .ok_or(())
}
