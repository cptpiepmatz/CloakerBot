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
use crate::assets::{QUOTES, replies};
use crate::data::config::{CompleteDefault, GuildConfigRandom};
use crate::interaction::application_command::help;
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};

pub fn modal(current: &GuildConfigRandom) -> InteractionResponseData {
    let mut enabled = super::select("enabled", SelectMenuType::Text, vec![], 1);
    enabled.default_values = None;
    enabled.required = Some(true);
    enabled.min_values = Some(1);
    enabled.options = Some(
        [("Enabled", "true", true), ("Disabled", "false", false)]
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
    super::modal(ModalCustomId::ConfigRandom, "Configure random takedowns", [
        super::label(
            "Random takedowns",
            "Enable or disable automatic random takedowns.",
            Component::SelectMenu(enabled)
        ),
        interval("min", "Minimum interval (hours)", current.min, "Default: 1 hour"),
        interval("max", "Maximum interval (hours)", current.max, "Default: 24 hours")
    ])
}

#[allow(deprecated)]
fn interval(custom_id: &str, name: &str, value: Option<f64>, placeholder: &str) -> Component {
    super::label(
        name,
        "Enter hours, up to 8760. Leave blank or enter -1 to restore the default.",
        Component::TextInput(TextInput {
            id: None,
            custom_id: custom_id.into(),
            label: None,
            style: TextInputStyle::Short,
            required: Some(false),
            min_length: None,
            max_length: Some(100),
            placeholder: Some(placeholder.into()),
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
            return reply(
                interaction,
                "Nothing was saved. Intervals must be positive numbers of hours, at most 8760. \
                 Leave blank or enter -1 for defaults. Open Random takedowns in /config to try \
                 again."
            )
            .await;
        }
    };
    let random = GuildConfigRandom { enabled: Some(enabled), min, max };
    let complete = random.complete();
    if complete.min > complete.max {
        return reply(
            interaction,
            "Nothing was saved. The minimum interval cannot exceed the maximum interval \
             (including defaults). Open Random takedowns in /config to try again."
        )
        .await;
    }
    STATE.database.update_config(guild_id, |config| config.random = random.clone()).await?;
    let content = if enabled {
        replies::config::random::enabled(
            help::MENTION.deref(),
            complete.max,
            complete.min,
            QUOTES.config.random.enabled
        )
    }
    else {
        replies::config::random::disabled(QUOTES.config.random.disabled)
    };
    reply(interaction, content).await
}
