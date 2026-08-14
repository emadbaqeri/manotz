use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MarkdownLink<'a> {
    pub dest: &'a str,
    pub heading: Option<&'a str>,
    pub display: &'a str,
    pub span: (usize, usize),
}

impl<'a> MarkdownLink<'a> {
    pub fn is_external(&self) -> bool {
        self.dest.starts_with("https://")
            || self.dest.starts_with("http://")
            || self.dest.starts_with("mailto:")
    }
}

pub fn parse_markdown_links<'a>(text: &'a str) -> Vec<MarkdownLink<'a>> {
    let mut links = Vec::new();
    let mut current_link: Option<(usize, &'a str)> = None;
    let mut display_text: &'a str = "";

    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                let dest_slice = match dest_url {
                    pulldown_cmark::CowStr::Borrowed(s) => s,
                    _ => "",
                };
                current_link = Some((range.start, dest_slice));
            }
            Event::Text(t) => {
                if current_link.is_some() {
                    display_text = match t {
                        pulldown_cmark::CowStr::Borrowed(s) => s,
                        _ => "",
                    };
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some((start, raw_dest)) = current_link.take() {
                    let (dest, heading) = match raw_dest.find('#') {
                        Some(hash_idx) => (&raw_dest[..hash_idx], Some(&raw_dest[hash_idx + 1..])),
                        None => (raw_dest, None),
                    };
                    links.push(MarkdownLink {
                        dest,
                        heading,
                        display: display_text,
                        span: (start, range.end),
                    });
                    display_text = "";
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
        assert_eq!(links[0].heading, None);
        assert_eq!(links[0].display, "Rust Guide");
        assert_eq!(links[0].span, (14, 35));
    }

    #[test]
    fn parse_markdown_link_with_heading_and_local_anchor() {
        let text = "See [Installation](guide.md#install) and [Jump](#intro).";
        let links = parse_markdown_links(text);
        assert_eq!(links.len(), 2);

        assert_eq!(links[0].dest, "guide.md");
        assert_eq!(links[0].heading, Some("install"));
        assert_eq!(links[0].display, "Installation");

        assert_eq!(links[1].dest, "");
        assert_eq!(links[1].heading, Some("intro"));
        assert_eq!(links[1].display, "Jump");
    }

    #[test]
    fn markdown_link_distinguishes_external_and_internal() {
        let text = "Internal: [Doc](doc.md). External: [Web](https://example.com)
  and [Mail](mailto:hi@test.com).";
        let links = parse_markdown_links(text);

        assert_eq!(links.len(), 3);
        assert!(!links[0].is_external());
        assert!(links[1].is_external());
        assert!(links[2].is_external());
    }
}
