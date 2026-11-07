use std::fmt::{self, Display};

use itertools::Itertools;
use twilight_mention::Mention;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{
    SelectDefaultValue, SelectMenuType, SeparatorSpacingSize
};
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::id::Id;
use twilight_model::id::marker::RoleMarker;

use super::ConfigModalActionError;
use crate::STATE;
use crate::assets::replies;
use crate::data::config::GuildConfigImmunity;
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};
use crate::util::component_template::ComponentTemplate;

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
    use ModalInteractionComponent as MIC;
    match field(data, id)? {
        MIC::RoleSelect(select) if select.values.len() <= 25 => Ok(select.values.clone()),
        _ => Err(ConfigModalActionError::InvalidField(id))
    }
}

fn format_roles(roles: &[Id<RoleMarker>]) -> impl Display + Sync + '_ {
    fmt::from_fn(move |f| {
        if roles.is_empty() {
            return write!(f, "{}", replies::config::immunity::none());
        }

        let mut iter = roles.into_iter().peekable();
        while let Some(id) = iter.next() {
            write!(f, "- {}", id.mention())?;
            if iter.peek().is_some() {
                write!(f, "\n")?;
            }
        }

        Ok(())
    })
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

    fn text(content: impl Display) -> ComponentTemplate {
        ComponentTemplate::Text(content.to_string())
    }

    fn separator(spacing: SeparatorSpacingSize) -> ComponentTemplate {
        ComponentTemplate::Separator { divider: Some(false), spacing: Some(spacing) }
    }

    use replies::config::immunity::*;
    let components = [
        text(saved()),
        text(general_roles(format_roles(&immunity.general))),
        separator(SeparatorSpacingSize::Small),
        text(random_roles(format_roles(&immunity.random))),
        separator(SeparatorSpacingSize::Small),
        text(general_silent_roles(format_roles(&immunity.general_silent))),
        separator(SeparatorSpacingSize::Small),
        text(random_silent_roles(format_roles(&immunity.random_silent))),
        separator(SeparatorSpacingSize::Large),
        text(unset_hint())
    ];

    STATE.database.update_config(guild_id, |config| config.immunity = immunity.clone()).await?;
    reply(interaction, components.into_iter()).await
}
