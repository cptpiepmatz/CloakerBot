use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::RwLock;
use twilight_http::client::InteractionClient;
use twilight_http::Client as HttpClient;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, UserBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{Com, SharedData, TakedownPlan, APPLICATION_ID};

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
    let com = com.deref();
    let Com {
        http, shared_data, ..
    } = com;

    // pull the target from the options
    let Some(InteractionData::ApplicationCommand(data)) = &interaction.data else {
        return;
    };
    let Some(CommandOptionValue::User(target_id)) = data.options.first().map(|opt| &opt.value) else {
        return;
    };
    let Some(guild_id) = data.guild_id else {
        return;
    };

    let response = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: Some(InteractionResponseDataBuilder::new().content("lol").build())
    };
    if let Ok(_) = http
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &response)
        .await
    {
        let planned_takedowns = &mut shared_data.write().await.planned_takedowns;
        let plan = TakedownPlan {
            initiator_id: interaction.author_id(),
            target_id: *target_id,
            countered: false
        };
        match planned_takedowns.get_mut(&guild_id) {
            None => {
                planned_takedowns.insert(guild_id, VecDeque::from([plan]));
            }
            Some(q) => {
                q.push_back(plan);
            }
        };
    }
}
