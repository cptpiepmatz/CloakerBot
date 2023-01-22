use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::takedown::TakedownType;
use crate::{Event, STATE};

pub async fn handle_danger_zone_join(
    guild_id: Id<GuildMarker>,
    _channel_id: Id<ChannelMarker>,
    target_id: Id<UserMarker>
) -> anyhow::Result<()> {
    STATE
        .event_sender
        .send(Event::TakedownInit {
            target_id,
            guild_id,
            kind: TakedownType::DangerZone
        })
        .await?;

    Ok(())
}
