use pulldown_cmark::{Parser, Event, Tag, TagEnd};

pub fn render_markdown(input: &str) -> String {
    let parser = Parser::new(input);

    let mut out = String::new();

    for event in parser {
        match event {
            Event::Text(text) => out.push_str(&text),
            Event::Start(Tag::Paragraph) => out.push('\n'),
            Event::End(TagEnd::Paragraph) => out.push('\n'),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),

            _ => {}
        }
    }

    out
}
