use pulldown_cmark::{Event, Tag, TagEnd};

use crate::markdown;

#[derive(Debug, PartialEq, Eq, Default)]
pub(super) struct Section {
    pub title: String,
    pub heading_level: u8,
    pub content: String
}

pub(super) fn split(input: &str) -> Vec<Section> {
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
