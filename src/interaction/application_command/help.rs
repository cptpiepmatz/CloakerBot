use std::ops::Deref;

use const_format::{concatcp, formatcp, str_replace};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::CommandMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, StringBuilder};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::assets::COLORS;
use crate::interaction::application_command::{
    cloaker, config, counter, defer, ranks, update_defer_embeds, HandleApplicationCommandError,
    TryFromInteractionError
};
use crate::sync::OnceLockExpect;
use crate::util::trim::trim;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "help";
pub const DESCRIPTION: &str = "Shows a help page.";
pub const CHOICES: [&str; 7] = [
    cloaker::NAME,
    formatcp!("{} {}", config::NAME, config::danger_zones::NAME),
    formatcp!("{} {}", config::NAME, config::log_channel::NAME),
    formatcp!("{} {}", config::NAME, config::target_bots::NAME),
    formatcp!("{} {}", config::NAME, config::random::NAME),
    counter::NAME,
    ranks::NAME
];
pub static ID: OnceLockExpect<Id<CommandMarker>> = OnceLockExpect::new();

pub enum HelpCommand {
    All,
    Cloaker,
    Config(ConfigHelpCommand),
    Counter,
    Ranks
}

pub enum ConfigHelpCommand {
    DangerZones,
    LogChannel,
    TargetBots,
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
                    (Some(config::NAME), Some(config::log_channel::NAME)) => {
                        Ok(HelpCommand::Config(ConfigHelpCommand::LogChannel))
                    }
                    (Some(config::NAME), Some(config::target_bots::NAME)) => {
                        Ok(HelpCommand::Config(ConfigHelpCommand::TargetBots))
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

macro_rules! prepare_md {
    ($in:expr) => {
        str_replace!(
            str_replace!(
                str_replace!(
                    str_replace!(str_replace!(trim!($in), "\r\n", "\n"), "\n\n\n", "<br><br>"),
                    "\n\n",
                    "<br>"
                ),
                "\n",
                " "
            ),
            "<br>",
            "\n"
        )
    };
}

const CLOAKER_HELP_TEXT: &str = prepare_md!(include_str!("../../../assets/help/cloaker.md"));
const CONFIG_DANGER_ZONES_HELP_TEXT: &str =
    prepare_md!(include_str!("../../../assets/help/config_danger_zones.md"));
const CONFIG_LOG_CHANNEL_HELP_TEXT: &str =
    prepare_md!(include_str!("../../../assets/help/config_log_channel.md"));
const CONFIG_TARGET_BOTS_HELP_TEXT: &str =
    prepare_md!(include_str!("../../../assets/help/config_target_bots.md"));
const CONFIG_RANDOM_HELP_TEXT: &str =
    prepare_md!(include_str!("../../../assets/help/config_random.md"));
const COUNTER_HELP_TEXT: &str = prepare_md!(include_str!("../../../assets/help/counter.md"));

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
    defer(&interaction).await?;

    let embed = EmbedBuilder::new().color(COLORS.primary as u32);

    let embed = match command {
        HelpCommand::All => embed
            .title("🔎 Help Overview")
            .description("For more info and help on a command use `/help <command name>`.")
            .field(EmbedFieldBuilder::new(
                format!("/{}", cloaker::NAME),
                "Take down your friends."
            ))
            .field(EmbedFieldBuilder::new(
                format!("/{}", config::NAME),
                concatcp!(
                    "Configure the bot for this server.\n",
                    formatcp!("**{}**\n", config::danger_zones::NAME),
                    "Define what channels should be marked as danger zones.\n",
                    formatcp!("**{}**\n", config::log_channel::NAME),
                    "Define what channel to use for logging.\n",
                    formatcp!("**{}**\n", config::target_bots::NAME),
                    "Define whether the CloakerBot should take down bots.\n",
                    formatcp!("**{}**\n", config::random::NAME),
                    "Configure the random takedowns."
                )
            ))
            .field(EmbedFieldBuilder::new(
                format!("/{}", counter::NAME),
                "Counter a takedown."
            ))
            .field(EmbedFieldBuilder::new(
                format!("/{}", ranks::NAME),
                "Create a badge that shows your takedown stats."
            )),

        HelpCommand::Cloaker => embed
            .title(format!("🔎 Help for /{}", cloaker::NAME))
            .description(CLOAKER_HELP_TEXT),

        HelpCommand::Config(ConfigHelpCommand::DangerZones) => embed
            .title(format!(
                "🔎 Help for /{} {}",
                config::NAME,
                config::danger_zones::NAME
            ))
            .description(CONFIG_DANGER_ZONES_HELP_TEXT),

        HelpCommand::Config(ConfigHelpCommand::LogChannel) => embed
            .title(format!(
                "🔎 Help for /{} {}",
                config::NAME,
                config::log_channel::NAME
            ))
            .description(CONFIG_LOG_CHANNEL_HELP_TEXT),

        HelpCommand::Config(ConfigHelpCommand::TargetBots) => embed
            .title(format!(
                "🔎 Help for /{} {}",
                config::NAME,
                config::target_bots::NAME
            ))
            .description(CONFIG_TARGET_BOTS_HELP_TEXT),

        HelpCommand::Config(ConfigHelpCommand::Random) => embed
            .title(format!(
                "🔎 Help for /{} {}",
                config::NAME,
                config::random::NAME
            ))
            .description(CONFIG_RANDOM_HELP_TEXT),

        HelpCommand::Counter => embed
            .title(format!("🔎 Help for /{}", counter::NAME))
            .description(COUNTER_HELP_TEXT),

        HelpCommand::Ranks => todo!()
    };

    #[allow(clippy::invisible_characters)]
    let embed = embed
        .field(EmbedFieldBuilder::new("​", ""))
        .field(EmbedFieldBuilder::new(
            "CloakerBot Discord Server",
            concat!(
                "If you have any issues come over to the ",
                "[discord server](https://cptpiepmatz.de/cloaker-bot/join-server)",
                " and ask there."
            )
        ));

    update_defer_embeds(interaction, embed).await?;

    Ok(())
}
