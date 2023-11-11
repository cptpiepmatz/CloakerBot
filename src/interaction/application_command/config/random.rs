use std::borrow::Cow;
use std::ops::Deref;

use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{CommandMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, NumberBuilder, SubCommandBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::{QUOTES, REPLIES};
use crate::data::config::GuildConfigRandom;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::ApplicationCommand::Config;
use crate::interaction::application_command::{
    self, HandleApplicationCommandError, TryFromInteractionError
};
use crate::sync::OnceLockExpect;
use crate::util::template::Template;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "random";
pub const DESCRIPTION: &str = "Configure random takedown behavior.";
pub static MENTION: OnceLockExpect<MentionFormat<CommandMention>> = OnceLockExpect::new();

pub const CLOAKER_QUOTE_TARGET_BOTS: &str =
    "They just gave a tiger a free pass in a playground. Better run, kids!";
pub const CLOAKER_QUOTE_NO_TARGET_BOTS: &str =
    "No more free-for-all fireworks. The party-poopers have spoken.";

pub struct RandomSubCommand {
    enable: Option<bool>,
    min: Option<f64>,
    max: Option<f64>
}

impl TryFrom<&Vec<CommandDataOption>> for RandomSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut enable: Option<bool> = None;
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "enable" => match &option.value {
                    CommandOptionValue::Boolean(b) => enable = Some(*b),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "allow".into(),
                            value: v.clone()
                        })
                    }
                },
                "min" => match &option.value {
                    CommandOptionValue::Number(f) => min = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "min".into(),
                            value: v.clone()
                        })
                    }
                },
                "max" => match &option.value {
                    CommandOptionValue::Number(f) => max = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "max".into(),
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
        Ok(RandomSubCommand { enable, min, max })
    }
}

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
    interaction: &Interaction,
    RandomSubCommand { enable, min, max }: RandomSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    let random = &guild_config.random;

    let (enabled, mut min, mut max) = match (enable, min, max) {
        (Some(false), _, _) => (Some(false), None, None),
        (Some(true), _, _) => (Some(true), random.max, random.min),
        (None, None, None) => (random.enabled, random.min, random.max),
        (_, Some(max), None) => (Some(true), Some(max), random.min),
        (_, None, Some(min)) => (Some(true), random.min, Some(min)),
        (_, Some(max), Some(min)) => (Some(true), Some(max), Some(min))
    };

    if matches!(min, Some(m) if m < 0.0) {
        min = None;
    }

    if matches!(max, Some(m) if m < 0.0) {
        max = None;
    }

    guild_config.random = GuildConfigRandom { enabled, min, max };

    fn format_f64(num: f64) -> String {
        let mut formatted = format!("{:.2}", num);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        // by truncating here we can avoid to allocate a new string
        formatted.truncate(trimmed.len());
        formatted
    }

    let content = match guild_config.random.enabled {
        None | Some(false) => REPLIES
            .config
            .random
            .disabled
            .template([&QUOTES.config.random.enabled]),
        Some(true) => REPLIES.config.random.enabled.template([
            // TODO: somehow show default values here
            &min.map(format_f64).unwrap_or("*[default min]*".into()),
            &max.map(format_f64).unwrap_or("*[default max]*".into()),
            application_command::help::MENTION.deref(),
            &QUOTES.config.random.disabled
        ])
    };

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .build();

    STATE
        .http
        .interaction(APPLICATION_ID)
        .create_response(
            interaction.id,
            interaction.token.as_str(),
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(response_data)
            }
        )
        .await?;

    Ok(())
}
