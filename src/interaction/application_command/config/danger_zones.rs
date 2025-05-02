use std::ops::Deref;

use serde_json::json;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::request::RequestBuilder;
use twilight_http::routing::Route;
use twilight_mention::fmt::{CommandMention, Mention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::InteractionResponseType;
use twilight_model::id::marker::CommandMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::SubCommandBuilder;

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::{application_command, message_component};
use crate::sync::OnceLockExpect;
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "danger-zones";
pub const DESCRIPTION: &str = "Define your danger zones.";
pub static MENTION: OnceLockExpect<MentionFormat<CommandMention>> = OnceLockExpect::new();

pub struct DangerZonesSubCommand;

impl TryFrom<&Vec<CommandDataOption>> for DangerZonesSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        if let Some(option) = options.iter().next() {
            return Err(TryFromInteractionError::UnknownField {
                command: format!("{} {}", super::NAME, NAME),
                field: String::from(&option.name)
            });
        }
        Ok(DangerZonesSubCommand)
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION).build()
}

pub async fn action(
    interaction: &Interaction,
    _command: DangerZonesSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let guild_config = STATE.guild_configs.read().await;
    let current = guild_config
        .get(&guild_id)
        .map(|config| config.danger_zones.as_slice())
        .unwrap_or_default();

    let channel_select = message_component::channel_select::danger_zones::component(&current);
    let button = message_component::button::config::component(
        message_component::button::config::ConfigUnsetButton::DangerZones
    );

    let payload = json!({
        "type": InteractionResponseType::ChannelMessageWithSource,
        "data": {
            "flags": MessageFlags::EPHEMERAL,
            "content": crate::assets::replies::config::danger_zones::select(
                BOT_USER_ID.mention(),
                application_command::help::MENTION.deref()
            ),
            "components": [
                {
                    "type": ComponentType::ActionRow,
                    "components": [channel_select]
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
    .use_authorization_token(true)
    .build()?;

    STATE.client.request::<CreateResponse<'_>>(request).await?;

    Ok(())
}
