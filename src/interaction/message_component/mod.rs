use twilight_model::application::interaction::Interaction;

use crate::impl_from_variant;
use crate::interaction::HandleInteractionError;

#[derive(Debug)]
pub enum HandleMessageComponentError {}

impl_from_variant!(
    HandleMessageComponentError,
    HandleInteractionError,
    MessageComponent
);

pub fn handle_message_component(
    interaction: &Interaction
) -> Result<(), HandleMessageComponentError> {
    todo!()
}
