use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::{Component, Embed, MessageFlags, ReactionType};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, SubCommandBuilder};
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;
use crate::util::template::Template;
use crate::assets::{COLORS, REPLIES, QUOTES};
use crate::data::config::GuildConfig;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "target-bots";
pub const DESCRIPTION: &str = "Configure if the bot should target bots.";

pub const CLOAKER_QUOTE_TARGET_BOTS: &str =
    "So I've got to play nice with the tin cans, huh? Just don’t expect me to oil them up.";
pub const CLOAKER_QUOTE_NO_TARGET_BOTS: &str =
    "Tin cans are back on the menu! Let's see what these robots are really made of.";

pub struct TargetBotsSubCommand {
    allow: Option<bool>
}

impl TryFrom<&Vec<CommandDataOption>> for TargetBotsSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut allow: Option<bool> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "allow" => match &option.value {
                    CommandOptionValue::Boolean(b) => allow = Some(*b),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "allow".into(),
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
        Ok(TargetBotsSubCommand { allow })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(BooleanBuilder::new("allow", "Allow/disallow targeting bots.").build())
        .build()
}

pub async fn action(
    interaction: &Interaction,
    TargetBotsSubCommand { allow }: TargetBotsSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let config = &mut guild_configs.entry(guild_id).or_default().target_bots;

    if let Some(update) = allow {
        config.replace(update);
    }

    let content = match config {
        None | Some(true) => REPLIES.config.target_bots.enabled.template([&QUOTES.config.target_bots.enabled]),
        Some(false) => REPLIES.config.target_bots.disabled.template([&QUOTES.config.target_bots.disabled])
    };

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .build();

    STATE
        .http
        .interaction(APPLICATION_ID)
        .create_response(
            interaction.id,
            interaction.token.as_str(),
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: response_data.into()
            }
        )
        .await?;

    Ok(())
}
