use std::sync::Arc;

use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, NumberBuilder, SubCommandBuilder};

use crate::db::config::{GuildConfig, GuildConfigRandom};
use crate::interaction::application_command::config::guild_config;
use crate::interaction::application_command::update_defer;
use crate::{db, Com};

pub const NAME: &str = "random";
const DESCRIPTION: &str = "Configure random takedown behavior.";

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(BooleanBuilder::new("enable", "Enable/disable random takedowns.").build())
        .option(
            NumberBuilder::new(
                "min",
                "Minimum time between takedowns. `-1` to set default value."
            )
            .build()
        )
        .option(
            NumberBuilder::new(
                "max",
                "Maximum time between takedowns. `-1` to set default value."
            )
            .build()
        )
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    enable: Option<bool>,
    min: Option<f64>,
    max: Option<f64>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let mut config = guild_config(guild_id, com.clone()).await?;

    match (enable, min, max) {
        (Some(false), _, _) => {
            config.random.enabled = false;
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let reply = "**🛠️ *Disabled* random takedowns on this server.**";
            update_defer(&interaction, reply, com.clone()).await?;
        }
        (None, None, None) => {
            let reply = match config.random.enabled {
                false => String::from("**🛠️ Random takedowns are *disabled* on this server.**"),
                true => format!(
                    concat!(
                        "**🛠️ ",
                        "Random takedowns are *enabled* on this server.**\n",
                        "The time interval for random takedowns is set to {}-{}."
                    ),
                    config.random.min, config.random.max
                )
            };
            update_defer(&interaction, &reply, com.clone()).await?;
        }
        (_, min, max) => {
            let min = min.unwrap_or(config.random.min);
            let max = max.unwrap_or(config.random.max);
            config.random = GuildConfigRandom {
                enabled: true,
                min,
                max
            };
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let reply = format!(
                concat!(
                    "**🛠️ ",
                    "*Enabled* random takedowns on this server.**\n",
                    "The time interval for random takedowns is set to {}-{}."
                ),
                min, max
            );
            update_defer(&interaction, &reply, com.clone()).await?;
        }
    }

    Ok(())
}
