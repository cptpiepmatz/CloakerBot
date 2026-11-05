use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use pulldown_cmark::{Event, Tag, TagEnd};
use quote::{format_ident, quote};
use scraper::{ElementRef, Html, Node as HtmlNode};

use crate::markdown;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Node {
    Text(String),
    Separator { large: Option<bool>, divider: Option<bool> },
    Container { color: u32, children: Vec<Self> },
    ActionRow(Vec<Self>),
    Button { style: String, custom_id: String, label: String, disabled: bool }
}

impl Node {
    pub fn codegen(&self, templates: &mut Vec<String>) -> TokenStream {
        fn text(value: &str, templates: &mut Vec<String>) -> TokenStream {
            let index = templates.len();
            templates.push(value.to_owned());
            quote!(text(#index))
        }

        match self {
            Self::Text(content) => {
                let content = text(content, templates);
                quote!(Component::Text(#content))
            }
            Self::Separator { large, divider } => {
                let spacing = match large {
                    Some(true) => quote!(Some(SeparatorSpacingSize::Large)),
                    Some(false) => quote!(Some(SeparatorSpacingSize::Small)),
                    None => quote!(None)
                };
                let divider = match divider {
                    Some(divider) => quote!(Some(#divider)),
                    None => quote!(None)
                };
                quote!(Component::Separator { divider: #divider, spacing: #spacing })
            }
            Self::Container { color, children } => {
                let children =
                    children.iter().map(|child| child.codegen(templates)).collect::<Vec<_>>();
                quote!(Component::Container { color: #color, components: vec![#(#children),*] })
            }
            Self::ActionRow(children) => {
                let children =
                    children.iter().map(|child| child.codegen(templates)).collect::<Vec<_>>();
                quote!(Component::ActionRow(vec![#(#children),*]))
            }
            Self::Button { style, custom_id, label, disabled } => {
                let style = format_ident!("{style}");
                let custom_id = text(custom_id, templates);
                let label = text(label, templates);
                quote!(Component::Button {
                    style: ButtonStyle::#style, custom_id: #custom_id, label: #label, disabled: #disabled
                })
            }
        }
    }
}

fn flush_markdown(events: &mut Vec<Event<'_>>, nodes: &mut Vec<Node>) {
    if !events.is_empty() {
        let content = markdown::render(events.drain(..));
        if !content.is_empty() {
            nodes.push(Node::Text(content));
        }
    }
}

fn markdown_nodes(input: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut content = Vec::new();
    let mut depth = 0;
    for event in markdown::parse(input) {
        match event {
            Event::Rule if depth == 0 => {
                flush_markdown(&mut content, &mut nodes);
                nodes.push(Node::Separator { large: None, divider: None });
            }
            Event::Start(tag) => {
                if depth == 0 && matches!(tag, Tag::Heading { .. }) {
                    flush_markdown(&mut content, &mut nodes);
                }
                depth += 1;
                content.push(Event::Start(tag));
            }
            Event::End(_) => {
                depth -= 1;
                content.push(event);
            }
            _ => content.push(event)
        }
    }
    flush_markdown(&mut content, &mut nodes);
    nodes
}

fn children(element: ElementRef<'_>, markdown: &[String]) -> Result<Vec<Node>, String> {
    let mut nodes = Vec::new();
    for child in element.children() {
        match child.value() {
            HtmlNode::Element(_) => {
                nodes.push(component(ElementRef::wrap(child).unwrap(), markdown)?);
            }
            HtmlNode::Text(text) => nodes.extend(markdown_nodes(text)),
            HtmlNode::Comment(_) => {}
            _ => return Err("unsupported component HTML".into())
        }
    }
    Ok(nodes)
}

fn component(element: ElementRef<'_>, markdown: &[String]) -> Result<Node, String> {
    let name = element.value().name();
    let mut attributes: BTreeMap<_, _> =
        element.value().attrs().map(|(name, value)| (name.to_owned(), value.to_owned())).collect();
    let node = match name {
        "cloaker-markdown-block" => {
            let index = attributes
                .remove("data-index")
                .and_then(|index| index.parse::<usize>().ok())
                .ok_or("invalid internal Markdown block")?;
            Node::Text(markdown.get(index).ok_or("invalid internal Markdown block")?.clone())
        }
        "hr" => {
            let large = match attributes.remove("size").as_deref() {
                None => None,
                Some("small") => Some(false),
                Some("large") => Some(true),
                _ => return Err("<hr> size must be small or large".into())
            };
            let divider = if attributes.contains_key("divider") {
                Some(boolean(&mut attributes, "divider", true)?)
            }
            else {
                None
            };
            Node::Separator { large, divider }
        }
        "container" => {
            let color =
                attributes.remove("color").ok_or("<container> requires color=\"#rrggbb\"")?;
            let color = color
                .strip_prefix('#')
                .filter(|color| color.len() == 6)
                .and_then(|color| u32::from_str_radix(color, 16).ok())
                .ok_or("container color must be #rrggbb")?;
            let children = children(element, markdown)?;
            if children.is_empty() || children.len() > 10 {
                return Err("<container> requires 1 to 10 child components".into());
            }
            if children
                .iter()
                .any(|child| matches!(child, Node::Container { .. } | Node::Button { .. }))
            {
                return Err("unsupported component inside <container>".into());
            }
            Node::Container { color, children }
        }
        "action-row" => {
            let children = children(element, markdown)?;
            if children.is_empty() ||
                children.len() > 5 ||
                children.iter().any(|child| !matches!(child, Node::Button { .. }))
            {
                return Err("<action-row> requires 1 to 5 buttons".into());
            }
            Node::ActionRow(children)
        }
        "button" => {
            if element.children().any(|child| child.value().is_element()) {
                return Err("<button> labels must be plain text".into());
            }
            let style = match attributes.remove("style").as_deref().unwrap_or("secondary") {
                "primary" => "Primary",
                "secondary" => "Secondary",
                "success" => "Success",
                "danger" => "Danger",
                _ => {
                    return Err(
                        "button style must be primary, secondary, success, or danger".into()
                    );
                }
            };
            let custom_id = attributes
                .remove("custom-id")
                .filter(|id| !id.is_empty())
                .ok_or("<button> requires custom-id, which may contain a placeholder")?;
            let disabled = boolean(&mut attributes, "disabled", false)?;
            let label = element.text().collect::<String>().trim().to_owned();
            if label.is_empty() {
                return Err("<button> requires a label".into());
            }
            Node::Button { style: style.to_owned(), custom_id, label, disabled }
        }
        _ => return Err(format!("unsupported component element <{name}>"))
    };
    no_attributes(&attributes)?;
    Ok(node)
}

fn boolean(
    attributes: &mut BTreeMap<String, String>,
    name: &str,
    default: bool
) -> Result<bool, String> {
    match attributes.remove(name).as_deref() {
        None => Ok(default),
        Some("true" | "") => Ok(true),
        Some("false") => Ok(false),
        _ => Err(format!("{name} must be true or false"))
    }
}

fn no_attributes(attributes: &BTreeMap<String, String>) -> Result<(), String> {
    match attributes.keys().next() {
        Some(name) => Err(format!("unsupported component attribute {name}")),
        None => Ok(())
    }
}

pub(super) fn parse(input: &str) -> Result<Vec<Node>, String> {
    fn flush(events: &mut Vec<Event<'_>>, html: &mut String, blocks: &mut Vec<String>) {
        let text = markdown::render(events.drain(..));
        if !text.is_empty() {
            // keep Markdown opaque so HTML inside code blocks stays literal
            let index = blocks.len();
            blocks.push(text);
            html.push_str(&format!(
                "<cloaker-markdown-block data-index=\"{index}\"></cloaker-markdown-block>"
            ));
        }
    }

    let mut html = String::new();
    let mut blocks = Vec::new();
    let mut content = Vec::new();
    let mut depth = 0;
    for event in markdown::parse(input) {
        match event {
            Event::Start(Tag::HtmlBlock) | Event::End(TagEnd::HtmlBlock) => {}
            Event::Html(raw) if depth == 0 => {
                flush(&mut content, &mut html, &mut blocks);
                html.push_str(&raw);
            }
            Event::Rule if depth == 0 => {
                flush(&mut content, &mut html, &mut blocks);
                html.push_str("<hr>");
            }
            Event::Start(tag) => {
                if depth == 0 && matches!(tag, Tag::Heading { .. }) {
                    flush(&mut content, &mut html, &mut blocks);
                }
                depth += 1;
                content.push(Event::Start(tag));
            }
            Event::End(_) => {
                depth -= 1;
                content.push(event);
            }
            _ => content.push(event)
        }
    }
    flush(&mut content, &mut html, &mut blocks);
    let fragment = Html::parse_fragment(&html);
    let nodes = children(fragment.root_element(), &blocks)?;
    if nodes.iter().any(|node| matches!(node, Node::Button { .. })) {
        return Err("<button> must be inside <action-row>".into());
    }
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::{Node, parse};

    #[test]
    fn splits_markdown_components() {
        let input = indoc! {"
            Before
            the headings.

            # Update
            Wrapped **markdown**
            with {CloakerBot}.

            Another paragraph.

            ## Details ##
            - first

            - second

            ```md
            # Not a heading
              preserved indentation
            ```
            A hard break\x20\x20
            continues here.

            ###### Empty
        "};
        assert_eq!(parse(input).unwrap(), vec![
            Node::Text("Before the headings.".into()),
            Node::Text(
                indoc! {"
                # Update
                Wrapped **markdown** with {CloakerBot}.
                Another paragraph.
            "}
                .trim_end()
                .into()
            ),
            Node::Text(
                indoc! {"
                ## Details
                - first
                - second
                ```md
                # Not a heading
                  preserved indentation
                ```
                A hard break\x20\x20
                continues here.
            "}
                .trim_end()
                .into()
            ),
            Node::Text("###### Empty".into())
        ]);
    }
}
