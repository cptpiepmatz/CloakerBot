use std::fmt::Display;

use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, Container, Separator, SeparatorSpacingSize,
    TextDisplay
};

/// A component tree whose text is formatted only when converted for Discord.
pub enum TemplateComponent<D> {
    Text(D),
    Separator { divider: Option<bool>, spacing: Option<SeparatorSpacingSize> },
    Container { color: u32, components: Vec<Self> },
    ActionRow(Vec<Self>),
    Button { style: ButtonStyle, custom_id: D, label: D, disabled: bool }
}

impl<D: Display> From<TemplateComponent<D>> for Component {
    fn from(template: TemplateComponent<D>) -> Self {
        match template {
            TemplateComponent::Text(content) => {
                Self::TextDisplay(TextDisplay { id: None, content: content.to_string() })
            }
            TemplateComponent::Separator { divider, spacing } => {
                Self::Separator(Separator { id: None, divider, spacing })
            }
            TemplateComponent::Container { color, components } => Self::Container(Container {
                id: None,
                accent_color: Some(Some(color)),
                spoiler: None,
                components: components.into_iter().map(Into::into).collect()
            }),
            TemplateComponent::ActionRow(components) => Self::ActionRow(ActionRow {
                id: None,
                components: components.into_iter().map(Into::into).collect()
            }),
            TemplateComponent::Button { style, custom_id, label, disabled } => {
                Self::Button(Button {
                    id: None,
                    style,
                    custom_id: Some(custom_id.to_string()),
                    label: Some(label.to_string()),
                    disabled,
                    emoji: None,
                    url: None,
                    sku_id: None
                })
            }
        }
    }
}
