use custom_id::ModalCustomId;
use derive_more::From;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{APPLICATION_ID, STATE};

pub mod config;
mod custom_id;

#[derive(Debug, From)]
pub enum HandleModalSubmitError {
    Database(sqlx::Error),
    NoModalSubmit,
    UnknownCustomId(String),
    ConfigAction(config::ConfigModalActionError),
    Http(twilight_http::Error)
}

pub async fn handle_modal_submit(interaction: &Interaction) -> Result<(), HandleModalSubmitError> {
    let Some(InteractionData::ModalSubmit(data)) = interaction.data.as_ref()
    else {
        return Err(HandleModalSubmitError::NoModalSubmit);
    };

    let custom_id: ModalCustomId = data.custom_id.parse()?;
    let can_manage = interaction
        .member
        .as_ref()
        .and_then(|member| member.permissions)
        .is_some_and(|permissions| permissions.contains(Permissions::MANAGE_GUILD));
    if !can_manage {
        return reply(
            interaction,
            "You need Manage Server permission to change the configuration."
        )
        .await;
    }

    match custom_id {
        ModalCustomId::ConfigDangerZones => config::danger_zones::action(interaction, data).await,
        ModalCustomId::ConfigLogChannel => config::log_channel::action(interaction, data).await,
        ModalCustomId::ConfigImmunity => config::immunity::action(interaction, data).await,
        ModalCustomId::ConfigRandom => config::random::action(interaction, data).await,
        ModalCustomId::ConfigTargetBots => config::target_bots::action(interaction, data).await
    }
}

fn field<'a>(
    data: &'a ModalInteractionData,
    custom_id: &'static str
) -> Result<&'a ModalInteractionComponent, config::ConfigModalActionError> {
    data.components
        .iter()
        .filter_map(|component| match component {
            ModalInteractionComponent::Label(label) => Some(label.component.as_ref()),
            _ => None
        })
        .find(|component| match component {
            ModalInteractionComponent::ChannelSelect(select) => select.custom_id == custom_id,
            ModalInteractionComponent::RoleSelect(select) => select.custom_id == custom_id,
            ModalInteractionComponent::StringSelect(select) => select.custom_id == custom_id,
            ModalInteractionComponent::TextInput(input) => input.custom_id == custom_id,
            _ => false
        })
        .ok_or(config::ConfigModalActionError::InvalidField(custom_id))
}

async fn reply(
    interaction: &Interaction,
    content: impl Into<String>
) -> Result<(), HandleModalSubmitError> {
    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .build();
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data)
        })
        .await?;
    Ok(())
}
