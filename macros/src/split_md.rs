use std::fs;

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

use crate::template;

#[derive(Debug, PartialEq, Eq)]
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
    let file_path = path.to_string_lossy();
    Ok(quote! {{
        const _: &str = std::include_str!(#file_path);
        const SECTIONS: &[crate::assets::MarkdownSection] = &[#(#records),*];
        SECTIONS
    }})
}

fn fence(line: &str) -> Option<(char, usize)> {
    let line = line.trim_start();
    let marker = line.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let count = line.chars().take_while(|character| *character == marker).count();
    (count >= 3).then_some((marker, count))
}

fn normalize(input: &str) -> String {
    let mut output = Vec::new();
    let mut prose = String::new();
    let mut code = None;
    for line in input.lines() {
        let boundary = fence(line);
        if code.is_some() || boundary.is_some() {
            if !prose.is_empty() {
                output.push(template::format_markdown(&prose));
                prose.clear();
            }
            output.push(line.to_owned());
            if let Some((marker, length)) = code {
                if boundary.is_some_and(|(end, count)| end == marker && count >= length) &&
                    line.trim().chars().all(|character| character == marker)
                {
                    code = None;
                }
            }
            else {
                code = boundary;
            }
        }
        else if line.ends_with("  ") || line.ends_with('\\') {
            prose.push_str(line.trim_end());
            output.push(template::format_markdown(&prose));
            prose.clear();
        }
        else {
            prose.push_str(line);
            prose.push('\n');
        }
    }
    if !prose.is_empty() {
        output.push(template::format_markdown(&prose));
    }
    output.join("\n").trim().to_owned()
}

fn split(input: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current = Section { title: String::new(), heading_level: 0, content: String::new() };
    let mut code = None;
    for line in input.lines() {
        if let Some((marker, length)) = code {
            if fence(line).is_some_and(|(end, count)| end == marker && count >= length) &&
                line.trim().chars().all(|character| character == marker)
            {
                code = None;
            }
        }
        else if let Some(boundary) = fence(line) {
            code = Some(boundary);
        }
        else {
            let trimmed = line.trim_start_matches(' ');
            let level = trimmed.chars().take_while(|character| *character == '#').count();
            if line.len() - trimmed.len() <= 3 &&
                (1..=6).contains(&level) &&
                (trimmed.len() == level || trimmed.as_bytes()[level].is_ascii_whitespace())
            {
                if current.heading_level != 0 || !current.content.trim().is_empty() {
                    current.content = normalize(&current.content);
                    sections.push(current);
                }
                let title = trimmed[level..].trim();
                let without_closing = title.trim_end_matches('#');
                let title = if without_closing.ends_with(char::is_whitespace) {
                    without_closing.trim_end()
                }
                else {
                    title
                };
                current = Section {
                    title: title.to_owned(),
                    heading_level: level as u8,
                    content: String::new()
                };
                continue;
            }
        }
        current.content.push_str(line);
        current.content.push('\n');
    }
    if current.heading_level != 0 || !current.content.trim().is_empty() {
        current.content = normalize(&current.content);
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
                    A hard break
                    continues here.
                "}
                .trim_end()
                .into()
            },
            Section { title: "Empty".into(), heading_level: 6, content: "".into() }
        ]);
    }
}
