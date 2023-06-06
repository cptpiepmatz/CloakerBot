use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{ChannelBuilder, SubCommandBuilder};

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::TryFromInteractionError;

pub const NAME: &str = "log-channel";
pub const DESCRIPTION: &str = "Define your log channel.";

pub struct LogChannelSubCommand {
    channel_id: Option<Id<ChannelMarker>>
}

impl TryFrom<&Vec<CommandDataOption>> for LogChannelSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut channel: Option<Id<ChannelMarker>> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "channel" => match &option.value {
                    CommandOptionValue::Channel(c) => channel = Some(*c),
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
        Ok(LogChannelSubCommand {
            channel_id: channel
        })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(ChannelBuilder::new("channel", "Channel to output logging data.").build())
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    LogChannelSubCommand { channel_id }: LogChannelSubCommand
) -> Result<(), ConfigActionError> {
    todo!()
}
