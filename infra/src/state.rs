use std::ops::{Deref, DerefMut};

use twilight_gateway::{Intents, Shard, ShardId};
use twilight_model::channel::message::AllowedMentions;

use crate::sync::Eject;
use crate::util::cache::ObservableCache;

pub struct State<T> {
    pub discord_shard: Eject<Shard>,
    pub discord_client: twilight_http::client::Client,
    pub discord_cache: ObservableCache,
    pub http_client: reqwest::Client,
    custom: T
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

impl<T> State<T> {
    pub fn new(
        dependencies: &impl StateDependencies,
        custom: impl FnOnce(State<()>) -> Self
    ) -> Self {
        let shard =
            Shard::new(ShardId::ONE, dependencies.discord_token(), dependencies.discord_intents());

        let discord_client = twilight_http::client::Client::builder()
            .token(dependencies.discord_token())
            .default_allowed_mentions(AllowedMentions::default())
            .build();

        custom(State {
            discord_shard: Eject::new(shard),
            discord_client,
            discord_cache: ObservableCache::new(),
            http_client: reqwest::Client::new(),
            custom: ()
        })
    }
}

impl State<()> {
    pub fn with<T>(self, custom: T) -> State<T> {
        State {
            custom,
            discord_shard: self.discord_shard,
            discord_client: self.discord_client,
            discord_cache: self.discord_cache,
            http_client: self.http_client
        }
    }
}

pub trait StateDependencies {
    fn discord_token(&self) -> String;
    fn discord_intents(&self) -> Intents;
}
