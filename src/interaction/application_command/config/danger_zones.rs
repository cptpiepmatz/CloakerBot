use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::SubCommandBuilder;

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::TryFromInteractionError;

pub const NAME: &str = "danger-zones";
pub const DESCRIPTION: &str = "Define your danger zones.";

pub struct DangerZonesSubCommand;

impl TryFrom<&Vec<CommandDataOption>> for DangerZonesSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        if let Some(option) = options.iter().next() {
            return Err(TryFromInteractionError::UnknownField {
                command: format!("{} {}", super::NAME, NAME),
                field: String::from(&option.name)
            });
        }
        Ok(DangerZonesSubCommand)
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION).build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    _: DangerZonesSubCommand
) -> Result<(), ConfigActionError> {
    todo!()
}
