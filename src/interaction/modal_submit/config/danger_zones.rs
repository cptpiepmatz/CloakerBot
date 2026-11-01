use itertools::Itertools;
use twilight_mention::Mention;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::channel::ChannelType;
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{SelectDefaultValue, SelectMenuType};
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;

use super::ConfigModalActionError;
use crate::STATE;
use crate::assets::{QUOTES, replies};
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};

pub fn modal(current: &[Id<ChannelMarker>]) -> InteractionResponseData {
    let mut select = super::select(
        "channels",
        SelectMenuType::Channel,
        current.iter().copied().map(SelectDefaultValue::Channel).collect(),
        25
    );
    select.channel_types = Some(vec![ChannelType::GuildVoice]);
    super::modal(ModalCustomId::ConfigDangerZones, "Configure danger zones", [super::label(
        "Danger zones",
        "Choose voice channels that trigger automatic takedowns. Clear the selection to unset.",
        Component::SelectMenu(select)
    )])
}

pub async fn action(
    interaction: &Interaction,
    data: &ModalInteractionData
) -> Result<(), HandleModalSubmitError> {
    let guild_id = interaction.guild_id.ok_or(ConfigModalActionError::NotInGuild)?;
    let ModalInteractionComponent::ChannelSelect(select) = field(data, "channels")?
    else {
        return Err(ConfigModalActionError::InvalidField("channels").into());
    };
    if select.values.len() > 25 {
        return Err(ConfigModalActionError::InvalidField("channels").into());
    }
    STATE
        .database
        .update_config(guild_id, |config| config.danger_zones = select.values.clone())
        .await?;
    let content = if select.values.is_empty() {
        replies::config::danger_zones::unset(QUOTES.config.danger_zones.no_channels)
    }
    else {
        replies::config::danger_zones::set(
            select.values.iter().map(|id| format!("- {}", id.mention())).join("\n"),
            QUOTES.config.danger_zones.channels
        )
    };
    reply(interaction, content).await
}
