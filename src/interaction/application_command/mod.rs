use twilight_model::application::interaction::Interaction;

use crate::interaction::HandleInteractionError;

mod cloaker;
mod config;
mod counter;
mod help;
mod ranks;

#[derive(Debug)]
pub enum HandleApplicationCommandError {}

impl From<HandleApplicationCommandError> for HandleInteractionError {
    fn from(value: HandleApplicationCommandError) -> Self {
        HandleInteractionError::ApplicationCommand(value)
    }
}

pub fn handle_application_command(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandError> {
    todo!()
}
