use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::LazyLock;

use thiserror::Error;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_mention::Mention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{CommandMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, UserBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use super::COMMANDS_REGISTER;
use crate::assets::replies::cloaker;
use crate::interaction::application_command;
use crate::interaction::application_command::{
    update_defer_msg, HandleApplicationCommandError, TryFromInteractionError
};
use crate::takedown::{TakedownFail, TakedownSuccess, TakedownType};
use crate::{takedown, APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "cloaker";
pub const DESCRIPTION: &str = "Take down your friends!";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.cloaker);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

#[derive(Debug)]
pub struct CloakerCommand {
    target_id: Id<UserMarker>
}

impl TryFrom<&CommandData> for CloakerCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let mut target: Option<Id<UserMarker>> = None;
        for option in &data.options {
            let CommandDataOption { name, value } = option;
            match name.as_str() {
                "target" => match value {
                    CommandOptionValue::User(u) => target = Some(*u),
                    _v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command: NAME.into(),
                            field: name.clone(),
                            value: value.clone()
                        })
                    }
                },
                _n => {
                    return Err(TryFromInteractionError::UnknownField {
                        command: NAME.into(),
                        field: name.clone()
                    })
                }
            }
        }

        if let Some(resolved) = &data.resolved {
            if let Some(user_id) = resolved.users.keys().next() {
                target = Some(*user_id);
            }

            if let Some(message) = resolved.messages.values().next() {
                target = Some(message.author.id);
            }
        }

        match target {
            None => Err(TryFromInteractionError::MissingField {
                command: NAME.into(),
                field: "target".into()
            }),
            Some(target) => Ok(CloakerCommand { target_id: target })
        }
    }
}

fn permissions() -> Permissions {
    Permissions::ADMINISTRATOR | Permissions::MODERATE_MEMBERS | Permissions::MOVE_MEMBERS
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("target", "User to take down.").required(true).build();
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .default_member_permissions(permissions())
        .dm_permission(false)
        .option(option)
        .build()
}

pub fn user_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::User).dm_permission(false).build()
}

pub fn message_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::Message).dm_permission(false).build()
}

#[derive(Debug, Error)]
pub enum CloakerActionError {
    #[error("not in guild")]
    NotInGuild,
    #[error("author not found")]
    NoAuthor
}

pub async fn action(
    interaction: &Interaction,
    command: CloakerCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(CloakerActionError::NotInGuild)?;
    let author_id = interaction.author_id().ok_or(CloakerActionError::NoAuthor)?;
    application_command::defer(interaction).await?;

    let content = match takedown::takedown(command.target_id, guild_id, TakedownType::Called {
        caller_id: author_id
    })
    .await
    {
        Ok(TakedownSuccess { .. }) => cloaker::success("TODO: quote", command.target_id.mention()),
        Err(TakedownFail::Countered { by_id }) => {
            cloaker::countered(by_id.mention(), "TODO: quote")
        }
        Err(fail) => cloaker::fail::template("TODO: quote", match fail {
            TakedownFail::Countered { .. } => unreachable!(),
            TakedownFail::Join(_) => cloaker::fail::join(),
            TakedownFail::MissingPermissions => cloaker::fail::missing_perms(),
            TakedownFail::NotFound => cloaker::fail::not_found(),
            TakedownFail::Fled => cloaker::fail::fled(),
            TakedownFail::Immune => cloaker::fail::immune(),
            TakedownFail::Rejected => cloaker::fail::rejected(BOT_USER_ID.mention()),
            TakedownFail::Client(err) => cloaker::fail::other(err)
        })
    };

    update_defer_msg(interaction, &content).await?;
    Ok(())
}
