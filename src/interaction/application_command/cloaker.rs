use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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

use crate::assets::REPLIES;
use crate::interaction::application_command;
use crate::interaction::application_command::{HandleApplicationCommandError, TryFromInteractionError, update_defer_msg};
use crate::sync::OnceLockExpect;
use crate::takedown::{TakedownFail, TakedownResult};
use crate::util::template::Template;
use crate::{takedown, APPLICATION_ID, STATE};

pub const NAME: &str = "cloaker";
pub const DESCRIPTION: &str = "Take down your friends!";
pub static ID: OnceLockExpect<Id<CommandMarker>> = OnceLockExpect::new();
pub static MENTION: OnceLockExpect<MentionFormat<CommandMention>> = OnceLockExpect::new();

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
    let option = UserBuilder::new("target", "User to take down.")
        .required(true)
        .build();
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .default_member_permissions(permissions())
        .dm_permission(false)
        .option(option)
        .build()
}

pub fn user_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::User)
        .dm_permission(false)
        .build()
}

pub fn message_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::Message)
        .dm_permission(false)
        .build()
}

#[derive(Debug)]
pub enum CloakerActionError {
    NotInGuild,
    NoAuthor
}

impl Display for CloakerActionError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CloakerActionError {}

pub async fn action(
    interaction: &Interaction,
    command: CloakerCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(CloakerActionError::NotInGuild)?;
    let author_id = interaction.author_id().ok_or(CloakerActionError::NoAuthor)?;
    application_command::defer(interaction).await?;

    let content = match takedown::takedown(command.target_id, guild_id, author_id, false).await {
        // TODO: add quotes for all replies
        TakedownResult::Success => REPLIES
            .cloaker
            .success
            .template([&command.target_id.mention()]),
        TakedownResult::Countered { by_id } => {
            REPLIES.cloaker.countered.template([&by_id.mention()])
        }
        TakedownResult::Fail(fail) => REPLIES.cloaker.fail.template.template([&match fail {
            TakedownFail::Join(_) => Cow::from(REPLIES.cloaker.fail.join),
            TakedownFail::MissingPermissions => Cow::from(REPLIES.cloaker.fail.missing_perms),
            TakedownFail::NotFound => Cow::from(REPLIES.cloaker.fail.not_found),
            TakedownFail::Fled => Cow::from(REPLIES.cloaker.fail.fled),
            TakedownFail::Immune => Cow::from(REPLIES.cloaker.fail.immune),
            TakedownFail::Rejected => Cow::from(REPLIES.cloaker.fail.rejected),
            TakedownFail::HttpError(err) => Cow::from(REPLIES.cloaker.fail.other.template([&err]))
        }])
    };

    update_defer_msg(interaction, &content).await?;
    Ok(())
}
