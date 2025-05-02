use std::fmt::{Display, Write};
use std::ops::Deref;

use const_format::{concatcp, formatcp, str_replace};
use itertools::Itertools;
use twilight_http::request::RequestBuilder;
use twilight_http::routing::Route;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_mention::Mention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::CommandMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, StringBuilder};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::{replies, COLORS};
use crate::interaction::application_command::{
    cloaker, config, counter, defer, ranks, update_defer_embeds, HandleApplicationCommandError,
    TryFromInteractionError
};
use crate::sync::OnceLockExpect;
use crate::{assets, APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "help";
pub const DESCRIPTION: &str = "Shows a help page.";
pub static CHOICES: &[&str] = &[
    cloaker::NAME,
    formatcp!("{} {}", config::NAME, config::danger_zones::NAME),
    formatcp!("{} {}", config::NAME, config::immunity::NAME),
    formatcp!("{} {}", config::NAME, config::log_channel::NAME),
    formatcp!("{} {}", config::NAME, config::random::NAME),
    counter::NAME,
    ranks::NAME
];
pub static ID: OnceLockExpect<Id<CommandMarker>> = OnceLockExpect::new();
pub static MENTION: OnceLockExpect<MentionFormat<CommandMention>> = OnceLockExpect::new();

pub enum HelpCommand {
    All,
    Cloaker,
    Config(ConfigHelpCommand),
    Counter,
    Ranks
}

pub enum ConfigHelpCommand {
    DangerZones,
    Immunity,
    LogChannel,
    Random
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
                field: String::from("command"),
                value: data.value.clone()
            });
        };

        match selected.as_str() {
            cloaker::NAME => Ok(HelpCommand::Cloaker),
            counter::NAME => Ok(HelpCommand::Counter),
            ranks::NAME => Ok(HelpCommand::Ranks),
            name => {
                let mut split = name.split(' ');
                match (split.next(), split.next()) {
                    (Some(config::NAME), Some(config::danger_zones::NAME)) => {
                        Ok(HelpCommand::Config(ConfigHelpCommand::DangerZones))
                    }
                    (Some(config::NAME), Some(config::immunity::NAME)) => {
                        Ok(HelpCommand::Config(ConfigHelpCommand::Immunity))
                    }
                    (Some(config::NAME), Some(config::log_channel::NAME)) => {
                        Ok(HelpCommand::Config(ConfigHelpCommand::LogChannel))
                    }
                    (Some(config::NAME), Some(config::random::NAME)) => {
                        Ok(HelpCommand::Config(ConfigHelpCommand::Random))
                    }
                    _ => Err(TryFromInteractionError::IncorrectFieldValue {
                        command: NAME.to_owned(),
                        field: String::from("command"),
                        value: data.value.clone()
                    })
                }
            }
        }
    }
}

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .option(
            StringBuilder::new("command", "Shows help for a command.")
                .choices(CHOICES.iter().map(|c| (c.to_owned(), c.to_owned())))
        )
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
                writeln!(content, "`/{}`", config::NAME);
                writeln!(content, "{}", config::DESCRIPTION);
                writeln!(content, "    {}", config::danger_zones::MENTION.deref());
                writeln!(content, "    {}", config::danger_zones::DESCRIPTION);
                writeln!(content, "    {}", config::immunity::MENTION.deref());
                writeln!(content, "    {}", config::immunity::DESCRIPTION);
                writeln!(content, "    {}", config::log_channel::MENTION.deref());
                writeln!(content, "    {}", config::log_channel::DESCRIPTION);
                writeln!(content, "    {}", config::random::MENTION.deref());
                writeln!(content, "    {}", config::random::DESCRIPTION);
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
            HelpCommand::Cloaker => {
                writeln!(content, "{}{}", help_title, cloaker::MENTION.deref());
                content.write_str(&assets::help_texts::cloaker(
                    BOT_USER_ID.mention(),
                    counter::MENTION.deref()
                ));
            }
            HelpCommand::Config(ConfigHelpCommand::DangerZones) => {
                writeln!(content, "{}{}", help_title, config::danger_zones::MENTION.deref());
                content.write_str(&assets::help_texts::config_danger_zones(
                    BOT_USER_ID.mention(),
                    counter::MENTION.deref()
                ));
            }
            HelpCommand::Config(ConfigHelpCommand::Immunity) => {
                writeln!(content, "{}{}", help_title, config::immunity::MENTION.deref());
                content.write_str(&assets::help_texts::config_immunity(BOT_USER_ID.mention()));
            }
            HelpCommand::Config(ConfigHelpCommand::LogChannel) => {
                writeln!(content, "{}{}", help_title, config::log_channel::MENTION.deref());
                content.write_str(&assets::help_texts::config_log_channel());
            }
            HelpCommand::Config(ConfigHelpCommand::Random) => {
                writeln!(content, "{}{}", help_title, config::random::MENTION.deref());
                content.write_str(&assets::help_texts::config_random(BOT_USER_ID.mention()));
            }
            HelpCommand::Counter => {
                writeln!(content, "{}{}", help_title, counter::MENTION.deref());
                content.write_str(&assets::help_texts::counter(
                    BOT_USER_ID.mention(),
                    counter::MENTION.deref()
                ));
            }
            HelpCommand::Ranks => {
                writeln!(content, "{}{}", help_title, ranks::MENTION.deref());
                // TODO: add ranks help content here
                write!(content, "🚧 Ranks are currently work in progress. 🚧");
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
