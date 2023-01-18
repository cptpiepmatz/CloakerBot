use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::db::config::GuildConfig;
use crate::db::DataStoreEntry;
use crate::STATE;

pub async fn user_is_valid_target(
    guild_id: Id<GuildMarker>,
    target_id: Id<UserMarker>,
    is_random: bool
) -> anyhow::Result<bool> {
    let GuildConfig {
        target_bots,
        immunity,
        ..
    } = GuildConfig::fetch(guild_id).await?;

    // user might be bot
    if !target_bots && crate::user_is_bot(target_id).await? {
        return Ok(false);
    }

    // user might be immune
    let immune_role_id = match is_random {
        true => immunity.random,
        false => immunity.general
    };
    if let Some(immune_role_id) = immune_role_id {
        let role_ids = match STATE.event_cache.member(guild_id, target_id) {
            Some(m) => Vec::from(m.roles()),
            None => STATE
                .http_cache
                .fetch_member(guild_id, target_id)
                .await?
                .roles
                .clone()
        };

        if role_ids.contains(&immune_role_id) {
            return Ok(false);
        }
    }

    Ok(true)
}
