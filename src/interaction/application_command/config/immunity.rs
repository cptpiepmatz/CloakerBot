use std::env::var;
use std::ops::Deref;
use std::sync::LazyLock;

use serde_json::json;
use twilight_cache_inmemory::model::message;
use twilight_http::request::RequestBuilder;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::routing::Route;
use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{CommandMarker, GuildMarker};
use twilight_model::oauth::scope::BOT;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::command::{StringBuilder, SubCommandBuilder};

use crate::data::config::{GuildConfig, GuildConfigImmunity};
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component::button::config::ConfigUnsetButton;
use crate::interaction::message_component::role_select::immunity::ImmunitySelect;
use crate::interaction::{application_command, message_component};
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "immunity";
pub const DESCRIPTION: &str = "Define what roles are immune to takedowns.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

pub enum ImmunitySubCommand {
    General,
    Random,
    GeneralSilent,
    RandomSilent
}

impl TryFrom<&Vec<CommandDataOption>> for ImmunitySubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut for_general = false;
        let mut for_random = false;
        let mut for_general_silent = false;
        let mut for_random_silent = false;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "type" => match &option.value {
                    CommandOptionValue::String(s) => match s.as_str() {
                        "general" => for_general = true,
                        "random" => for_random = true,
                        "general-silent" => for_general_silent = true,
                        "random-silent" => for_random_silent = true,
                        _ => {
                            return Err(TryFromInteractionError::IncorrectFieldValue {
                                command,
                                field: "type".into(),
                                value: option.value.clone()
                            });
                        }
                    },
                    _n => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "type".into(),
                            value: option.value.clone()
                        });
                    }
                },
                n => return Err(TryFromInteractionError::UnknownField { command, field: n.into() })
            }
        }

        match (for_general, for_random, for_general_silent, for_random_silent) {
            (true, _, _, _) => Ok(ImmunitySubCommand::General),
            (_, true, _, _) => Ok(ImmunitySubCommand::Random),
            (_, _, true, _) => Ok(ImmunitySubCommand::GeneralSilent),
            (_, _, _, true) => Ok(ImmunitySubCommand::RandomSilent),
            (false, false, false, false) => {
                Err(TryFromInteractionError::MissingField { command, field: "type".into() })
            }
        }
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            StringBuilder::new("type", "Select which type of immunity role.")
                .choices([
                    ("general".to_owned(), "general".to_owned()),
                    ("random".to_owned(), "random".to_owned()),
                    ("general-silent".to_owned(), "general-silent".to_owned()),
                    ("random-silent".to_owned(), "random-silent".to_owned())
                ])
                .required(true)
        )
        .build()
}

pub async fn action(
    interaction: &Interaction,
    command: ImmunitySubCommand
) -> Result<(), HandleApplicationCommandError> {
    let variant = match command {
        ImmunitySubCommand::General => ImmunitySelect::General,
        ImmunitySubCommand::Random => ImmunitySelect::Random,
        ImmunitySubCommand::GeneralSilent => ImmunitySelect::GeneralSilent,
        ImmunitySubCommand::RandomSilent => ImmunitySelect::RandomSilent
    };

    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let config = STATE.database.guild_config(guild_id).await?;
    let current = match variant {
        ImmunitySelect::General => config.immunity.general.as_slice(),
        ImmunitySelect::Random => config.immunity.random.as_slice(),
        ImmunitySelect::GeneralSilent => config.immunity.general_silent.as_slice(),
        ImmunitySelect::RandomSilent => config.immunity.random_silent.as_slice()
    };

    let role_select = message_component::role_select::immunity::component(variant, current);
    let button = match variant {
        ImmunitySelect::General => ConfigUnsetButton::ImmunityGeneral.component(),
        ImmunitySelect::Random => ConfigUnsetButton::ImmunityRandom.component(),
        ImmunitySelect::GeneralSilent => ConfigUnsetButton::ImmunityGeneralSilent.component(),
        ImmunitySelect::RandomSilent => ConfigUnsetButton::ImmunityRandomSilent.component()
    };

    use crate::assets::replies::config::immunity;
    let bot = BOT_USER_ID.mention();
    let cmd = application_command::help::MENTION.deref();
    let content = match variant {
        ImmunitySelect::General => immunity::general::select(bot, cmd),
        ImmunitySelect::Random => immunity::random::select(bot, cmd),
        ImmunitySelect::GeneralSilent => immunity::general::silent::select(bot, cmd),
        ImmunitySelect::RandomSilent => immunity::random::silent::select(bot, cmd)
    };

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .components([
            Component::ActionRow(ActionRow { id: None, components: vec![role_select] }),
            Component::ActionRow(ActionRow { id: None, components: vec![button] })
        ])
        .build();

    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response_data)
        })
        .await?;

    Ok(())
}
