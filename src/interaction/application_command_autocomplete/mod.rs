use derive_more::From;
use twilight_model::application::command::{CommandOptionChoice, CommandOptionChoiceValue};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::application_command::cloaker;
use crate::{APPLICATION_ID, STATE};

#[derive(Debug, From)]
pub enum HandleApplicationCommandAutocompleteError {
    Http(twilight_http::Error)
}

pub async fn handle_application_command_autocomplete(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandAutocompleteError> {
    let choices = match interaction.data.as_ref() {
        Some(InteractionData::ApplicationCommand(data)) if data.name == cloaker::NAME => data
            .options
            .iter()
            .find_map(|option| match (&*option.name, &option.value) {
                ("target", CommandOptionValue::Focused(query, _)) => Some(query.to_lowercase()),
                _ => None
            })
            .map(|query| cloaker_choices(interaction, &query))
            .unwrap_or_default(),
        _ => Vec::new()
    };
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
            data: Some(InteractionResponseDataBuilder::new().choices(choices).build())
        })
        .await?;
    Ok(())
}

fn cloaker_choices(interaction: &Interaction, query: &str) -> Vec<CommandOptionChoice> {
    let Some(guild_id) = interaction.guild_id
    else {
        return Vec::new();
    };
    let caller_channel = interaction.author_id().and_then(|user_id| {
        STATE.cache.voice_state(user_id, guild_id).map(|state| state.channel_id())
    });
    let Some(voice_states) = STATE.cache.guild_voice_states(guild_id)
    else {
        return Vec::new();
    };
    let mut candidates = voice_states
        .iter()
        .filter_map(|&user_id| {
            let channel_id = STATE.cache.voice_state(user_id, guild_id)?.channel_id();
            let user = STATE.cache.user(user_id);
            let nickname = STATE
                .cache
                .member(guild_id, user_id)
                .and_then(|member| member.nick().map(str::to_owned));
            let username = user.as_ref().map(|user| user.name.as_str());
            let global_name = user.as_ref().and_then(|user| user.global_name.as_deref());
            let id = user_id.to_string();
            if ![nickname.as_deref(), global_name, username, Some(id.as_str())]
                .into_iter()
                .flatten()
                .any(|name| name.to_lowercase().contains(query))
            {
                return None;
            }
            let display_name = nickname.as_deref().or(global_name).or(username).unwrap_or(&id);
            let label = match username {
                Some(username) if username != display_name => {
                    format!("{display_name} (@{username})")
                }
                _ => display_name.to_owned()
            };
            Some((caller_channel != Some(channel_id), label.to_lowercase(), user_id, label))
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable_by(|a, b| (&a.0, &a.1, &a.2).cmp(&(&b.0, &b.1, &b.2)));
    candidates
        .into_iter()
        .take(25)
        .map(|(_, _, user_id, label)| CommandOptionChoice {
            name: label.chars().take(100).collect(),
            name_localizations: None,
            value: CommandOptionChoiceValue::String(user_id.to_string())
        })
        .collect()
}
