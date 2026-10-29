use twilight_cache_inmemory::UpdateCache;
use twilight_mention::Mention;
use twilight_model::channel::ChannelType;
use twilight_model::gateway::payload::incoming::MemberAdd;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};

use crate::util::ClientError;
use crate::{BOT_USER_ID, STATE};

pub async fn fallback_channel(guild_id: Id<GuildMarker>) -> Result<Option<Id<ChannelMarker>>, ClientError> {
    if STATE.cache.member(guild_id, BOT_USER_ID).is_none() {
        let member = STATE.client.guild_member(guild_id, BOT_USER_ID).await?.model().await?;
        STATE.cache.update(&MemberAdd { guild_id, member });
    }

    let Some(guild) = STATE.cache.guild(guild_id) else {
        return Ok(None);
    };
    let mut candidates: Vec<_> = [guild.system_channel_id(), guild.public_updates_channel_id()]
        .into_iter().flatten().collect();
    drop(guild);

    if let Some(channels) = STATE.cache.guild_channels(guild_id) {
        let mut general: Vec<_> = channels.iter().filter_map(|id| {
            let channel = STATE.cache.channel(*id)?;
            (channel.name.as_deref() == Some("general")).then_some(*id)
        }).collect();
        general.sort();
        candidates.extend(general);
    }

    Ok(candidates.into_iter().find(|id| can_log(*id)))
}

fn can_log(channel_id: Id<ChannelMarker>) -> bool {
    let Some(channel) = STATE.cache.channel(channel_id) else { return false; };
    if !matches!(channel.kind, ChannelType::GuildText | ChannelType::GuildAnnouncement) {
        return false;
    }
    STATE.cache.permissions().in_channel(BOT_USER_ID, channel_id)
        .is_ok_and(|permissions| permissions.contains(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES))
}

pub async fn fallback_description(guild_id: Id<GuildMarker>) -> String {
    match fallback_channel(guild_id).await {
        Ok(Some(channel)) => channel.mention().to_string(),
        Ok(None) => "unavailable (logging is disabled)".into(),
        Err(error) => {
            macros::warn!("could not find fallback log channel: {error:?}");
            "unavailable (could not check permissions)".into()
        }
    }
}
