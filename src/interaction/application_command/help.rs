use std::fmt::Write;
use std::ops::Deref;
use std::sync::LazyLock;

use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandOptionValue
};
use twilight_model::application::interaction::{Interaction, InteractionContextType};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use super::COMMANDS_REGISTER;
use crate::assets::{COLORS, replies};
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError, cloaker, config, counter, ranks
};
use crate::{APPLICATION_ID, BOT_USER_ID, STATE, assets};

pub const NAME: &str = "help";
pub const DESCRIPTION: &str = "Explains CloakerBot features and configuration by topic.";
pub static CHOICES: &[&str] = &[
    "takedowns",
    "config",
    "danger zones",
    "immunity",
    "log channel",
    "random takedowns",
    "countering",
    "ranks"
];
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.help);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

pub enum HelpCommand {
    All,
    Takedowns,
    Config,
    DangerZones,
    Immunity,
    LogChannel,
    RandomTakedowns,
    Countering,
    Ranks
}

impl TryFrom<&CommandData> for HelpCommand {
    type Error = TryFromInteractionError;

    fn try_from(value: &CommandData) -> Result<Self, Self::Error> {
        let Some(data) = value.options.first()
        else {
            return Ok(Self::All);
        };

        let CommandOptionValue::String(selected) = &data.value
        else {
            return Err(TryFromInteractionError::IncorrectFieldType {
                command: NAME.to_owned(),
                field: String::from("topic"),
                value: data.value.clone()
            });
        };

        match selected.as_str() {
            "takedowns" => Ok(Self::Takedowns),
            "config" => Ok(Self::Config),
            "danger zones" => Ok(Self::DangerZones),
            "immunity" => Ok(Self::Immunity),
            "log channel" => Ok(Self::LogChannel),
            "random takedowns" => Ok(Self::RandomTakedowns),
            "countering" => Ok(Self::Countering),
            "ranks" => Ok(Self::Ranks),
            _ => Err(TryFromInteractionError::IncorrectFieldValue {
                command: NAME.to_owned(),
                field: String::from("topic"),
                value: data.value.clone()
            })
        }
    }
}

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .contexts([InteractionContextType::Guild])
        .option(StringBuilder::new("topic", "The topic you want help with.").autocomplete(true))
        .build()
}

pub enum HelpActionError {}

pub async fn action(
    interaction: &Interaction,
    command: HelpCommand
) -> Result<(), HandleApplicationCommandError> {
    let mut content = String::new();
    let help_title = replies::help::title();

    // writing to String is infallible anyways
    #[allow(unused_must_use)]
    {
        match command {
            HelpCommand::All => {
                content.write_str(&replies::help::overview());
                content.write_char('\n');
                writeln!(content, "{}", cloaker::MENTION.deref());
                writeln!(content, "{}", cloaker::DESCRIPTION);
                content.write_char('\n');
                writeln!(content, "{}", config::MENTION.deref());
                writeln!(content, "{}", config::DESCRIPTION);
                content.write_char('\n');
                writeln!(content, "{}", counter::MENTION.deref());
                writeln!(content, "{}", counter::DESCRIPTION);
                content.write_char('\n');
                writeln!(content, "{}", MENTION.deref());
                writeln!(content, "{}", DESCRIPTION);
                content.write_char('\n');
                writeln!(content, "{}", ranks::MENTION.deref());
                writeln!(content, "{}", ranks::DESCRIPTION);
            }
            HelpCommand::Takedowns => {
                writeln!(content, "{}Takedowns", help_title);
                content.write_str(&assets::help_texts::takedowns(
                    BOT_USER_ID.mention(),
                    cloaker::MENTION.deref(),
                    counter::MENTION.deref()
                ));
            }
            HelpCommand::Config => {
                writeln!(content, "{}Config", help_title);
                content.write_str(&assets::help_texts::config(config::MENTION.deref()));
            }
            HelpCommand::DangerZones => {
                writeln!(content, "{}Danger zones", help_title);
                content.write_str(&assets::help_texts::danger_zones(
                    BOT_USER_ID.mention(),
                    counter::MENTION.deref()
                ));
            }
            HelpCommand::Immunity => {
                writeln!(content, "{}Immunity", help_title);
                content.write_str(&assets::help_texts::immunity(BOT_USER_ID.mention()));
            }
            HelpCommand::LogChannel => {
                writeln!(content, "{}Log channel", help_title);
                content.write_str(&assets::help_texts::log_channel());
            }
            HelpCommand::RandomTakedowns => {
                writeln!(content, "{}Random takedowns", help_title);
                content.write_str(&assets::help_texts::random_takedowns(BOT_USER_ID.mention()));
            }
            HelpCommand::Countering => {
                writeln!(content, "{}Countering", help_title);
                content.write_str(&assets::help_texts::countering(
                    BOT_USER_ID.mention(),
                    counter::MENTION.deref()
                ));
            }
            HelpCommand::Ranks => {
                writeln!(content, "{}Ranks", help_title);
                content.write_str(&assets::help_texts::ranks(
                    BOT_USER_ID.mention(),
                    ranks::MENTION.deref()
                ));
            }
        }

        content.write_char('\n');
        content.write_char('\n');
        content.write_str(&replies::help::support());
    }

    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: InteractionResponseDataBuilder::new()
            .content(content)
            .flags(MessageFlags::EPHEMERAL)
            .build()
            .into()
    };
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &response)
        .await?;

    Ok(())
}
