use std::ops::Deref;

use tokio::sync::mpsc::{Receiver as TokioMpscReceiver, Sender as TokioMpscSender};
use twilight_cache_inmemory::{InMemoryCache, UpdateCache};
use twilight_model::gateway::event::{Event as DiscordEvent, Event};
use twilight_model::gateway::payload::incoming::VoiceStateUpdate;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};

use crate::util;

#[derive(Debug)]
pub struct ObservableCache {
    inner: InMemoryCache,
    emitter: TokioMpscSender<CacheUpdateEvent>
}

#[derive(Debug)]
pub enum CacheUpdateEvent {
    VoiceChannelUpdate(VoiceChannelCacheUpdate)
}

#[derive(Debug)]
pub struct VoiceChannelCacheUpdate {
    pub guild_id: Id<GuildMarker>,
    pub user_id: Id<UserMarker>,
    pub old: Option<Id<ChannelMarker>>,
    pub new: Option<Id<ChannelMarker>>
}

impl Deref for ObservableCache {
    type Target = InMemoryCache;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ObservableCache {
    pub fn new() -> (Self, TokioMpscReceiver<CacheUpdateEvent>) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        (Self { inner: Default::default(), emitter: tx }, rx)
    }

    pub async fn observe(&self, event: &DiscordEvent) {
        match event {
            Event::VoiceStateUpdate(update) => self.handle_voice_state_update(update).await,
            _ => {}
        }

        self.inner.update(event);
    }

    async fn handle_voice_state_update(&self, update: &VoiceStateUpdate) {
        let user_id = update.user_id;
        let guild_id = util::require!(Some(update.guild_id));

        let event_channel_id = update.channel_id;
        let cache_channel_id =
            self.inner.voice_state(user_id, guild_id).map(|cache_ref| cache_ref.channel_id());

        match (cache_channel_id, event_channel_id) {
            (None, None) => {}
            (Some(old), Some(new)) if old == new => {}
            (old, new) => {
                let _ = self
                    .emitter
                    .send(CacheUpdateEvent::VoiceChannelUpdate(VoiceChannelCacheUpdate {
                        guild_id,
                        user_id,
                        old,
                        new
                    }))
                    .await;
            }
        }
    }
}
