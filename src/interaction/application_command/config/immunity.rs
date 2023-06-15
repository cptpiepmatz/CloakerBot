use std::env::var;

use serde_json::json;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::request::RequestBuilder;
use twilight_http::routing::Route;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::InteractionResponseType;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{StringBuilder, SubCommandBuilder};

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component;
use crate::interaction::message_component::role_select::immunity::ImmunitySelect;
use crate::STATE;

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
    let role_select = message_component::role_select::immunity::component(variant);
    let button = message_component::button::config::component(match variant {
        ImmunitySelect::General => {
            message_component::button::config::ConfigUnsetButton::ImmunityGeneral
        }
        ImmunitySelect::Random => {
            message_component::button::config::ConfigUnsetButton::ImmunityRandom
        }
    });

    let payload = json!({
        "type": InteractionResponseType::ChannelMessageWithSource,
        "data": {
            "flags": MessageFlags::EPHEMERAL,
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
