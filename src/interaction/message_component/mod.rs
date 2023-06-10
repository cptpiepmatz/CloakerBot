use twilight_model::application::interaction::Interaction;

use crate::interaction::HandleInteractionError;
use crate::util::impl_from_variant::impl_from_variant;

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
