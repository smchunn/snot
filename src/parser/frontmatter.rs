use std::collections::HashMap;

use serde::Deserialize;

/// Structured frontmatter parsed via serde_yaml.
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct Frontmatter {
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub aliases: Vec<String>,

    #[serde(default)]
    pub id: Option<String>,

    /// Catch-all for user-defined fields we don't explicitly model.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Extract the raw YAML frontmatter string and the remaining content.
/// Returns (Some(yaml_str), rest) if frontmatter delimiters are found,
/// otherwise (None, original_content).
pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    if !content.starts_with("---") {
        return (None, content);
    }

    // Skip the opening "---" and any trailing characters on that line
    let after_open = match content[3..].find('\n') {
        Some(pos) => 3 + pos + 1,
        None => return (None, content),
    };

    // The opening line must be just "---" (possibly with trailing whitespace)
    let first_line = content[3..after_open].trim();
    if !first_line.is_empty() {
        return (None, content);
    }

    // Find closing "---"
    if let Some(close_pos) = content[after_open..].find("\n---") {
        let yaml_str = &content[after_open..after_open + close_pos];
        // Skip past the closing "---\n"
        let rest_start = after_open + close_pos + 4; // "\n---"
        let rest_start = rest_start
            + content[rest_start..]
                .find('\n')
                .map(|p| p + 1)
                .unwrap_or(content.len() - rest_start);
        (Some(yaml_str), &content[rest_start..])
    } else {
        (None, content)
    }
}

/// Parse a YAML string into a Frontmatter struct.
pub fn parse_frontmatter(yaml: &str) -> Result<Frontmatter, serde_yaml::Error> {
    serde_yaml::from_str(yaml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter_basic() {
        let content = "---\ntags: [work]\n---\n\n# Title";
        let (fm, rest) = split_frontmatter(content);
        assert_eq!(fm.unwrap(), "tags: [work]");
        assert!(rest.contains("# Title"));
    }

    #[test]
    fn test_split_frontmatter_none() {
        let content = "# No frontmatter here";
        let (fm, rest) = split_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(rest, content);
    }

    #[test]
    fn test_split_frontmatter_unclosed() {
        let content = "---\ntags: [work]\nNo closing delimiter";
        let (fm, _) = split_frontmatter(content);
        assert!(fm.is_none());
    }

    #[test]
    fn test_parse_frontmatter_array_tags() {
        let yaml = "tags: [work, meeting, project]";
        let fm = parse_frontmatter(yaml).unwrap();
        assert_eq!(fm.tags, vec!["work", "meeting", "project"]);
    }

    #[test]
    fn test_parse_frontmatter_list_tags() {
        let yaml = "tags:\n  - work\n  - meeting";
        let fm = parse_frontmatter(yaml).unwrap();
        assert_eq!(fm.tags, vec!["work", "meeting"]);
    }

    #[test]
    fn test_parse_frontmatter_aliases() {
        let yaml = "aliases:\n  - Daily Standup\n  - Standup Notes";
        let fm = parse_frontmatter(yaml).unwrap();
        assert_eq!(fm.aliases, vec!["Daily Standup", "Standup Notes"]);
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let yaml = "";
        let fm = parse_frontmatter(yaml).unwrap();
        assert!(fm.tags.is_empty());
        assert!(fm.aliases.is_empty());
        assert!(fm.id.is_none());
    }

    #[test]
    fn test_parse_frontmatter_extra_fields() {
        let yaml = "tags: [work]\ncustom_field: hello\ndate: 2025-01-01";
        let fm = parse_frontmatter(yaml).unwrap();
        assert_eq!(fm.tags, vec!["work"]);
        assert!(fm.extra.contains_key("custom_field"));
    }

    #[test]
    fn test_parse_frontmatter_quoted_values() {
        let yaml = "tags:\n  - \"multi word tag\"\n  - 'another tag'";
        let fm = parse_frontmatter(yaml).unwrap();
        assert_eq!(fm.tags, vec!["multi word tag", "another tag"]);
    }
}
