use twilight_mention::Mention;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandInteractionDataResolved, CommandOptionValue, InteractionChannel
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::ActionRow;
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::channel::ChannelType;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{ChannelBuilder, SubCommandBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "log-channel";
pub const DESCRIPTION: &str = "Define your log channel.";

pub struct LogChannelSubCommand {
    channel: Option<InteractionChannel>
}

impl
    TryFrom<(
        &Vec<CommandDataOption>,
        &Option<CommandInteractionDataResolved>
    )> for LogChannelSubCommand
{
    type Error = TryFromInteractionError;

    fn try_from(
        options: (
            &Vec<CommandDataOption>,
            &Option<CommandInteractionDataResolved>
        )
    ) -> Result<Self, Self::Error> {
        let mut channel_id: Option<Id<ChannelMarker>> = None;
        let mut channel: Option<InteractionChannel> = None;

        let command = format!("{} {}", super::NAME, NAME);
        for option in options.0 {
            match option.name.as_str() {
                "channel" => match &option.value {
                    CommandOptionValue::Channel(c) => channel_id = Some(*c),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "channel".into(),
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

        if let (Some(channel_id), Some(resolved)) = (channel_id, &options.1) {
            if let Some(int_channel) = resolved.channels.get(&channel_id) {
                channel = int_channel.clone().into();
            }
        }

        Ok(LogChannelSubCommand { channel })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(ChannelBuilder::new("channel", "Channel to output logging data.").build())
        .build()
}

pub async fn action(
    interaction: &Interaction,
    LogChannelSubCommand { channel }: LogChannelSubCommand
) -> Result<(), HandleApplicationCommandError> {
    // check if the channel is guild text or guild announcement
    if let Some(c) = &channel {
        let (ChannelType::GuildText | ChannelType::GuildAnnouncement) = c.kind else {
            let content = format!("{} is not a valid log channel.", c.id.mention());
            let response_data = InteractionResponseDataBuilder::new()
                .content(content)
                .flags(MessageFlags::EPHEMERAL)
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

            return Ok(())
        };
    }

    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let config = &mut guild_configs.entry(guild_id).or_default().log_channel;
    let current_log_channel_id = match &channel {
        None => config.clone(),
        Some(channel) => config.replace(channel.id)
    };

    let content = match (channel, current_log_channel_id) {
        (None, None) => "No log channel set.".to_owned(),
        (Some(c), None) => format!("Set {} as log channel.", c.id.mention()),
        (None, Some(cc_id)) => format!("Currently {} is set as log channel.", cc_id.mention()),
        (Some(c), Some(cc_id)) => format!(
            "Set {} as log channel. Previously {} was set.",
            c.id.mention(),
            cc_id.mention()
        )
    };

    let button = message_component::button::config::component(
        message_component::button::config::ConfigUnsetButton::LogChannel
    );
    let response_data = InteractionResponseDataBuilder::new()
        .content(content)
        .flags(MessageFlags::EPHEMERAL)
        .components([Component::ActionRow(ActionRow {
            components: vec![button]
        })])
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
