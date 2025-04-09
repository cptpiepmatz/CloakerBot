use serde_json::json;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::request::RequestBuilder;
use twilight_http::routing::Route;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_mention::Mention;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption,
    /* CommandInteractionDataResolved, */ CommandOptionValue /* InteractionChannel */
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::channel::ChannelType;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{ChannelMarker, CommandMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{ChannelBuilder, SubCommandBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::REPLIES;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component;
use crate::sync::OnceLockExpect;
use crate::util::template::Template;
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "log-channel";
pub const DESCRIPTION: &str = "Define your log channel.";
pub static MENTION: OnceLockExpect<MentionFormat<CommandMention>> = OnceLockExpect::new();

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
    let button = message_component::button::config::component(
        message_component::button::config::ConfigUnsetButton::LogChannel
    );

    let payload = json!({
        "type": InteractionResponseType::ChannelMessageWithSource,
        "data": {
            "flags": MessageFlags::EPHEMERAL,
            "content": REPLIES.config.log_channel.select.template([
                &BOT_USER_ID.mention(),
                // TODO: get fallback log channel here
                &"[insert channel here]"
            ]),
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

// #[deprecated]
// pub async fn old_action(
//     interaction: &Interaction,
//     LogChannelSubCommand { channel }: LogChannelSubCommand
// ) -> Result<(), HandleApplicationCommandError> { // check if the channel is
//   guild text or guild announcement if let Some(c) = &channel { let
//   (ChannelType::GuildText | ChannelType::GuildAnnouncement) = c.kind else {
//   let content = format!("{} is not a valid log channel.", c.id.mention());
//   let response_data = InteractionResponseDataBuilder::new() .content(content)
//   .flags(MessageFlags::EPHEMERAL) .build();

//             STATE
//                 .http
//                 .interaction(APPLICATION_ID)
//                 .create_response(
//                     interaction.id,
//                     interaction.token.as_str(),
//                     &InteractionResponse {
//                         kind:
// InteractionResponseType::ChannelMessageWithSource,
// data: Some(response_data)                     }
//                 )
//                 .await?;

//             return Ok(());
//         };
//     }

//     let guild_id =
// interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;     let mut
// guild_configs = STATE.guild_configs.write().await;     let config = &mut
// guild_configs.entry(guild_id).or_default().log_channel;
//     let current_log_channel_id = match &channel {
//         None => config.clone(),
//         Some(channel) => config.replace(channel.id)
//     };

//     let content = match (channel, current_log_channel_id) {
//         (None, None) => "No log channel set.".to_owned(),
//         (Some(c), None) => format!("Set {} as log channel.", c.id.mention()),
//         (None, Some(cc_id)) => format!("Currently {} is set as log channel.",
// cc_id.mention()),         (Some(c), Some(cc_id)) => format!(
//             "Set {} as log channel. Previously {} was set.",
//             c.id.mention(),
//             cc_id.mention()
//         )
//     };

//     let button = message_component::button::config::component(
//         message_component::button::config::ConfigUnsetButton::LogChannel
//     );
//     let response_data = InteractionResponseDataBuilder::new()
//         .content(content)
//         .flags(MessageFlags::EPHEMERAL)
//         .components([Component::ActionRow(ActionRow {
//             components: vec![button]
//         })])
//         .build();

//     STATE
//         .http
//         .interaction(APPLICATION_ID)
//         .create_response(
//             interaction.id,
//             interaction.token.as_str(),
//             &InteractionResponse {
//                 kind: InteractionResponseType::ChannelMessageWithSource,
//                 data: Some(response_data)
//             }
//         )
//         .await?;

//     Ok(())
// }
