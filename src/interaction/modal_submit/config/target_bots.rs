use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{SelectMenuOption, SelectMenuType};
use twilight_model::http::interaction::InteractionResponseData;

use super::ConfigModalActionError;
use crate::STATE;
use crate::assets::{QUOTES, templates};
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};

pub fn modal(current: bool) -> InteractionResponseData {
    use templates::config::target_bots::modal;
    let mut allow = super::select("allow", SelectMenuType::Text, vec![], 1);
    allow.default_values = None;
    allow.required = Some(true);
    allow.min_values = Some(1);
    allow.options = Some(
        [
            (modal::allowed().to_string(), "true", true),
            (modal::protected().to_string(), "false", false)
        ]
        .into_iter()
        .map(|(label, value, state)| SelectMenuOption {
            label,
            value: value.into(),
            default: current == state,
            description: None,
            emoji: None
        })
        .collect()
    );
    super::modal(ModalCustomId::ConfigTargetBots, modal::title(), [super::label(
        modal::label(),
        modal::description(),
        Component::SelectMenu(allow)
    )])
}

pub async fn action(
    interaction: &Interaction,
    data: &ModalInteractionData
) -> Result<(), HandleModalSubmitError> {
    let guild_id = interaction.guild_id.ok_or(ConfigModalActionError::NotInGuild)?;
    let ModalInteractionComponent::StringSelect(select) = field(data, "allow")?
    else {
        return Err(ConfigModalActionError::InvalidField("allow").into());
    };
    let allow = match select.values.as_slice() {
        [value] if value == "true" => true,
        [value] if value == "false" => false,
        _ => return Err(ConfigModalActionError::InvalidField("allow").into())
    };
    STATE.database.update_config(guild_id, |config| config.target_bots = allow).await?;
    let content = if allow {
        templates::config::target_bots::enabled(QUOTES.config.target_bots.enabled).to_string()
    }
    else {
        templates::config::target_bots::disabled(QUOTES.config.target_bots.disabled).to_string()
    };
    reply(interaction, content).await
}
