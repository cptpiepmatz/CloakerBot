use twilight_model::voice::VoiceState;

use crate::db::config::GuildConfig;
use crate::db::DataStoreEntry;
use crate::{Event, BOT_USER_ID, STATE};

pub async fn handle_voice_state(voice_state: VoiceState) -> anyhow::Result<()> {
    let VoiceState {
        guild_id,
        channel_id,
        user_id,
        ..
    } = voice_state;

    if user_id == BOT_USER_ID {
        return Ok(());
    }

    if let (Some(guild_id), Some(channel_id)) = (guild_id, channel_id) {
        let danger_zones = GuildConfig::fetch_or_default(guild_id).await?;
        if danger_zones.danger_zones.contains(&channel_id) {
            STATE
                .event_sender
                .send(Event::DangerZoneJoin {
                    target_id: user_id,
                    guild_id,
                    channel_id
                })
                .await?;
        }
    }

    Ok(())
}
