#[derive(Debug, PartialEq, Default, Clone)]
pub struct Frontmatter {
    pub aliases: Vec<String>,
}

/// Helper: Extracts text between leading `---` and ending `---` if document
/// starts with frontmatter. Handles UTF-8 BOM (`\u{feff}`) and standalone `---` fence lines.
fn extract_frontmatter_block(text: &str) -> Option<&str> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let rest = &trimmed[3..];
    let first_line_end = rest.find('\n')?;
    let first_line = rest[..first_line_end].trim();
    if !first_line.is_empty() {
        return None;
    }

    let body = &rest[first_line_end + 1..];
    for line in body.lines() {
        if line.trim().starts_with("---") {
            let offset = line.as_ptr() as usize - body.as_ptr() as usize;
            return Some(body[..offset].trim());
        }
    }

    None
}

/// Helper: Strips trailing YAML comments (`# comment`) outside single/double quotes.
fn strip_yaml_comment(s: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut comment_start = None;

    for (i, ch) in s.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double => {
                comment_start = Some(i);
                break;
            }
            _ => {}
        }
    }

    if let Some(idx) = comment_start {
        &s[..idx]
    } else {
        s
    }
}

pub fn parse_frontmatter(text: &str) -> Option<Frontmatter> {
    let block = extract_frontmatter_block(text)?;
    let mut aliases = Vec::new();
    let mut in_aliases_block = false;

    for line in block.lines() {
        let line_without_comment = strip_yaml_comment(line);
        let trimmed = line_without_comment.trim();

        if let Some(rest) = trimmed.strip_prefix("aliases:") {
            let rest = rest.trim();
            if rest.starts_with('[') && rest.ends_with(']') {
                let inner = &rest[1..rest.len() - 1];
                for item in inner.split(',') {
                    let cleaned = item.trim().trim_matches('"').trim_matches('\'');
                    if !cleaned.is_empty() {
                        aliases.push(cleaned.to_string());
                    }
                }
                in_aliases_block = false;
            } else if rest.is_empty() {
                in_aliases_block = true;
            }
        } else if in_aliases_block {
            if let Some(item) = trimmed.strip_prefix("- ") {
                let cleaned = item.trim().trim_matches('"').trim_matches('\'');
                if !cleaned.is_empty() {
                    aliases.push(cleaned.to_string());
                }
            } else if !trimmed.is_empty() {
                in_aliases_block = false;
            }
        }
    }

    Some(Frontmatter { aliases })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frontmatter_extracts_inline_aliases() {
        let content = "---\naliases: [TODO List, Tasks]\n---# Notes";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.aliases, vec!["TODO List", "Tasks"]);
    }

    #[test]
    fn parse_frontmatter_extracts_yaml_list_aliases() {
        let content = "---\naliases:\n - TODO List\n - Tasks\n---\n&Notes";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.aliases, vec!["TODO List", "Tasks"]);
    }

    #[test]
    fn parse_frontmatter_handles_inline_aliases_with_trailing_comments() {
        let content = "---\naliases: [TODO List, Tasks] # inline comment\n---\n# Notes";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.aliases, vec!["TODO List", "Tasks"]);
    }

    #[test]
    fn parse_frontmatter_handles_yaml_list_aliases_with_comments() {
        let content = "---\naliases:\n  - TODO List # urgent item\n  - \"Project #9\" # secondary\n---\n# Notes";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.aliases, vec!["TODO List", "Project #9"]);
    }

    #[test]
    fn parse_frontmatter_handles_utf8_bom() {
        let content = "\u{feff}---\naliases: [TODO List]\n---\n# Notes";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.aliases, vec!["TODO List"]);
    }

    #[test]
    fn parse_frontmatter_ignores_non_fence_dashes() {
        let content = "---\naliases: [TODO List]\n# line with --- inside\n---\n# Notes";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.aliases, vec!["TODO List"]);
    }
}
