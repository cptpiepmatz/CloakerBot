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
use crate::assets::{QUOTES, replies};
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};
use crate::{BOT_USER_ID, STATE};

pub fn modal(current: Option<Id<ChannelMarker>>) -> InteractionResponseData {
    let mut select = super::select(
        "channel",
        SelectMenuType::Channel,
        current.into_iter().map(SelectDefaultValue::Channel).collect(),
        1
    );
    select.channel_types = Some(vec![ChannelType::GuildText]);
    super::modal(ModalCustomId::ConfigLogChannel, "Configure log channel", [super::label(
        "Log channel",
        "Choose a text channel. Clear the selection to use the automatic fallback.",
        Component::SelectMenu(select)
    )])
}

pub async fn action(
    interaction: &Interaction,
    data: &ModalInteractionData
) -> Result<(), HandleModalSubmitError> {
    let guild_id = interaction.guild_id.ok_or(ConfigModalActionError::NotInGuild)?;
    let ModalInteractionComponent::ChannelSelect(select) = field(data, "channel")?
    else {
        return Err(ConfigModalActionError::InvalidField("channel").into());
    };
    if select.values.len() > 1 {
        return Err(ConfigModalActionError::InvalidField("channel").into());
    }
    let channel = select.values.first().copied();
    STATE.database.update_config(guild_id, |config| config.log_channel = channel).await?;
    let content = match channel {
        Some(id) => replies::config::log_channel::set(
            BOT_USER_ID.mention(),
            id.mention(),
            QUOTES.config.log_channel.channel
        )
        .to_string(),
        None => replies::config::log_channel::unset(
            BOT_USER_ID.mention(),
            crate::channel_logging::fallback_description(guild_id).await,
            QUOTES.config.log_channel.no_channel
        )
        .to_string()
    };
    reply(interaction, content).await
}
