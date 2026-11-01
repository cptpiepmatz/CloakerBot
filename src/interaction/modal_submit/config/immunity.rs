use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{SelectDefaultValue, SelectMenuType};
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::id::Id;
use twilight_model::id::marker::RoleMarker;

use super::ConfigModalActionError;
use crate::STATE;
use crate::data::config::GuildConfigImmunity;
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};

pub fn modal(current: &GuildConfigImmunity) -> InteractionResponseData {
    let fields = [
        ("general", "General immunity", "Protected from all takedowns.", &current.general),
        ("random", "Random immunity", "Protected from random takedowns only.", &current.random),
        (
            "general_silent",
            "Silent general immunity",
            "Never targeted by any takedown.",
            &current.general_silent
        ),
        (
            "random_silent",
            "Silent random immunity",
            "Never selected for random takedowns.",
            &current.random_silent
        )
    ];
    super::modal(
        ModalCustomId::ConfigImmunity,
        "Immunity",
        fields.map(|(id, title, description, roles)| {
            super::label(
                title,
                description,
                Component::SelectMenu(super::select(
                    id,
                    SelectMenuType::Role,
                    roles.iter().copied().map(SelectDefaultValue::Role).collect(),
                    25
                ))
            )
        })
    )
}

fn roles(
    data: &ModalInteractionData,
    id: &'static str
) -> Result<Vec<Id<RoleMarker>>, ConfigModalActionError> {
    let ModalInteractionComponent::RoleSelect(select) = field(data, id)?
    else {
        return Err(ConfigModalActionError::InvalidField(id));
    };
    if select.values.len() > 25 {
        return Err(ConfigModalActionError::InvalidField(id));
    }
    Ok(select.values.clone())
}

pub async fn action(
    interaction: &Interaction,
    data: &ModalInteractionData
) -> Result<(), HandleModalSubmitError> {
    let guild_id = interaction.guild_id.ok_or(ConfigModalActionError::NotInGuild)?;
    let immunity = GuildConfigImmunity {
        general: roles(data, "general")?,
        random: roles(data, "random")?,
        general_silent: roles(data, "general_silent")?,
        random_silent: roles(data, "random_silent")?
    };
    STATE.database.update_config(guild_id, |config| config.immunity = immunity).await?;
    reply(
        interaction,
        "Immunity settings saved. Clear a role selection to unset that immunity type."
    )
    .await
}
