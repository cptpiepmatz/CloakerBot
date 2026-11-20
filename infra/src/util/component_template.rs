use std::iter;

use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, Container, Separator, SeparatorSpacingSize,
    TextDisplay
};

/// A component tree with text formatted for Discord.
pub enum ComponentTemplate {
    Text(String),
    Separator { divider: Option<bool>, spacing: Option<SeparatorSpacingSize> },
    Container { color: u32, components: Vec<Self> },
    ActionRow(Vec<Self>),
    Button { style: ButtonStyle, custom_id: String, label: String, disabled: bool }
}

impl From<ComponentTemplate> for Component {
    fn from(template: ComponentTemplate) -> Self {
        match template {
            ComponentTemplate::Text(content) => {
                Self::TextDisplay(TextDisplay { id: None, content })
            }
            ComponentTemplate::Separator { divider, spacing } => {
                Self::Separator(Separator { id: None, divider, spacing })
            }
            ComponentTemplate::Container { color, components } => Self::Container(Container {
                id: None,
                accent_color: Some(Some(color)),
                spoiler: None,
                components: components.into_iter().map(Into::into).collect()
            }),
            ComponentTemplate::ActionRow(components) => Self::ActionRow(ActionRow {
                id: None,
                components: components.into_iter().map(Into::into).collect()
            }),
            ComponentTemplate::Button { style, custom_id, label, disabled } => {
                Self::Button(Button {
                    id: None,
                    style,
                    custom_id: Some(custom_id),
                    label: Some(label),
                    disabled,
                    emoji: None,
                    url: None,
                    sku_id: None
                })
            }
        }
    }
}

pub struct Text;
pub struct Iter;

pub trait IntoComponentTemplates<Kind> {
    type Iter: ExactSizeIterator<Item = ComponentTemplate>;
    fn into_components(self) -> Self::Iter;
}

impl<I> IntoComponentTemplates<Iter> for I
where
    I: ExactSizeIterator<Item = ComponentTemplate>
{
    type Iter = I;

    fn into_components(self) -> Self::Iter {
        self
    }
}

impl IntoComponentTemplates<Text> for String {
    type Iter = iter::Once<ComponentTemplate>;

    fn into_components(self) -> Self::Iter {
        iter::once(ComponentTemplate::Text(self))
    }
}

impl IntoComponentTemplates<Text> for &str {
    type Iter = iter::Once<ComponentTemplate>;

    fn into_components(self) -> Self::Iter {
        iter::once(ComponentTemplate::Text(self.to_owned()))
    }
}
