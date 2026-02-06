use std::collections::HashSet;

use regex::Regex;

use super::frontmatter;

/// Result of parsing a markdown note file.
#[derive(Debug, Clone)]
pub struct ParsedNote {
    pub title: String,
    pub aliases: Vec<String>,
    pub tags: HashSet<String>,
    pub links: HashSet<String>,
}

/// Parse a markdown note, extracting metadata from both frontmatter and inline content.
pub fn parse(content: &str) -> ParsedNote {
    let (fm_yaml, body) = frontmatter::split_frontmatter(content);

    let fm = fm_yaml
        .and_then(|yaml| frontmatter::parse_frontmatter(yaml).ok())
        .unwrap_or_default();

    let mut tags = extract_inline_tags(body);
    for tag in &fm.tags {
        tags.insert(tag.clone());
    }

    let links = extract_links(body);
    let title = extract_title(body).unwrap_or_else(|| "Untitled".to_string());
    let aliases = fm.aliases;

    ParsedNote {
        title,
        aliases,
        tags,
        links,
    }
}

/// Extract inline tags (#tag) from markdown content.
/// Does not match headings (##, ###, etc.).
fn extract_inline_tags(content: &str) -> HashSet<String> {
    let re = Regex::new(r"(?:^|[^#\w])#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Extract wiki-links [[note-name]] or [[note-name|display text]].
/// Returns normalized note IDs.
fn extract_links(content: &str) -> HashSet<String> {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| {
            cap.get(1)
                .map(|m| crate::note::normalize_note_id(m.as_str()))
        })
        .collect()
}

/// Extract the first H1 heading from the content.
fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inline_tags() {
        let content = "This is a note with #tag1 and #tag2";
        let tags = extract_inline_tags(content);
        assert!(tags.contains("tag1"));
        assert!(tags.contains("tag2"));
    }

    #[test]
    fn test_tags_ignore_headings() {
        let content = "## Heading\n### Another\nBut #real-tag here";
        let tags = extract_inline_tags(content);
        assert!(tags.contains("real-tag"));
        assert!(!tags.contains("Heading"));
        assert!(!tags.contains("#"));
    }

    #[test]
    fn test_extract_links() {
        let content = "Link to [[Another Note]] and [[yet-another|display]]";
        let links = extract_links(content);
        assert!(links.contains("another-note"));
        assert!(links.contains("yet-another"));
    }

    #[test]
    fn test_extract_title() {
        let content = "# My Note Title\n\nContent here";
        assert_eq!(extract_title(content).unwrap(), "My Note Title");
    }

    #[test]
    fn test_extract_title_no_h1() {
        let content = "## Only H2\n\nContent here";
        assert!(extract_title(content).is_none());
    }

    #[test]
    fn test_parse_full_note() {
        let content = "---\ntags: [work, meeting]\naliases:\n  - Daily Standup\n---\n\n# Meeting Notes\n\nDiscussing [[project-plan]] with #team\n";
        let parsed = parse(content);
        assert_eq!(parsed.title, "Meeting Notes");
        assert!(parsed.tags.contains("work"));
        assert!(parsed.tags.contains("meeting"));
        assert!(parsed.tags.contains("team"));
        assert!(parsed.links.contains("project-plan"));
        assert_eq!(parsed.aliases, vec!["Daily Standup"]);
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Simple Note\n\nJust some content with #tag1\n";
        let parsed = parse(content);
        assert_eq!(parsed.title, "Simple Note");
        assert!(parsed.tags.contains("tag1"));
        assert!(parsed.aliases.is_empty());
    }

    #[test]
    fn test_parse_no_title() {
        let content = "Just content, no heading";
        let parsed = parse(content);
        assert_eq!(parsed.title, "Untitled");
    }
}
