use std::sync::LazyLock;

use serde_json::json;
use twilight_http::request::RequestBuilder;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::routing::Route;
use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandDataOption,
    /* CommandInteractionDataResolved, */ CommandOptionValue /* InteractionChannel */
};
use twilight_model::channel::ChannelType;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, CommandMarker, GuildMarker};
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::command::{ChannelBuilder, SubCommandBuilder};

use crate::assets::replies;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component;
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "log-channel";
pub const DESCRIPTION: &str = "Define your log channel.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

pub const CLOAKER_QUOTE_LOG_CHANNEL: &str =
    "I'd rather eat glass than report in, but hey, let’s not disappoint the bureaucracy.";

pub struct LogChannelSubCommand;

impl TryFrom<&Vec<CommandDataOption>> for LogChannelSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        if let Some(option) = options.iter().next() {
            return Err(TryFromInteractionError::UnknownField {
                command: format!("{} {}", super::NAME, NAME),
                field: String::from(&option.name)
            });
        }
        Ok(LogChannelSubCommand)
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION).build()
}

pub async fn action(
    interaction: &Interaction,
    _command: LogChannelSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let guild_config = STATE.guild_configs.read().await;
    let current = guild_config.get(&guild_id).map(|config| config.log_channel).flatten();

    let channel_select = message_component::channel_select::log_channel::component(current);
    let button = message_component::button::config::ConfigUnsetButton::LogChannel.component();

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(replies::config::log_channel::select(
            BOT_USER_ID.mention(),
            crate::logging::fallback_description(guild_id).await
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
