use std::collections::HashSet;
use regex::Regex;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ParsedNote {
    pub title: String,
    pub aliases: Vec<String>,
    pub tags: HashSet<String>,
    pub links: HashSet<String>,
    pub frontmatter: Option<String>,
}

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn parse(content: &str) -> Result<ParsedNote> {
        let (frontmatter, content_without_fm) = Self::extract_frontmatter(content);

        let mut tags = Self::extract_tags(&content_without_fm);
        let mut aliases = Vec::new();

        // Extract tags and aliases from frontmatter if present
        if let Some(ref fm) = frontmatter {
            tags.extend(Self::extract_frontmatter_tags(fm));
            aliases = Self::extract_frontmatter_aliases(fm);
        }

        let links = Self::extract_links(&content_without_fm);
        let title = Self::extract_title(&content_without_fm);

        Ok(ParsedNote {
            title,
            aliases,
            tags,
            links,
            frontmatter,
        })
    }

    fn extract_frontmatter(content: &str) -> (Option<String>, String) {
        if !content.starts_with("---") {
            return (None, content.to_string());
        }

        if let Some(end_pos) = content[3..].find("---") {
            let frontmatter = content[3..end_pos + 3].trim().to_string();
            let rest = content[end_pos + 6..].to_string();
            (Some(frontmatter), rest)
        } else {
            (None, content.to_string())
        }
    }

    fn extract_frontmatter_tags(frontmatter: &str) -> HashSet<String> {
        let mut tags = HashSet::new();
        let mut in_tags_section = false;

        // Look for tags in YAML frontmatter
        // Support both:
        // tags: [tag1, tag2, tag3]
        // tags:
        //   - tag1
        //   - tag2

        for line in frontmatter.lines() {
            let line = line.trim();

            // Array format: tags: [tag1, tag2]
            if line.starts_with("tags:") {
                let tags_str = line.strip_prefix("tags:").unwrap().trim();
                if tags_str.starts_with('[') && tags_str.ends_with(']') {
                    let tags_content = &tags_str[1..tags_str.len()-1];
                    for tag in tags_content.split(',') {
                        let tag = tag.trim().trim_matches('"').trim_matches('\'');
                        if !tag.is_empty() {
                            tags.insert(tag.to_string());
                        }
                    }
                }
                in_tags_section = true;
                continue;
            }

            // List format: - tag (only if we're in tags section)
            if in_tags_section && line.starts_with("- ") {
                let tag = line.strip_prefix("- ").unwrap().trim();
                let tag = tag.trim_matches('"').trim_matches('\'');
                if !tag.is_empty() {
                    tags.insert(tag.to_string());
                }
            } else if in_tags_section && !line.is_empty() && !line.starts_with("- ") {
                // Exit tags section if we hit a non-list item
                in_tags_section = false;
            }
        }

        tags
    }

    fn extract_frontmatter_aliases(frontmatter: &str) -> Vec<String> {
        let mut aliases = Vec::new();
        let mut in_aliases_section = false;

        // Look for aliases in YAML frontmatter
        // Support both:
        // aliases: [alias1, alias2]
        // aliases:
        //   - alias1
        //   - alias2

        for line in frontmatter.lines() {
            let line = line.trim();

            // Array format: aliases: [alias1, alias2]
            if line.starts_with("aliases:") {
                let aliases_str = line.strip_prefix("aliases:").unwrap().trim();
                if aliases_str.starts_with('[') && aliases_str.ends_with(']') {
                    let aliases_content = &aliases_str[1..aliases_str.len()-1];
                    for alias in aliases_content.split(',') {
                        let alias = alias.trim().trim_matches('"').trim_matches('\'');
                        if !alias.is_empty() {
                            aliases.push(alias.to_string());
                        }
                    }
                }
                in_aliases_section = true;
                continue;
            }

            // List format: - alias (only if we're in aliases section)
            if in_aliases_section && line.starts_with("- ") {
                let alias = line.strip_prefix("- ").unwrap().trim();
                let alias = alias.trim_matches('"').trim_matches('\'');
                if !alias.is_empty() {
                    aliases.push(alias.to_string());
                }
            } else if in_aliases_section && !line.is_empty() && !line.starts_with("- ") {
                // Exit aliases section if we hit a non-list item
                in_aliases_section = false;
            }
        }

        aliases
    }

    fn extract_tags(content: &str) -> HashSet<String> {
        let mut tags = HashSet::new();

        // Match #tag pattern (but not ##heading)
        let tag_regex = Regex::new(r"(?:^|[^#\w])#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();

        for cap in tag_regex.captures_iter(content) {
            if let Some(tag) = cap.get(1) {
                tags.insert(tag.as_str().to_string());
            }
        }

        tags
    }

    fn extract_links(content: &str) -> HashSet<String> {
        let mut links = HashSet::new();

        // Match wiki-style links [[note-name]] or [[note-name|display text]]
        let wiki_link_regex = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();

        for cap in wiki_link_regex.captures_iter(content) {
            if let Some(link) = cap.get(1) {
                // Convert link to ID format (lowercase, with dashes)
                let link_id = Self::normalize_note_id(link.as_str());
                links.insert(link_id);
            }
        }

        links
    }

    fn extract_title(content: &str) -> String {
        // Try to find first H1 heading
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("# ") {
                return line.strip_prefix("# ").unwrap().trim().to_string();
            }
        }

        // If no H1, return "Untitled"
        "Untitled".to_string()
    }

    pub fn normalize_note_id(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags() {
        let content = "This is a note with #tag1 and #tag2";
        let tags = MarkdownParser::extract_tags(content);
        assert!(tags.contains("tag1"));
        assert!(tags.contains("tag2"));
    }

    #[test]
    fn test_extract_links() {
        let content = "Link to [[another-note]] and [[yet-another|display]]";
        let links = MarkdownParser::extract_links(content);
        assert!(links.contains("another-note"));
        assert!(links.contains("yet-another"));
    }

    #[test]
    fn test_extract_title() {
        let content = "# My Note Title\n\nContent here";
        let title = MarkdownParser::extract_title(content);
        assert_eq!(title, "My Note Title");
    }

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\ntags: [work, meeting]\n---\n\n# Note Title";
        let (fm, rest) = MarkdownParser::extract_frontmatter(content);
        assert!(fm.is_some());
        assert!(rest.contains("# Note Title"));
    }

    #[test]
    fn test_frontmatter_tags() {
        let frontmatter = "tags: [work, meeting, project]";
        let tags = MarkdownParser::extract_frontmatter_tags(frontmatter);
        assert!(tags.contains("work"));
        assert!(tags.contains("meeting"));
        assert!(tags.contains("project"));
    }

    #[test]
    fn test_normalize_note_id() {
        assert_eq!(
            MarkdownParser::normalize_note_id("My New Note"),
            "my-new-note"
        );
    }
}
