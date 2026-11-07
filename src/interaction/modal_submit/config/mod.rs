pub mod danger_zones;
pub mod immunity;
pub mod log_channel;
pub mod random;
pub mod target_bots;

use std::fmt::Display;

use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{
    Label, SelectDefaultValue, SelectMenu, SelectMenuType
};
use twilight_model::http::interaction::InteractionResponseData;
use twilight_util::builder::InteractionResponseDataBuilder;

use super::custom_id::ModalCustomId;

#[derive(Debug)]
pub enum ConfigModalActionError {
    NotInGuild,
    InvalidField(&'static str),
    UnexpectedCustomId(ModalCustomId)
}

fn label(name: impl Display, description: impl Display, component: Component) -> Component {
    Component::Label(Label {
        id: None,
        label: name.to_string(),
        description: Some(description.to_string()),
        component: Box::new(component)
    })
}

fn select(
    custom_id: &str,
    kind: SelectMenuType,
    default_values: Vec<SelectDefaultValue>,
    max_values: u8
) -> SelectMenu {
    SelectMenu {
        id: None,
        custom_id: custom_id.into(),
        kind,
        default_values: Some(default_values),
        channel_types: None,
        disabled: false,
        required: Some(false),
        min_values: Some(0),
        max_values: Some(max_values),
        options: None,
        placeholder: None
    }
}

fn modal(
    custom_id: ModalCustomId,
    title: impl Display,
    components: impl IntoIterator<Item = Component>
) -> InteractionResponseData {
    InteractionResponseDataBuilder::new()
        .custom_id(custom_id.to_string())
        .title(title.to_string())
        .components(components)
        .build()
}
