use twilight_cache_inmemory::UpdateCache;
use twilight_mention::Mention;
use twilight_model::channel::ChannelType;
use twilight_model::gateway::payload::incoming::MemberAdd;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};

use crate::assets::templates::channel_logging;
use crate::util::ClientError;
use crate::{BOT_USER_ID, STATE};

pub async fn fallback_channel(
    guild_id: Id<GuildMarker>
) -> Result<Option<Id<ChannelMarker>>, ClientError> {
    cache_bot_member(guild_id).await?;

    let Some(guild) = STATE.cache.guild(guild_id)
    else {
        return Ok(None);
    };
    let mut candidates: Vec<_> = [guild.system_channel_id(), guild.public_updates_channel_id()]
        .into_iter()
        .flatten()
        .collect();
    drop(guild);

    if let Some(channels) = STATE.cache.guild_channels(guild_id) {
        let mut general: Vec<_> = channels
            .iter()
            .filter_map(|id| {
                let channel = STATE.cache.channel(*id)?;
                (channel.name.as_deref() == Some("general")).then_some(*id)
            })
            .collect();
        general.sort();
        candidates.extend(general);
    }

    Ok(candidates.into_iter().find(|id| can_log(*id)))
}

async fn cache_bot_member(guild_id: Id<GuildMarker>) -> Result<(), ClientError> {
    if STATE.cache.member(guild_id, BOT_USER_ID).is_none() {
        let member = STATE.client.guild_member(guild_id, BOT_USER_ID).await?.model().await?;
        STATE.cache.update(&MemberAdd { guild_id, member });
    }

    Ok(())
}

#[derive(Debug, derive_more::From)]
pub enum LogError {
    Client(ClientError),
    Http(twilight_http::Error),
    Database(sqlx::Error)
}

pub async fn log_takedown(
    guild_id: Id<GuildMarker>,
    target_id: Id<twilight_model::id::marker::UserMarker>,
    target_channel_id: Option<Id<ChannelMarker>>,
    kind: crate::takedown::TakedownType,
    result: &crate::takedown::TakedownResult
) -> Result<(), LogError> {
    use crate::takedown::{CounterOutcome, TakedownFail, TakedownType};

    if matches!(result, Err(TakedownFail::Immune | TakedownFail::NotFound)) {
        return Ok(());
    }
    let configured = STATE.database.guild_config(guild_id).await?.log_channel;
    let channel_id = match configured {
        Some(channel_id) => {
            cache_bot_member(guild_id).await?;
            if !can_log(channel_id) ||
                !STATE
                    .cache
                    .channel(channel_id)
                    .is_some_and(|channel| channel.guild_id == Some(guild_id))
            {
                tracing::debug!(%guild_id, %channel_id, "skipping discord log: configured channel is inaccessible or belongs to another guild");
                return Ok(());
            }
            channel_id
        }
        None => match fallback_channel(guild_id).await? {
            Some(channel_id) => channel_id,
            None => {
                tracing::debug!(%guild_id, "skipping discord log: no fallback channel available");
                return Ok(());
            }
        }
    };
    let source = match kind {
        TakedownType::Called { caller_id } => {
            channel_logging::source::called(caller_id.mention()).to_string()
        }
        TakedownType::Random => channel_logging::source::random().to_string(),
        TakedownType::DangerZone { channel_id } => {
            channel_logging::source::danger_zone(channel_id.mention()).to_string()
        }
    };
    let outcome = match result {
        Ok(_) => channel_logging::outcome::success().to_string(),
        Err(TakedownFail::Countered { by_id, outcome }) => match outcome {
            CounterOutcome::Parried => {
                channel_logging::outcome::parried(by_id.mention()).to_string()
            }
            CounterOutcome::Removed { user_id } => {
                channel_logging::outcome::countered(by_id.mention(), user_id.mention()).to_string()
            }
        },
        Err(TakedownFail::Fled) => channel_logging::outcome::fled().to_string(),
        Err(TakedownFail::Rejected) => channel_logging::outcome::rejected().to_string(),
        Err(TakedownFail::MissingPermissions) => {
            channel_logging::outcome::missing_permissions().to_string()
        }
        Err(TakedownFail::Join(_)) => channel_logging::outcome::join().to_string(),
        Err(TakedownFail::Audio(_)) => channel_logging::outcome::audio().to_string(),
        Err(TakedownFail::Client(_)) => channel_logging::outcome::client().to_string(),
        Err(TakedownFail::Database(_)) => channel_logging::outcome::database().to_string(),
        Err(TakedownFail::Immune | TakedownFail::NotFound) => unreachable!()
    };
    let target_channel = match target_channel_id {
        Some(channel_id) => channel_id.mention().to_string(),
        None => channel_logging::unknown_channel().to_string()
    };
    let content =
        channel_logging::takedown(target_channel, outcome, source, target_id.mention()).to_string();
    STATE.client.create_message(channel_id).content(&content).await?;
    Ok(())
}

fn can_log(channel_id: Id<ChannelMarker>) -> bool {
    let Some(channel) = STATE.cache.channel(channel_id)
    else {
        return false;
    };
    if !matches!(channel.kind, ChannelType::GuildText | ChannelType::GuildAnnouncement) {
        return false;
    }
    STATE.cache.permissions().in_channel(BOT_USER_ID, channel_id).is_ok_and(|permissions| {
        permissions.contains(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES)
    })
}

pub async fn fallback_description(guild_id: Id<GuildMarker>) -> String {
    match fallback_channel(guild_id).await {
        Ok(Some(channel)) => channel.mention().to_string(),
        Ok(None) => "unavailable (logging is disabled)".into(),
        Err(error) => {
            tracing::warn!(?error, "could not find fallback log channel");
            "unavailable (could not check permissions)".into()
        }
    }
}
