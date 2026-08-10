pub struct Wikilinks<'a> {
    pub target: &'a str,
    pub heading: Option<&'a str>,
    pub display: &'a str,
    pub span: (usize, usize),
}

pub fn parse_wikilinks(text: &str) -> Vec<Wikilinks<'_>> {
    let mut links = Vec::new();
    let mut start_search = 0;

    while let Some(open) = text[start_search..].find("[[") {
        let open_idx = start_search + open;
        let content_start = open_idx + 2;

        if let Some(close) = text[content_start..].find("]]") {
            let close_idx = content_start + close;
            let raw_content = &text[content_start..close_idx];

            let (link_part, display) = match raw_content.find('|') {
                Some(pipe_idx) => {
                    let target_part = raw_content[..pipe_idx].trim();
                    let display = raw_content[pipe_idx + 1..].trim();
                    (target_part, display)
                }
                None => {
                    let target_part = raw_content.trim();
                    (target_part, target_part)
                }
            };

            let (target, heading) = match link_part.find('#') {
                Some(hash_idx) => {
                    let target_str = link_part[..hash_idx].trim();
                    let heading_str = link_part[hash_idx + 1..].trim();
                    (target_str, Some(heading_str))
                }
                None => (link_part, None),
            };

            links.push(Wikilinks {
                target,
                heading,
                display,
                span: (open_idx, close_idx + 2),
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

        // 4. [[Note#Heading|custom]]
        assert_eq!(links[3].target, "Note");
        assert_eq!(links[3].heading, Some("Heading"));
        assert_eq!(links[3].display, "custom");

        // 5. [#Local Heading]
        assert_eq!(links[4].target, "");
        assert_eq!(links[4].heading, Some("Local Heading"));
    }
}
