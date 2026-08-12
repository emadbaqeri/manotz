#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Wikilink<'a> {
    pub target: &'a str,
    pub heading: Option<&'a str>,
    pub display: &'a str,
    pub span: (usize, usize),
}

pub fn parse_wikilinks(text: &str) -> Vec<Wikilink<'_>> {
    let mut links = Vec::new();
    let mut start_search = 0;

    while let Some(open) = text[start_search..].find("[[") {
        let open_idx = start_search + open;
        let content_start = open_idx + 2;

        if let Some(close) = text[content_start..].find("]]") {
            let close_idx = content_start + close;
            let mut raw_content = &text[content_start..close_idx];
            let mut real_open_idx = open_idx;

            if let Some(inner) = raw_content.rfind("[[") {
                real_open_idx = content_start + inner;
                raw_content = &text[real_open_idx + 2..close_idx];
            }

            let (link_part, explicit_display) = match raw_content.find('|') {
                Some(pipe_idx) => {
                    let target_part = raw_content[..pipe_idx].trim();
                    let display_part = raw_content[pipe_idx + 1..].trim();
                    (target_part, Some(display_part))
                }
                None => (raw_content.trim(), None),
            };

            let (target, heading) = match link_part.find('#') {
                Some(hash_idx) => {
                    let target_str = link_part[..hash_idx].trim();
                    let heading_str = link_part[hash_idx + 1..].trim();
                    (target_str, Some(heading_str))
                }
                None => (link_part, None),
            };

            let display = match explicit_display {
                Some(disp) => disp,
                None => {
                    if target.is_empty() {
                        heading.unwrap_or("")
                    } else {
                        link_part
                    }
                }
            };

            links.push(Wikilink {
                target,
                heading,
                display,
                span: (real_open_idx, close_idx + 2),
            });
            start_search = close_idx + 2;
        } else {
            break;
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wikilinks_extracts_all_patterns() {
        let text = "See [[Note]], [[Note|alias]], [[Note#Heading]], and [[Note#Heading|custom]]. Also [[#Local Heading]].";

        let links = parse_wikilinks(text);
        assert_eq!(links.len(), 5);

        // 1. [[Note]]
        assert_eq!(links[0].target, "Note");
        assert_eq!(links[0].heading, None);
        assert_eq!(links[0].display, "Note");

        // 2. [[Note|alias]]
        assert_eq!(links[1].target, "Note");
        assert_eq!(links[1].heading, None);
        assert_eq!(links[1].display, "alias");

        // 3. [[Note#Heading]]
        assert_eq!(links[2].target, "Note");
        assert_eq!(links[2].heading, Some("Heading"));
        assert_eq!(links[2].display, "Note#Heading");

        // 4. [[Note#Heading|custom]]
        assert_eq!(links[3].target, "Note");
        assert_eq!(links[3].heading, Some("Heading"));
        assert_eq!(links[3].display, "custom");

        // 5. [[#Local Heading]]
        assert_eq!(links[4].target, "");
        assert_eq!(links[4].heading, Some("Local Heading"));
        assert_eq!(links[4].display, "Local Heading");
    }

    #[test]
    fn parse_wikilinks_continues_after_unclosed_opener() {
        let text = "Unclosed [[opener here but later [[Valid Note]] appears";
        let links = parse_wikilinks(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Valid Note");
        assert_eq!(links[0].display, "Valid Note");
    }

    #[test]
    fn parse_wikilinks_unclosed_opener_at_eof() {
        let text = "Valid [[Link]] followed by trailing [[unclosed opener";
        let links = parse_wikilinks(text);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Link");
        assert_eq!(links[0].display, "Link");
    }
}
