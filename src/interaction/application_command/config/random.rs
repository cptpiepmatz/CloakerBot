use std::borrow::Cow;
use std::ops::Deref;
use std::sync::LazyLock;

use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::channel::message::component::ActionRow;
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{CommandMarker, GuildMarker};
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::command::{BooleanBuilder, NumberBuilder, SubCommandBuilder};

use crate::assets::{self, QUOTES};
use crate::data::config::{CompleteDefault, GuildConfigRandom};
use crate::interaction::application_command::ApplicationCommand::Config;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    self, HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component::button::config::ConfigRandomButton;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "random";
pub const DESCRIPTION: &str = "Configure random takedown behavior.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

pub struct RandomSubCommand {
    min: Option<f64>,
    max: Option<f64>
}

impl TryFrom<&Vec<CommandDataOption>> for RandomSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "min" => match &option.value {
                    CommandOptionValue::Number(f) => min = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "min".into(),
                            value: v.clone()
                        });
                    }
                },
                "max" => match &option.value {
                    CommandOptionValue::Number(f) => max = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "max".into(),
                            value: v.clone()
                        });
                    }
                },
                n => return Err(TryFromInteractionError::UnknownField { command, field: n.into() })
            }
        }
        Ok(RandomSubCommand { min, max })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            NumberBuilder::new("min", "Minimum time between takedowns. `-1` to set default value.")
                .build()
        )
        .option(
            NumberBuilder::new("max", "Maximum time between takedowns. `-1` to set default value.")
                .build()
        )
        .build()
}

pub async fn action(
    interaction: &Interaction,
    RandomSubCommand { min, max }: RandomSubCommand
) -> Result<(), HandleApplicationCommandError> {
    use ConfigRandomButton as CRB;

    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    let random_config = &guild_config.random;

    let (enabled, mut min, mut max) = match (min, max) {
        (None, None) => (random_config.enabled, random_config.min, random_config.max),
        (Some(max), None) => (Some(true), Some(max), random_config.min),
        (None, Some(min)) => (Some(true), random_config.min, Some(min)),
        (Some(max), Some(min)) => (Some(true), Some(max), Some(min))
    };

    if matches!(min, Some(m) if m < 0.0) {
        min = None;
    }

    if matches!(max, Some(m) if m < 0.0) {
        max = None;
    }

    guild_config.random = GuildConfigRandom { enabled, min, max };

    {
        // remove schedule entry so that the next check can update
        let mut random_takedown_schedule = STATE.random_takedown_schedule.write().await;
        random_takedown_schedule.remove(&guild_id);
    }

    let complete_random = guild_config.random.complete();
    let content = match guild_config.random.enabled {
        None | Some(false) => {
            assets::replies::config::random::disabled(QUOTES.config.random.enabled)
        }
        Some(true) => assets::replies::config::random::enabled(
            application_command::help::MENTION.deref(),
            complete_random.max,
            complete_random.min,
            QUOTES.config.random.disabled
        )
    };

    let component = Component::ActionRow(ActionRow {
        components: vec![match complete_random.enabled {
            // use the inverse component to toggle config
            true => CRB::Disable.component(),
            false => CRB::Enable.component()
        }]
    });

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .components([component])
        .build();

    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response_data)
        })
        .await?;

    Ok(())
}
