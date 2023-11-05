use twilight_model::application::interaction::Interaction;

use crate::interaction::HandleInteractionError;

#[derive(Debug)]
pub enum HandleApplicationCommandAutocompleteError {}

pub fn handle_application_command_autocomplete(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandAutocompleteError> {
    todo!()
}
