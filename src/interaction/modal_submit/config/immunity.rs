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
use crate::assets::templates;
use crate::data::config::GuildConfigImmunity;
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};
use crate::util::component_template::ComponentTemplate;

pub fn modal(current: &GuildConfigImmunity) -> InteractionResponseData {
    use templates::config::immunity::modal;
    let fields = [
        (
            "general",
            modal::general::label().to_string(),
            modal::general::description().to_string(),
            &current.general
        ),
        (
            "random",
            modal::random::label().to_string(),
            modal::random::description().to_string(),
            &current.random
        ),
        (
            "general_silent",
            modal::general_silent::label().to_string(),
            modal::general_silent::description().to_string(),
            &current.general_silent
        ),
        (
            "random_silent",
            modal::random_silent::label().to_string(),
            modal::random_silent::description().to_string(),
            &current.random_silent
        )
    ];
    super::modal(
        ModalCustomId::ConfigImmunity,
        modal::title(),
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
            return write!(f, "{}", templates::config::immunity::none());
        }

        let mut iter = roles.iter().peekable();
        while let Some(id) = iter.next() {
            write!(f, "- {}", id.mention())?;
            if iter.peek().is_some() {
                writeln!(f)?;
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

    use templates::config::immunity::*;
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
