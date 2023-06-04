use twilight_model::application::interaction::Interaction;

use crate::impl_from_variant;
use crate::interaction::HandleInteractionError;

#[derive(Debug)]
pub enum HandleApplicationCommandAutocompleteError {}

impl_from_variant!(
    HandleApplicationCommandAutocompleteError,
    HandleInteractionError,
    ApplicationCommandAutocomplete
);

pub fn handle_application_command_autocomplete(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandAutocompleteError> {
    todo!()
}
