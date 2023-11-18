use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::STATE;

pub fn counter(guild_id: Id<GuildMarker>, countered_by_id: Id<UserMarker>) -> Result<Id<UserMarker>, ()> {
    STATE
        .planned_takedowns
        .apply(guild_id, |plan| match plan.countered_by_id {
            None => {
                let _ = plan.countered_by_id.insert(countered_by_id);
                Some(plan.target_id)
            },
            Some(_) => None,
        })
        .flatten()
        .ok_or(())
}
