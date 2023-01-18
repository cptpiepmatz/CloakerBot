use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use twilight_mention::Mention;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ButtonStyle, ComponentType};
use twilight_model::channel::ChannelType;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{StringBuilder, SubCommandBuilder};

use crate::db::config::GuildConfig;
use crate::interaction::application_command::TryFromInteractionError;
use crate::interaction::message_component;
use crate::interaction::message_component::config_immunity::{BUTTON_CUSTOM_ID, SELECT_CUSTOM_ID};
use crate::{Com, APPLICATION_ID};

pub const NAME: &str = "immunity";
const DESCRIPTION: &str = "Define what roles are immune to takedowns.";

pub enum ImmunitySubCommand {
    General,
    Random
}

impl TryFrom<&Vec<CommandDataOption>> for ImmunitySubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut for_general = false;
        let mut for_random = false;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "type" => match &option.value {
                    CommandOptionValue::String(s) => match s.as_str() {
                        "general" => for_general = true,
                        "random" => for_random = true,
                        _ => {
                            return Err(TryFromInteractionError::IncorrectFieldValue {
                                command,
                                field: "type".into(),
                                value: option.value.clone()
                            })
                        }
                    },
                    n => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "type".into(),
                            value: option.value.clone()
                        })
                    }
                },
                n => {
                    return Err(TryFromInteractionError::UnknownField {
                        command,
                        field: n.into()
                    })
                }
            }
        }

        match (for_general, for_random) {
            (true, _) => Ok(ImmunitySubCommand::General),
            (_, true) => Ok(ImmunitySubCommand::Random),
            (false, false) => Err(TryFromInteractionError::MissingField {
                command,
                field: "type".into()
            })
        }
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            StringBuilder::new("type", "Select which type of immunity role.")
                .choices([
                    ("general".to_owned(), "general".to_owned()),
                    ("random".to_owned(), "random".to_owned())
                ])
                .required(true)
        )
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    command: ImmunitySubCommand,
    com: Arc<Com>
) -> anyhow::Result<()> {
    use ImmunitySubCommand::{General, Random};

    let config = GuildConfig::fetch(guild_id, com.clone()).await?;
    let role_id = match command {
        General => config.immunity.general,
        Random => config.immunity.random
    };

    let content = match (&command, role_id) {
        (General, None) => "**🛠️ *No* role is marked to be immune to *all* takedowns.**".into(),
        (Random, None) => "**🛠️ *No* role is marked to be immune to *random* takedowns.**".into(),
        (General, Some(role_id)) => format!(
            "**🛠️ Everyone with the role {} is immune to *all* takedowns.**",
            role_id.mention()
        ),
        (Random, Some(role_id)) => format!(
            "**🛠️ Everyone with the role {} is immune to *random* takedowns.**",
            role_id.mention()
        )
    };

    let (select_custom_id, button_custom_id) = match &command {
        General => (SELECT_CUSTOM_ID.general, BUTTON_CUSTOM_ID.general),
        Random => (SELECT_CUSTOM_ID.random, BUTTON_CUSTOM_ID.random)
    };
    let payload = json!({
        "content": content,
        "components": [
            {
                "type": ComponentType::ActionRow,
                "components": [{
                    // select menu for channel select
                    "type": 6,
                    "custom_id": format!("{}#{}", select_custom_id, interaction.id),
                    "min_values": 0,
                    "max_values": 1,
                    "placeholder": "🛟 Select role to be immune."
                }]
            },
            {
                "type": ComponentType::ActionRow,
                "components": [{
                    // button to remove all channels
                    "type": ComponentType::Button,
                    "custom_id": format!("{}#{}", button_custom_id, interaction.id),
                    "label": "Delete Immune Role",
                    "style": ButtonStyle::Danger,
                    "emoji": {
                        "id": null,
                        "name": "🗑️"
                    }
                }]
            }
        ]
    });

    com.http
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .payload_json(payload.to_string().as_bytes())
        .await?;

    let interaction_id = interaction.id;
    com.tracked_interactions.insert(interaction.id, interaction);

    tokio::time::sleep(Duration::from_secs(3 * 60)).await;

    if let Some((_, interaction)) = com.tracked_interactions.remove(&interaction_id) {
        let payload = json!({
            "content": format!(
                "**⌛ Timed Out.**\nTry `/{} {}` again to modify immune roles.",
                super::NAME,
                NAME
            ),
            "components": []
        });

        com.http
            .interaction(APPLICATION_ID)
            .update_response(&interaction.token)
            .payload_json(payload.to_string().as_bytes())
            .await?;
    }

    Ok(())
}
