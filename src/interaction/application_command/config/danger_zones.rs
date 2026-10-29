use std::ops::Deref;
use std::sync::LazyLock;

use serde_json::json;
use twilight_http::request::RequestBuilder;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::routing::Route;
use twilight_mention::fmt::{CommandMention, Mention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::command::SubCommandBuilder;

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::{COMMANDS_REGISTER, application_command, message_component};
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "danger-zones";
pub const DESCRIPTION: &str = "Define your danger zones.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

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
    let guild_config = STATE.database.guild_config(guild_id).await?;
    let current = guild_config.danger_zones.as_slice();

    let channel_select = message_component::channel_select::danger_zones::component(&current);
    let button = message_component::button::config::ConfigUnsetButton::DangerZones.component();

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(crate::assets::replies::config::danger_zones::select(
            BOT_USER_ID.mention(),
            application_command::help::MENTION.deref()
        ))
        .components([
            Component::ActionRow(ActionRow { id: None, components: vec![channel_select] }),
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
