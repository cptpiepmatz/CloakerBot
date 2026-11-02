use derive_more::From;
use twilight_model::application::interaction::{Interaction, InteractionType};
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::HandleDiscordEventError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, handle_application_command
};
use crate::interaction::application_command_autocomplete::{
    HandleApplicationCommandAutocompleteError, handle_application_command_autocomplete
};
use crate::interaction::message_component::{
    HandleMessageComponentError, handle_message_component
};

mod application_command;
mod application_command_autocomplete;
pub(crate) mod message_component;
mod modal_submit;

pub use application_command::{COMMANDS_REGISTER, CommandsRegister};

pub fn owner_update() -> impl Iterator<Item = String> {
    use application_command::{config, counter, help};
    use twilight_mention::Mention;

    let replacements = [
        ("{CloakerBot}", crate::BOT_USER_ID.mention().to_string()),
        ("{/config log-channel}", config::MENTION.to_string()),
        ("{/config danger-zones}", config::MENTION.to_string()),
        ("{/config immunity}", config::MENTION.to_string()),
        ("{/counter}", counter::MENTION.to_string()),
        ("{/help}", help::MENTION.to_string())
    ];
    crate::assets::owner_notifications::MSG01_V3_UPDATE_SECTIONS.iter().map(move |section| {
        let content = if section.heading_level == 0 {
            section.content.to_owned()
        }
        else {
            format!(
                "{} {}\n\n{}",
                "#".repeat(section.heading_level.into()),
                section.title,
                section.content
            )
        };
        replacements.iter().fold(content, |content, (placeholder, replacement)| {
            content.replace(placeholder, replacement)
        })
    })
}

#[derive(Debug, From)]
pub enum HandleInteractionError {
    MessageComponent(HandleMessageComponentError),
    ModalSubmit(modal_submit::HandleModalSubmitError),
    ApplicationCommand(HandleApplicationCommandError),
    ApplicationCommandAutocomplete(HandleApplicationCommandAutocompleteError)
}

#[tracing::instrument(skip_all, fields(
    interaction_id = %interaction_create.id,
    guild_id = interaction_create.guild_id.map(|id| id.get()),
    interaction_kind = ?interaction_create.kind
), err(Debug))]
pub async fn handle_interaction(
    interaction_create: &InteractionCreate
) -> Result<(), HandleInteractionError> {
    let interaction: &Interaction = interaction_create;
    tracing::debug!("handling interaction");
    match interaction.kind {
        InteractionType::MessageComponent => handle_message_component(interaction).await?,
        InteractionType::ModalSubmit => modal_submit::handle_modal_submit(interaction).await?,
        InteractionType::ApplicationCommand => handle_application_command(interaction).await?,
        InteractionType::ApplicationCommandAutocomplete => {
            handle_application_command_autocomplete(interaction).await?
        }

        _ => ()
    }

    Ok(())
}
