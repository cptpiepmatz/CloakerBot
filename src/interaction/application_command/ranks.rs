use std::sync::LazyLock;

use derive_more::From;
use thiserror::Error;
use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::http::attachment::Attachment;
use twilight_model::id::Id;
use twilight_model::id::marker::{CommandMarker, UserMarker};
use twilight_util::builder::command::{CommandBuilder, UserBuilder};

use super::COMMANDS_REGISTER;
use crate::data::stats;
use crate::interaction::application_command;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::ranks::CreateRanksGraphicError;
use crate::{APPLICATION_ID, STATE, takedown};

pub const NAME: &str = "ranks";
pub const DESCRIPTION: &str = "Show off your takedown stats in a rank profile.";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.ranks);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

#[derive(Debug)]
pub struct RanksCommand {
    user_id: Option<Id<UserMarker>>
}

impl TryFrom<&CommandData> for RanksCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let mut user: Option<Id<UserMarker>> = None;
        for option in &data.options {
            let CommandDataOption { name, value } = option;
            match (name.as_str(), value) {
                ("user", CommandOptionValue::User(u)) => user = Some(*u),
                ("user", _) => {
                    return Err(TryFromInteractionError::IncorrectFieldType {
                        command: NAME.into(),
                        field: name.clone(),
                        value: value.clone()
                    });
                }
                (_, _) => {
                    return Err(TryFromInteractionError::UnknownField {
                        command: NAME.into(),
                        field: name.clone()
                    });
                }
            }
        }

        Ok(RanksCommand { user_id: user })
    }
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("user", "User to show ranks for.").required(false).build();

    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .option(option)
        .build()
}

// TODO: implement Error
#[derive(Debug, From)]
pub enum RanksActionError {
    NotInGuild,
    NoAuthor,
    CreateRanksGraphic(CreateRanksGraphicError)
}

pub async fn action(
    interaction: &Interaction,
    command: RanksCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(RanksActionError::NotInGuild)?;
    let author_id = interaction.author_id().ok_or(RanksActionError::NoAuthor)?;
    application_command::defer(interaction).await?;

    let user_id = command.user_id.unwrap_or(author_id);
    let member = STATE.client.guild_member(guild_id, user_id).await?.model().await?;
    let name = member.nick.or(member.user.global_name).unwrap_or(member.user.name);

    let png = crate::ranks::create_ranks_graphic(user_id, guild_id)
        .await
        .map_err(RanksActionError::CreateRanksGraphic)?;

    STATE
        .client
        .interaction(APPLICATION_ID)
        .update_response(interaction.token.as_str())
        .attachments(&[Attachment {
            description: Some(format!("CloakerBot Ranks Profile of {name}.")),
            file: png.into(),
            filename: format!("user_rank_{guild_id}_{user_id}.png"),
            id: 0
        }])
        .await?;

    Ok(())
}
