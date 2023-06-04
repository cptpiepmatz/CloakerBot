use twilight_model::application::interaction::Interaction;

use crate::interaction::HandleInteractionError;

#[derive(Debug)]
pub enum HandleMessageComponentError {}

impl From<HandleMessageComponentError> for HandleInteractionError {
    fn from(value: HandleMessageComponentError) -> Self {
        HandleInteractionError::MessageComponent(value)
    }
}

pub fn handle_message_component(
    interaction: &Interaction
) -> Result<(), HandleMessageComponentError> {
    todo!()
}
