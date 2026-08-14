use std::borrow::Cow;

use pulldown_cmark::{CowStr, Event, Parser, Tag, TagEnd};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MarkdownLink<'a> {
    pub dest: Cow<'a, str>,
    pub heading: Option<Cow<'a, str>>,
    pub display: Cow<'a, str>,
    pub span: (usize, usize),
}

impl<'a> MarkdownLink<'a> {
    pub fn is_external(&self) -> bool {
        let dest = self.dest.as_ref();
        if let Some(colon_idx) = dest.find(':') {
            let scheme = &dest[..colon_idx];
            scheme.eq_ignore_ascii_case("http")
                || scheme.eq_ignore_ascii_case("https")
                || scheme.eq_ignore_ascii_case("mailto")
                || scheme.eq_ignore_ascii_case("ftp")
        } else {
            false
        }
    }
}

fn cow_str_to_cow<'a>(cow: CowStr<'a>) -> Cow<'a, str> {
    match cow {
        CowStr::Borrowed(s) => Cow::Borrowed(s),
        CowStr::Boxed(s) => Cow::Owned(s.into_string()),
        CowStr::Inlined(s) => Cow::Owned(s.to_string()),
    }
}

pub fn parse_markdown_links<'a>(text: &'a str) -> Vec<MarkdownLink<'a>> {
    let mut links = Vec::new();
    let mut current_link: Option<(usize, Cow<'a, str>)> = None;
    let mut display_acc = String::new();

    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                current_link = Some((range.start, cow_str_to_cow(dest_url)));
                display_acc.clear();
            }
            Event::Text(t) => {
                if current_link.is_some() {
                    display_acc.push_str(t.as_ref());
                }
            }
            Event::Code(c) => {
                if current_link.is_some() {
                    display_acc.push_str(c.as_ref());
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some((start, raw_dest)) = current_link.take() {
                    let (dest, heading) = match raw_dest {
                        Cow::Borrowed(s) => match s.find('#') {
                            Some(hash_idx) => (
                                Cow::Borrowed(&s[..hash_idx]),
                                Some(Cow::Borrowed(&s[hash_idx + 1..])),
                            ),
                            None => (Cow::Borrowed(s), None),
                        },
                        Cow::Owned(s) => match s.find('#') {
                            Some(hash_idx) => (
                                Cow::Owned(s[..hash_idx].to_string()),
                                Some(Cow::Owned(s[hash_idx + 1..].to_string())),
                            ),
                            None => (Cow::Owned(s), None),
                        },
                    };

                    let display = if display_acc.is_empty() {
                        Cow::Borrowed("")
                    } else {
                        Cow::Owned(std::mem::take(&mut display_acc))
                    };

                    links.push(MarkdownLink {
                        dest,
                        heading,
                        display,
                        span: (start, range.end),
                    });
                }
            }
            _ => {}
        }
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_markdown_link_simple() {
        let text = "Check out the [Rust Guide](rust.md) for details.";
        let links = parse_markdown_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].dest, "rust.md");
        assert_eq!(links[0].heading.as_deref(), None);
        assert_eq!(links[0].display, "Rust Guide");
        assert_eq!(links[0].span, (14, 35));
    }

    #[test]
    fn parse_markdown_link_with_heading_and_local_anchor() {
        let text = "See [Installation](guide.md#install) and [Jump](#intro).";
        let links = parse_markdown_links(text);
        assert_eq!(links.len(), 2);

        assert_eq!(links[0].dest, "guide.md");
        assert_eq!(links[0].heading.as_deref(), Some("install"));
        assert_eq!(links[0].display, "Installation");

        assert_eq!(links[1].dest, "");
        assert_eq!(links[1].heading.as_deref(), Some("intro"));
        assert_eq!(links[1].display, "Jump");
    }

    #[test]
    fn markdown_link_distinguishes_external_and_internal() {
        let text = "Internal: [Doc](doc.md). External: [Web](https://example.com) and [Upper](HTTPS://EXAMPLE.COM) and [Mail](MAILTO:hi@test.com).";
        let links = parse_markdown_links(text);

        assert_eq!(links.len(), 4);
        assert!(!links[0].is_external());
        assert!(links[1].is_external());
        assert!(links[2].is_external());
        assert!(links[3].is_external());
    }

    #[test]
    fn markdown_link_handles_multi_fragment_label_and_entities() {
        let text = "Check [A & B with `code` and *styled*](foo&amp;bar.md#heading).";
        let links = parse_markdown_links(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].dest, "foo&bar.md");
        assert_eq!(links[0].heading.as_deref(), Some("heading"));
        assert_eq!(links[0].display, "A & B with code and styled");
        assert!(!links[0].is_external());
    }
}
