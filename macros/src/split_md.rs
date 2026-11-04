use std::fs;

use proc_macro2::TokenStream;
use pulldown_cmark::{Event, Tag, TagEnd};
use quote::quote;
use syn::LitStr;

use crate::markdown;

#[derive(Debug, PartialEq, Eq, Default)]
struct Section {
    title: String,
    heading_level: u8,
    content: String
}

pub fn macro_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let file_path: LitStr = syn::parse2(input)?;
    let path = crate::resolve_path(&file_path)?;
    let content =
        fs::read_to_string(&path).map_err(|error| syn::Error::new(file_path.span(), error))?;
    let sections = split(&content);
    let records = sections.iter().map(|Section { title, heading_level, content }| {
        quote! {
            crate::assets::MarkdownSection {
                title: #title,
                heading_level: #heading_level,
                content: #content
            }
        }
    });

    Ok(quote! {{
        const _: &str = std::include_str!(#file_path);
        const SECTIONS: &[crate::assets::MarkdownSection] = &[#(#records),*];
        SECTIONS
    }})
}

fn split(input: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current = Section::default();
    let mut content = Vec::new();
    let mut depth = 0;
    let mut events = markdown::parse(input);
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) if depth == 0 => {
                current.content = markdown::render(content.drain(..));
                if current.heading_level != 0 || !current.content.is_empty() {
                    sections.push(current);
                }
                let title = markdown::render(
                    events
                        .by_ref()
                        .take_while(|event| !matches!(event, Event::End(TagEnd::Heading(_))))
                );
                current = Section { title, heading_level: level as u8, content: String::new() };
            }
            Event::Start(_) => {
                depth += 1;
                content.push(event);
            }
            Event::End(_) => {
                depth -= 1;
                content.push(event);
            }
            _ => content.push(event)
        }
    }
    current.content = markdown::render(content);
    if current.heading_level != 0 || !current.content.is_empty() {
        sections.push(current);
    }
    sections
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::{Section, split};

    #[test]
    fn splits_markdown_sections() {
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
        assert_eq!(split(input), vec![
            Section { title: "".into(), heading_level: 0, content: "Before the headings.".into() },
            Section {
                title: "Update".into(),
                heading_level: 1,
                content: indoc! {"
                    Wrapped **markdown** with {CloakerBot}.
                    Another paragraph.
                "}
                .trim_end()
                .into()
            },
            Section {
                title: "Details".into(),
                heading_level: 2,
                content: indoc! {"
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
            },
            Section { title: "Empty".into(), heading_level: 6, content: "".into() }
        ]);
    }
}
