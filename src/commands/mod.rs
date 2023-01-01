mod cloaker;
mod config;
mod counter;
mod help;
mod ranks;

use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
use twilight_model::id::Id;

pub async fn set_commands(client: &InteractionClient<'_>) -> Result<(), TwilightHttpError> {
    let _res = client
        .set_guild_commands(Id::new(581884294495207444), &[
            cloaker::chat_input(),
            cloaker::user_context_menu(),
            cloaker::message_context_menu(),
            counter::chat_input(),
            counter::user_context_menu(),
            counter::message_context_menu(),
            config::chat_input(),
            ranks::chat_input(),
            help::chat_input()
        ])
        .await?;
    Ok(())
}
