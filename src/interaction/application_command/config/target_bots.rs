use std::sync::Arc;

use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, SubCommandBuilder};

use crate::db::config::GuildConfig;
use crate::interaction::application_command::TryFromInteractionError;
use crate::State;

pub const NAME: &str = "target-bots";
const DESCRIPTION: &str = "Configure if the bot should target bots.";

pub struct TargetBotsSubCommand {
    allow: Option<bool>
}

impl TryFrom<&Vec<CommandDataOption>> for TargetBotsSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut allow: Option<bool> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "allow" => match &option.value {
                    CommandOptionValue::Boolean(b) => allow = Some(*b),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "allow".into(),
                            value: v.clone()
                        })
                    }
                },
                n => {
                    return Err(TryFromInteractionError::UnknownField {
                        command,
                        field: n.into()
                    })
                }
            }
        }
        Ok(TargetBotsSubCommand { allow })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(BooleanBuilder::new("allow", "Allow/disallow targeting bots.").build())
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    TargetBotsSubCommand { allow }: TargetBotsSubCommand
) -> anyhow::Result<()> {
    let mut config = GuildConfig::fetch(guild_id).await?;

    match allow {
        None => {
            let reply = match config.target_bots {
                true => "**🛠️ Taking down bots is *allowed* on this server.**",
                false => "**🛠️ Taking down bots is *disallowed* on this server.**"
            };
            super::super::update_defer(&interaction, reply).await?;
        }
        Some(allow) => {
            config.target_bots = allow;
            config.save(guild_id).await?;
            let reply = match allow {
                true => "**🛠️ *Allowed* taking down bots on this server.**",
                false => "**🛠️ *Disallowed* taking down bots on this server.**"
            };
            super::super::update_defer(&interaction, reply).await?;
        }
    }

    Ok(())
}
