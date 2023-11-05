use std::env::var;
use std::ops::Deref;

use serde_json::json;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::request::RequestBuilder;
use twilight_http::routing::Route;
use twilight_mention::Mention;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::InteractionResponseType;
use twilight_model::id::marker::{CommandMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{StringBuilder, SubCommandBuilder};

use crate::assets::REPLIES;
use crate::data::config::{GuildConfig, GuildConfigImmunity};
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component::role_select::immunity::ImmunitySelect;
use crate::interaction::{application_command, message_component};
use crate::sync::OnceLockExpect;
use crate::util::template::Template;
use crate::{BOT_USER_ID, STATE};

pub const NAME: &str = "immunity";
pub const DESCRIPTION: &str = "Define what roles are immune to takedowns.";

pub enum ImmunitySubCommand {
    General,
    Random
}

impl TryFrom<&Vec<CommandDataOption>> for ImmunitySubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut for_general = false;
        let mut for_random = false;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "type" => match &option.value {
                    CommandOptionValue::String(s) => match s.as_str() {
                        "general" => for_general = true,
                        "random" => for_random = true,
                        _ => {
                            return Err(TryFromInteractionError::IncorrectFieldValue {
                                command,
                                field: "type".into(),
                                value: option.value.clone()
                            })
                        }
                    },
                    _n => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "type".into(),
                            value: option.value.clone()
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

        match (for_general, for_random) {
            (true, _) => Ok(ImmunitySubCommand::General),
            (_, true) => Ok(ImmunitySubCommand::Random),
            (false, false) => Err(TryFromInteractionError::MissingField {
                command,
                field: "type".into()
            })
        }
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            StringBuilder::new("type", "Select which type of immunity role.")
                .choices([
                    ("general".to_owned(), "general".to_owned()),
                    ("random".to_owned(), "random".to_owned())
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
        ImmunitySubCommand::Random => ImmunitySelect::Random
    };

    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let guild_configs = STATE.guild_configs.read().await;
    let current = match guild_configs.get(&guild_id) {
        None => &[],
        Some(GuildConfig {
            immunity: GuildConfigImmunity { general, random },
            ..
        }) => match variant {
            ImmunitySelect::General => general.as_slice(),
            ImmunitySelect::Random => random.as_slice()
        }
    };

    let role_select = message_component::role_select::immunity::component(variant, current);
    let button = message_component::button::config::component(match variant {
        ImmunitySelect::General => {
            message_component::button::config::ConfigUnsetButton::ImmunityGeneral
        }
        ImmunitySelect::Random => {
            message_component::button::config::ConfigUnsetButton::ImmunityRandom
        }
    });

    let content = (match variant {
        ImmunitySelect::General => REPLIES.config.immunity.general.select,
        ImmunitySelect::Random => REPLIES.config.immunity.random.select
    })
    .template([
        &BOT_USER_ID.mention(),
        application_command::help::MENTION.deref()
    ]);

    let payload = json!({
        "type": InteractionResponseType::ChannelMessageWithSource,
        "data": {
            "flags": MessageFlags::EPHEMERAL,
            "content": content,
            "components": [
                {
                    "type": ComponentType::ActionRow,
                    "components": [role_select]
                },
                {
                    "type": ComponentType::ActionRow,
                    "components": [button]
                }
            ]
        }
    });

    let request = RequestBuilder::new(&Route::InteractionCallback {
        interaction_id: interaction.id.into(),
        interaction_token: interaction.token.as_str()
    })
    .json(&payload)
    .expect("payload is serializable")
    .use_authorization_token(true)
    .build();

    STATE.http.request::<CreateResponse<'_>>(request).await?;

    Ok(())
}
