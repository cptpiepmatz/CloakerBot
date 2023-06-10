use twilight_model::application::interaction::Interaction;

use crate::interaction::HandleInteractionError;
use crate::util::impl_from_variant::impl_from_variant;

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
