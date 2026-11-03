use std::time::Duration;

use chrono::{DateTime, Local};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{RngExt, SeedableRng};

use crate::STATE;
use crate::data::config::{CompleteDefault, GuildConfig, GuildConfigRandom};
use crate::takedown::{TakedownResult, TakedownType, takedown};

#[derive(Debug, derive_more::From)]
pub enum CheckRandomError {
    Database(sqlx::Error)
}

#[tracing::instrument(skip_all)]
pub async fn handle_random_check() -> Result<(), CheckRandomError> {
    let mut rng = StdRng::from_rng(&mut rand::rng());
    let guild_configs = STATE.database.random_configs().await?;
    let mut due = Vec::new();
    for (guild_id, config) in guild_configs {
        let config_random = config.complete();

        if !config_random.min.is_finite() ||
            !config_random.max.is_finite() ||
            config_random.min <= 0.0 ||
            config_random.min > config_random.max ||
            config_random.max > 24.0 * 365.0
        {
            tracing::warn!(%guild_id, min = config_random.min, max = config_random.max,
                "invalid random interval");
            continue;
        }

        let now = Local::now();
        let next = now +
            Duration::from_secs_f64(
                rng.random_range(config_random.min..=config_random.max) * 3600.0
            );
        if !STATE.database.claim_random(guild_id, &config, now, next).await? {
            continue;
        }
        tracing::debug!(%guild_id, "claimed random takedown schedule");

        let guild_voice_states = STATE.cache.guild_voice_states(guild_id);
        let voice_user_count = guild_voice_states.as_ref().map_or(0, |states| states.len());
        let Some(guild_voice_states) = guild_voice_states.filter(|states| states.len() >= 3)
        else {
            tracing::debug!(%guild_id, voice_user_count, "skipping random takedown: insufficient voice users");
            continue;
        };

        let target_id = guild_voice_states.iter().choose(&mut rng).expect("at least 3");
        tracing::debug!(%guild_id, %target_id, voice_user_count, "selected random takedown target");
        due.push((*target_id, guild_id));
    }

    for (target_id, guild_id) in due {
        if let Err(error) = takedown(target_id, guild_id, TakedownType::Random).await {
            match error {
                crate::takedown::TakedownFail::Client(_) |
                crate::takedown::TakedownFail::Database(_) |
                crate::takedown::TakedownFail::Join(_) |
                crate::takedown::TakedownFail::Audio(_) => {
                    tracing::warn!(%guild_id, %target_id, ?error, "random takedown failed");
                }
                _ => {}
            }
        }
    }

    Ok(())
}
