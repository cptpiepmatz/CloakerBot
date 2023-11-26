use std::time::Duration;

use chrono::{DateTime, Local};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};

use crate::data::config::{CompleteDefault, GuildConfig, GuildConfigRandom};
use crate::takedown::{takedown, TakedownResult, TakedownType};
use crate::STATE;

#[derive(Debug)]
pub enum CheckRandomError {}

pub async fn handle_random_check() -> Result<(), CheckRandomError> {
    let mut rng = StdRng::from_entropy();
    let guild_configs = STATE.guild_configs.read().await;
    let mut random_takedown_schedule = STATE.random_takedown_schedule.write().await;
    for (guild_id, config) in guild_configs.iter() {
        let config_random = config.random.complete();

        if !config_random.enabled {
            random_takedown_schedule.remove(guild_id);
            continue;
        }

        let schedule = random_takedown_schedule
            .entry(*guild_id)
            .or_insert_with(|| {
                let in_hours = rng.gen_range(config_random.min..=config_random.max);
                let in_hours = Duration::from_secs_f64(in_hours * 3600.0);
                Local::now() + in_hours
            });

        let now = Local::now();
        if now < *schedule {
            continue;
        }

        let Some(guild_voice_states) = (match STATE.cache.guild_voice_states(*guild_id) {
            None => None,
            Some(gvs) if gvs.len() < 3 => None,
            Some(gvs) => Some(gvs)
        })
        else {
            random_takedown_schedule.remove(guild_id);
            continue;
        };

        let target_id = guild_voice_states
            .iter()
            .choose(&mut rng)
            .expect("at least 3");
        // TODO: consider if this should return an error
        let _ = takedown(*target_id, *guild_id, TakedownType::Random).await;
        random_takedown_schedule.remove(guild_id);
    }

    Ok(())
}
