use std::borrow::Cow;

use crate::{
    markdown::{link::parse_markdown_links, wikilink::parse_wikilinks},
    render::{Colour, Style},
};
use pulldown_cmark::{Event, Parser, Tag};

pub mod frontmatter;
pub mod link;
pub mod wikilink;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NoteLink<'a> {
    pub target: Cow<'a, str>,
    pub heading: Option<Cow<'a, str>>,
    pub display: Cow<'a, str>,
    pub span: (usize, usize),
}

pub fn extract_note_links<'a>(text: &'a str) -> Vec<NoteLink<'a>> {
    let mut links = Vec::new();

    for w in parse_wikilinks(text) {
        links.push(NoteLink {
            target: Cow::Borrowed(w.target),
            heading: w.heading.map(Cow::Borrowed),
            display: Cow::Borrowed(w.display),
            span: w.span,
        });
    }

    for m in parse_markdown_links(text) {
        if !m.is_external() {
            links.push(NoteLink {
                target: m.dest,
                heading: m.heading,
                display: m.display,
                span: m.span,
            });
        }
    }

    links.sort_by_key(|l| l.span.0);
    links
}

#[derive(Debug, PartialEq, Clone)]
pub enum HighlightKind {
    Heading,
    Emphasis,
    Bold,
    Code,
    Link,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Highlight {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

pub fn highlight(text: &str) -> Vec<Highlight> {
    let mut spans = Vec::new();

    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                spans.push(Highlight {
                    start: range.start,
                    end: range.end,
                    kind: HighlightKind::Heading,
                });
            }
            Event::Start(Tag::Emphasis) => {
                spans.push(Highlight {
                    start: range.start,
                    end: range.end,
                    kind: HighlightKind::Emphasis,
                });
            }
            Event::Start(Tag::Strong) => {
                spans.push(Highlight {
                    start: range.start,
                    end: range.end,
                    kind: HighlightKind::Bold,
                });
            }
            Event::Code(_) => {
                spans.push(Highlight {
                    start: range.start,
                    end: range.end,
                    kind: HighlightKind::Code,
                });
            }
            Event::Start(Tag::Link { .. }) => {
                spans.push(Highlight {
                    start: range.start,
                    end: range.end,
                    kind: HighlightKind::Link,
                });
            }
            _ => {}
        }
    }

    spans
}

pub fn style_for(kind: HighlightKind) -> Style {
    match kind {
        HighlightKind::Heading => Style::new(true, Some(Colour::Rgb(190, 180, 255)), None),
        HighlightKind::Emphasis => Style::new(false, Some(Colour::Rgb(220, 160, 180)), None),
        HighlightKind::Bold => Style::new(true, Some(Colour::Rgb(240, 230, 210)), None),
        HighlightKind::Code => Style::new(
            false,
            Some(Colour::Rgb(180, 200, 180)),
            Some(Colour::Rgb(40, 44, 52)),
        ),
        HighlightKind::Link => Style::new(false, Some(Colour::Rgb(100, 180, 255)), None),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        markdown::{HighlightKind, extract_note_links},
        render::Colour,
    };

    #[test]
    fn highlight_atx_heading() {
        let text = "# Heading";
        let spans = super::highlight(text);

        assert!(
            spans
                .iter()
                .any(|h| { h.kind == HighlightKind::Heading && h.start == 0 && h.end == 9 }),
            "expected a Heading highlight for {text:?}, got {spans:?}"
        );
    }

    #[test]
    fn highlight_emphasis() {
        let text = "*hi*";
        let spans = super::highlight(text);

        assert!(
            spans
                .iter()
                .any(|h| { h.kind == HighlightKind::Emphasis && h.start == 0 && h.end == 4 }),
            "expected Emphasis for {text:?}, got {spans:?}"
        );
    }

    #[test]
    fn highlight_bold() {
        let text = "**bold**";
        let spans = super::highlight(text);

        assert!(
            spans
                .iter()
                .any(|h| { h.kind == HighlightKind::Bold && h.start == 0 && h.end == 8 }),
            "expected Bold for {text:?}, got {spans:?}"
        );
    }

    #[test]
    fn highlight_code() {
        let text = "`code`";
        let spans = super::highlight(text);

        assert!(
            spans
                .iter()
                .any(|h| { h.kind == HighlightKind::Code && h.start == 0 && h.end == 6 }),
            "expected Code for {text:?}, got {spans:?}"
        )
    }

    #[test]
    fn highlight_link() {
        let text = "[hi](note.md)";
        let spans = super::highlight(text);

        assert!(
            spans
                .iter()
                .any(|h| { h.kind == HighlightKind::Link && h.start == 0 && h.end == 13 }),
            "expected Link for {text:?}, got {spans:?}"
        );
    }

    #[test]
    fn style_for_heading_is_bold_lavender() {
        let style = super::style_for(HighlightKind::Heading);
        assert!(style.bold);
        assert_eq!(style.fg, Some(Colour::Rgb(190, 180, 255)));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn style_for_emphasis_is_muted_rose() {
        let style = super::style_for(HighlightKind::Emphasis);
        assert!(!style.bold);
        assert_eq!(style.fg, Some(Colour::Rgb(220, 160, 180)));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn style_for_bold_is_warm_white() {
        let style = super::style_for(HighlightKind::Bold);
        assert!(style.bold);
        assert_eq!(style.fg, Some(Colour::Rgb(240, 230, 210)));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn style_for_code_is_chip() {
        let style = super::style_for(HighlightKind::Code);
        assert!(!style.bold);
        assert_eq!(style.fg, Some(Colour::Rgb(180, 200, 180)));
        assert_eq!(style.bg, Some(Colour::Rgb(40, 44, 52)));
    }

    #[test]
    fn style_for_link_is_blue() {
        let style = super::style_for(HighlightKind::Link);
        assert!(!style.bold);
        assert_eq!(style.fg, Some(Colour::Rgb(100, 180, 255)));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn extract_note_links_unifies_wikilinks_and_markdown_links() {
        let text =
            "Start with [[Rust]] and check [Cargo](cargo.md). Skip [Web](https://rust-lang.org).";
        let links = extract_note_links(text);

        assert_eq!(links.len(), 2);

        assert_eq!(links[0].target, "Rust");
        assert_eq!(links[0].display, "Rust");
        assert_eq!(links[0].span, (11, 19));

        assert_eq!(links[1].target, "cargo.md");
        assert_eq!(links[1].display, "Cargo");
        assert_eq!(links[1].span, (30, 47));
    }
}
