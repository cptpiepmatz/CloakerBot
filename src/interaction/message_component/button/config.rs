use twilight_http::request::Request;
use twilight_http::response::marker::EmptyBody;
use twilight_http::routing::Route;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::message_component::HandleMessageComponentError;
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::{APPLICATION_ID, STATE};

#[derive(Debug)]
pub enum ConfigButtonActionError {
    NotInGuild,
    Serialize(serde_json::Error),
    UnexpectedCustomId(MessageComponentCustomId)
}

pub async fn open_modal(
    interaction: &Interaction,
    custom_id: MessageComponentCustomId
) -> Result<(), HandleMessageComponentError> {
    let guild_id = interaction.guild_id.ok_or(ConfigButtonActionError::NotInGuild)?;
    let can_manage = interaction
        .member
        .as_ref()
        .and_then(|member| member.permissions)
        .is_some_and(|permissions| permissions.contains(Permissions::MANAGE_GUILD));
    if !can_manage {
        STATE
            .client
            .interaction(APPLICATION_ID)
            .create_response(interaction.id, &interaction.token, &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .flags(MessageFlags::EPHEMERAL)
                        .content("You need Manage Server permission to change the configuration.")
                        .build()
                )
            })
            .await?;
        return Ok(());
    }
    let config = STATE.database.guild_config(guild_id).await?;
    use MessageComponentCustomId as Id;

    use crate::interaction::modal_submit::config as modals;
    let data = match custom_id {
        Id::ConfigOpenDangerZones => modals::danger_zones::modal(&config.danger_zones),
        Id::ConfigOpenLogChannel => modals::log_channel::modal(config.log_channel),
        Id::ConfigOpenImmunity => modals::immunity::modal(&config.immunity),
        Id::ConfigOpenRandom => modals::random::modal(&config.random),
        Id::ConfigOpenTargetBots => modals::target_bots::modal(config.target_bots),
        id => return Err(ConfigButtonActionError::UnexpectedCustomId(id).into())
    };
    let mut response = serde_json::to_value(InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(data)
    })
    .map_err(ConfigButtonActionError::Serialize)?;
    // Twilight 0.17 emits disabled on selects, but Discord forbids it in modals
    if let Some(components) = response["data"]["components"].as_array_mut() {
        for label in components {
            if let Some(component) = label["component"].as_object_mut() {
                component.remove("disabled");
            }
        }
    }
    let request = Request::builder(&Route::InteractionCallback {
        interaction_id: interaction.id.get(),
        interaction_token: &interaction.token,
        with_response: false
    })
    .use_authorization_token(false)
    .json(&response)
    .build()?;
    STATE.client.request::<EmptyBody>(request).await?;
    Ok(())
}
