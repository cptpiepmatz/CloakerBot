use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::Arc;

use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, UserBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::replies;
use crate::interactions::commands::TryFromInteractionError;
use crate::permissions::get_in_channel_permissions;
use crate::{Com, TakedownPlan, APPLICATION_ID, BOT_USER_ID};

#[derive(Debug)]
pub struct CloakerCommand {
    target: Id<UserMarker>
}

impl TryFrom<&mut CommandData> for CloakerCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &mut CommandData) -> Result<Self, Self::Error> {
        let mut target: Option<Id<UserMarker>> = None;
        for option in data.options {
            let CommandDataOption { name, value } = option;
            match name.as_str() {
                "target" => match value {
                    CommandOptionValue::User(u) => target = Some(u),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command: NAME.into(),
                            field: name,
                            value: value.clone()
                        })
                    }
                },
                n => {
                    return Err(TryFromInteractionError::UnknownField {
                        command: NAME.into(),
                        field: name
                    })
                }
            }
        }
        
        match target {
            None => Err(TryFromInteractionError::MissingField {command: NAME.into(), field: "target".into()}),
            Some(target) => Ok(CloakerCommand { target })
        }
    }
}

pub const NAME: &str = "cloaker";
const DESCRIPTION: &str = "Take down your friends!";

fn permissions() -> Permissions {
    Permissions::ADMINISTRATOR | Permissions::MODERATE_MEMBERS | Permissions::MOVE_MEMBERS
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("user", "User to take down.")
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

pub async fn action(interaction: Interaction, com: Arc<Com>) {
    let destruct_com = com.clone();
    let Com {
        http,
        planned_takedowns,
        event_cache: cache,
        ..
    } = destruct_com.deref();

    // pull the target from the options
    let Some(InteractionData::ApplicationCommand(data)) = &interaction.data else {
        return;
    };
    let Some(CommandOptionValue::User(target_id)) = data.options.first().map(|opt| &opt.value) else {
        return;
    };
    let target_id = *target_id;
    let Some(guild_id) = data.guild_id else {
        return;
    };

    // check if target is in voice channel
    let Some(voice_channel_id) = cache.voice_state(target_id, guild_id).map(|vs| vs.channel_id()) else {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new()
                .flags(MessageFlags::EPHEMERAL)
                .content(replies::select(&replies::NO_CHANNEL))
                .build())
        };

        #[allow(clippy::single_match)]
        match http.interaction(APPLICATION_ID).create_response(interaction.id, interaction.token.as_str(), &response).await {
            Ok(_) => {}, // TODO: add logging
            Err(_) => {}
        }

        return;
    };

    // check if bot can join voice channel
    let Ok(channel_perms) = get_in_channel_permissions(BOT_USER_ID, guild_id, voice_channel_id, com).await else {
        // TODO: add some logging here
        return;
    };
    if !channel_perms.contains(Permissions::CONNECT) {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .flags(MessageFlags::EPHEMERAL)
                    .content(replies::select(&replies::CANT_JOIN))
                    .build()
            )
        };

        #[allow(clippy::single_match)]
        match http
            .interaction(APPLICATION_ID)
            .create_response(interaction.id, interaction.token.as_str(), &response)
            .await
        {
            Ok(_) => {} // TODO: add logging
            Err(_) => {}
        }

        return;
    }

    // check if target is bot

    let response = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: None
    };
    if (http
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &response)
        .await)
        .is_ok()
    {
        let plan = TakedownPlan {
            initiator_id: interaction.author_id(),
            target_id,
            countered: false
        };
        match planned_takedowns.get_mut(&guild_id) {
            None => {
                planned_takedowns.insert(guild_id, VecDeque::from([plan]));
            }
            Some(mut q) => {
                q.push_back(plan);
            }
        };
    }
}
