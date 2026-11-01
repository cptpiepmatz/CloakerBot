use itertools::Itertools;
use twilight_mention::Mention;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::modal::{
    ModalInteractionComponent, ModalInteractionData
};
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{SelectDefaultValue, SelectMenuType};
use twilight_model::http::interaction::InteractionResponseData;

use super::ConfigModalActionError;
use crate::assets::QUOTES;
use crate::assets::replies::config;
use crate::data::config::GuildConfigImmunity;
use crate::interaction::modal_submit::custom_id::ModalCustomId;
use crate::interaction::modal_submit::{HandleModalSubmitError, field, reply};
use crate::{BOT_USER_ID, STATE};

#[derive(Debug, Copy, Clone)]
pub enum ImmunityModal {
    General,
    Random,
    GeneralSilent,
    RandomSilent
}

impl TryFrom<ModalCustomId> for ImmunityModal {
    type Error = ConfigModalActionError;

    fn try_from(value: ModalCustomId) -> Result<Self, Self::Error> {
        Ok(match value {
            ModalCustomId::ConfigImmunityGeneral => Self::General,
            ModalCustomId::ConfigImmunityRandom => Self::Random,
            ModalCustomId::ConfigImmunityGeneralSilent => Self::GeneralSilent,
            ModalCustomId::ConfigImmunityRandomSilent => Self::RandomSilent,
            custom_id => return Err(ConfigModalActionError::UnexpectedCustomId(custom_id))
        })
    }
}

pub fn modal(command: ImmunityModal, current: &GuildConfigImmunity) -> InteractionResponseData {
    let (custom_id, title, roles) = match command {
        ImmunityModal::General => {
            (ModalCustomId::ConfigImmunityGeneral, "General immunity", &current.general)
        }
        ImmunityModal::Random => {
            (ModalCustomId::ConfigImmunityRandom, "Random immunity", &current.random)
        }
        ImmunityModal::GeneralSilent => (
            ModalCustomId::ConfigImmunityGeneralSilent,
            "Silent general immunity",
            &current.general_silent
        ),
        ImmunityModal::RandomSilent => (
            ModalCustomId::ConfigImmunityRandomSilent,
            "Silent random immunity",
            &current.random_silent
        )
    };
    let select = super::select(
        "roles",
        SelectMenuType::Role,
        roles.iter().copied().map(SelectDefaultValue::Role).collect(),
        25
    );
    super::modal(custom_id, title, [super::label(
        "Immune roles",
        "Members with these roles are immune. Clear the selection to unset.",
        Component::SelectMenu(select)
    )])
}

pub async fn action(
    interaction: &Interaction,
    custom_id: ModalCustomId,
    data: &ModalInteractionData
) -> Result<(), HandleModalSubmitError> {
    let guild_id = interaction.guild_id.ok_or(ConfigModalActionError::NotInGuild)?;
    let ModalInteractionComponent::RoleSelect(select) = field(data, "roles")?
    else {
        return Err(ConfigModalActionError::InvalidField("roles").into());
    };
    if select.values.len() > 25 {
        return Err(ConfigModalActionError::InvalidField("roles").into());
    }
    let variant = ImmunityModal::try_from(custom_id)?;
    use ImmunityModal as IM;
    STATE
        .database
        .update_config(guild_id, |config| {
            let roles = match variant {
                IM::General => &mut config.immunity.general,
                IM::Random => &mut config.immunity.random,
                IM::GeneralSilent => &mut config.immunity.general_silent,
                IM::RandomSilent => &mut config.immunity.random_silent
            };
            *roles = select.values.clone();
        })
        .await?;

    let roles = select.values.iter().map(|id| format!("- {}", id.mention())).join("\n");
    let content = match (select.values.is_empty(), variant) {
        (true, IM::General) => {
            config::immunity::general::unset(QUOTES.config.immunity.general.no_roles)
        }
        (true, IM::Random) => {
            config::immunity::random::unset(QUOTES.config.immunity.random.no_roles)
        }
        (true, IM::GeneralSilent) => {
            config::immunity::general::silent::unset(QUOTES.config.immunity.general.no_roles)
        }
        (true, IM::RandomSilent) => {
            config::immunity::random::silent::unset(QUOTES.config.immunity.random.no_roles)
        }
        (false, IM::General) => config::immunity::general::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.general.roles,
            roles
        ),
        (false, IM::Random) => config::immunity::random::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.random.roles,
            roles
        ),
        (false, IM::GeneralSilent) => config::immunity::general::silent::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.general.roles,
            roles
        ),
        (false, IM::RandomSilent) => config::immunity::random::silent::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.random.roles,
            roles
        )
    };
    reply(interaction, content).await
}
