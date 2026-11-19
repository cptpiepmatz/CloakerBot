use pulldown_cmark::{Event, Options as ParserOptions, Parser, Tag};
use pulldown_cmark_to_cmark::{
    Options, State, calculate_code_block_token_count, cmark_resume_with_options
};

pub fn parse(input: &str) -> Parser<'_> {
    Parser::new_ext(input, ParserOptions::ENABLE_STRIKETHROUGH)
}

pub fn render<'a>(events: impl IntoIterator<Item = Event<'a>>) -> String {
    let events: Vec<_> = events
        .into_iter()
        .map(|event| match event {
            Event::SoftBreak => Event::Text(" ".into()),
            event => event
        })
        .collect();
    let options = Options {
        newlines_after_headline: 1,
        newlines_after_paragraph: 1,
        newlines_after_codeblock: 1,
        newlines_after_htmlblock: 1,
        newlines_after_table: 1,
        newlines_after_rule: 1,
        newlines_after_list: 1,
        newlines_after_blockquote: 1,
        list_token: '-',
        code_block_token_count: calculate_code_block_token_count(events.iter()).unwrap_or(3).max(3),
        ..Options::default()
    };
    let mut output = String::new();
    let mut state = State::default();
    for chunk in
        events.split_inclusive(|event| matches!(event, Event::Start(Tag::BlockQuote(None))))
    {
        state = cmark_resume_with_options(chunk.iter(), &mut output, Some(state), options.clone())
            .expect("Markdown serialization failed");
        if matches!(chunk.last(), Some(Event::Start(Tag::BlockQuote(None)))) {
            // omit the empty opening quote line introduced by upstream PR #7
            state.newlines_before_start = 0;
        }
    }
    state.finalize(&mut output).expect("Markdown serialization failed");
    output.trim().to_owned()
}

pub fn format(input: &str) -> String {
    render(parse(input))
}
