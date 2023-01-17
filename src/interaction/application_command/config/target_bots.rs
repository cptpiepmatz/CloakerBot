use std::sync::Arc;

use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, SubCommandBuilder};

use crate::db::config::GuildConfig;
use crate::interaction::application_command::config::guild_config;
use crate::interaction::application_command::update_defer;
use crate::{db, Com};

pub const NAME: &str = "target-bots";
const DESCRIPTION: &str = "Configure if the bot should target bots.";

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(BooleanBuilder::new("allow", "Allow/disallow targeting bots.").build())
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    allow: Option<bool>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let mut config = guild_config(guild_id, com.clone()).await?;

    match allow {
        None => {
            let reply = match config.target_bots {
                true => "**🛠️ Taking down bots is *allowed* on this server.**",
                false => "**🛠️ Taking down bots is *disallowed* on this server.**"
            };
            update_defer(&interaction, reply, com.clone()).await?;
        }
        Some(allow) => {
            config.target_bots = allow;
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let reply = match allow {
                true => "**🛠️ *Allowed* taking down bots on this server.**",
                false => "**🛠️ *Disallowed* taking down bots on this server.**"
            };
            update_defer(&interaction, reply, com.clone()).await?;
        }
    }

    Ok(())
}
