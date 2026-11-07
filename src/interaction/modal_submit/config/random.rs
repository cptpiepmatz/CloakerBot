use std::fmt::Display;
use std::ops::Deref;

use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{
    SelectMenuOption, SelectMenuType, TextInput, TextInputStyle
};
use twilight_model::http::interaction::InteractionResponseData;

use super::ConfigModalActionError;
use crate::STATE;
use crate::assets::{QUOTES, templates};
use crate::data::config::{CompleteDefault, GuildConfigRandom};
use crate::interaction::application_command::help;
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};

pub fn modal(current: &GuildConfigRandom) -> InteractionResponseData {
    use templates::config::random::modal;
    let mut enabled = super::select("enabled", SelectMenuType::Text, vec![], 1);
    enabled.default_values = None;
    enabled.required = Some(true);
    enabled.min_values = Some(1);
    enabled.options = Some(
        [
            (modal::enabled().to_string(), "true", true),
            (modal::disabled().to_string(), "false", false)
        ]
        .into_iter()
        .map(|(label, value, state)| SelectMenuOption {
            label: label.into(),
            value: value.into(),
            default: current.complete().enabled == state,
            description: None,
            emoji: None
        })
        .collect()
    );
    super::modal(ModalCustomId::ConfigRandom, modal::title(), [
        super::label(modal::label(), modal::description(), Component::SelectMenu(enabled)),
        interval("min", modal::min::label(), current.min, modal::min::placeholder()),
        interval("max", modal::max::label(), current.max, modal::max::placeholder())
    ])
}

#[allow(deprecated)]
fn interval(
    custom_id: &str,
    name: impl Display,
    value: Option<f64>,
    placeholder: impl Display
) -> Component {
    super::label(
        name,
        templates::config::random::modal::interval_description(),
        Component::TextInput(TextInput {
            id: None,
            custom_id: custom_id.into(),
            label: None,
            style: TextInputStyle::Short,
            required: Some(false),
            min_length: None,
            max_length: Some(100),
            placeholder: Some(placeholder.to_string()),
            value: value.map(|value| value.to_string())
        })
    )
}

fn parse_interval(
    data: &ModalInteractionData,
    name: &'static str
) -> Result<Option<f64>, ConfigModalActionError> {
    let ModalInteractionComponent::TextInput(input) = field(data, name)?
    else {
        return Err(ConfigModalActionError::InvalidField(name));
    };
    let value = input.value.trim();
    if value.is_empty() || value == "-1" {
        return Ok(None);
    }
    let value: f64 = value.parse().map_err(|_| ConfigModalActionError::InvalidField(name))?;
    if !value.is_finite() || value <= 0.0 || value > 8760.0 {
        return Err(ConfigModalActionError::InvalidField(name));
    }
    Ok(Some(value))
}

pub async fn action(
    interaction: &Interaction,
    data: &ModalInteractionData
) -> Result<(), HandleModalSubmitError> {
    let guild_id = interaction.guild_id.ok_or(ConfigModalActionError::NotInGuild)?;
    let ModalInteractionComponent::StringSelect(select) = field(data, "enabled")?
    else {
        return Err(ConfigModalActionError::InvalidField("enabled").into());
    };
    let enabled = match select.values.as_slice() {
        [value] if value == "true" => true,
        [value] if value == "false" => false,
        _ => return Err(ConfigModalActionError::InvalidField("enabled").into())
    };
    let (min, max) = match (parse_interval(data, "min"), parse_interval(data, "max")) {
        (Ok(min), Ok(max)) => (min, max),
        _ => {
            return reply(interaction, templates::config::random::invalid_interval().to_string())
                .await;
        }
    };
    let random = GuildConfigRandom { enabled: Some(enabled), min, max };
    let complete = random.complete();
    if complete.min > complete.max {
        return reply(interaction, templates::config::random::invalid_interval_order().to_string())
            .await;
    }
    STATE.database.update_config(guild_id, |config| config.random = random.clone()).await?;
    let content = if enabled {
        templates::config::random::enabled(
            help::MENTION.deref(),
            complete.max,
            complete.min,
            QUOTES.config.random.enabled
        )
        .to_string()
    }
    else {
        templates::config::random::disabled(QUOTES.config.random.disabled).to_string()
    };
    reply(interaction, content).await
}
