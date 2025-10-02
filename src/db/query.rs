use super::storage::{Database, Note};
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashSet;
use std::process::Command;
use std::path::PathBuf;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Tag(String),
    Contains(String),
    LinkedTo(String),
    DateRange(DateTime<Utc>, DateTime<Utc>),
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
    Not(Box<Query>),
    All,
}

impl Query {
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(Query::All);
        }

        // SQL parser for queries
        SqlParser::new(input).parse()
    }
}

struct SqlParser {
    input: String,
    pos: usize,
}

impl SqlParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Query> {
        self.skip_whitespace();

        // Optional: SELECT * FROM notes
        if self.consume_keyword("SELECT") {
            self.skip_whitespace();
            self.consume_char('*')?;
            self.skip_whitespace();
            self.consume_keyword_strict("FROM")?;
            self.skip_whitespace();
            self.consume_keyword_strict("notes")?;
            self.skip_whitespace();
        }

        // Optional: WHERE
        if self.consume_keyword("WHERE") {
            self.skip_whitespace();
        }

        // If we have consumed SELECT/WHERE or neither, parse the condition
        if self.at_end() {
            return Ok(Query::All);
        }

        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Query> {
        let mut left = self.parse_and()?;

        while self.consume_keyword("OR") {
            let right = self.parse_and()?;
            left = Query::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Query> {
        let mut left = self.parse_primary()?;

        while self.consume_keyword("AND") {
            let right = self.parse_primary()?;
            left = Query::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Query> {
        self.skip_whitespace();

        if self.consume_keyword("NOT") {
            let query = self.parse_primary()?;
            return Ok(Query::Not(Box::new(query)));
        }

        if self.peek() == Some('(') {
            self.advance();
            let query = self.parse_or()?;
            self.skip_whitespace();
            self.consume_char(')')?;
            return Ok(query);
        }

        // Parse SQL condition: column OPERATOR value
        let column = self.consume_identifier()?;
        self.skip_whitespace();

        match column.to_lowercase().as_str() {
            "tags" => {
                self.consume_keyword_strict("CONTAINS")?;
                let value = self.consume_string_value()?;
                Ok(Query::Tag(value))
            }
            "content" => {
                self.consume_keyword_strict("LIKE")?;
                let value = self.consume_string_value()?;
                // Remove % wildcards if present
                let value = value.trim_matches('%').to_string();
                Ok(Query::Contains(value))
            }
            "links_to" => {
                self.skip_whitespace();
                if self.peek() == Some('=') {
                    self.advance();
                } else {
                    self.consume_keyword_strict("LIKE")?;
                }
                let value = self.consume_string_value()?;
                let value = value.trim_matches('%').to_string();
                Ok(Query::LinkedTo(value))
            }
            "modified_date" => {
                self.consume_keyword_strict("BETWEEN")?;
                let start_str = self.consume_string_value()?;
                self.skip_whitespace();
                self.consume_keyword_strict("AND")?;
                let end_str = self.consume_string_value()?;

                let start = self.parse_date(&start_str)?;
                let end = self.parse_date(&end_str)?;
                Ok(Query::DateRange(start, end))
            }
            _ => Err(anyhow!("Unknown column: {}", column)),
        }
    }

    fn parse_date(&self, date_str: &str) -> Result<DateTime<Utc>> {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| anyhow!("Invalid date format: {}", date_str))?;
        Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap().is_whitespace() {
            self.pos += 1;
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
        }
    }

    fn consume_char(&mut self, expected: char) -> Result<()> {
        self.skip_whitespace();
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(anyhow!("Expected '{}', found '{:?}'", expected, self.peek()))
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let remaining = &self.input[self.pos..];
        let keyword_upper = keyword.to_uppercase();

        if remaining.to_uppercase().starts_with(&keyword_upper) {
            let end_pos = self.pos + keyword.len();
            if end_pos >= self.input.len() ||
               !self.input.chars().nth(end_pos).unwrap().is_alphanumeric() {
                self.pos = end_pos;
                return true;
            }
        }
        false
    }

    fn consume_keyword_strict(&mut self, keyword: &str) -> Result<()> {
        if self.consume_keyword(keyword) {
            Ok(())
        } else {
            Err(anyhow!("Expected keyword '{}'", keyword))
        }
    }

    fn consume_identifier(&mut self) -> Result<String> {
        self.skip_whitespace();
        let start = self.pos;

        while self.pos < self.input.len() {
            let ch = self.input.chars().nth(self.pos).unwrap();
            if ch.is_alphanumeric() || ch == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if start == self.pos {
            return Err(anyhow!("Expected identifier"));
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn consume_string_value(&mut self) -> Result<String> {
        self.skip_whitespace();

        if self.peek() == Some('\'') || self.peek() == Some('"') {
            let quote = self.peek().unwrap();
            self.advance();
            let start = self.pos;

            while self.pos < self.input.len() {
                let ch = self.input.chars().nth(self.pos).unwrap();
                if ch == quote {
                    let result = self.input[start..self.pos].to_string();
                    self.advance();
                    return Ok(result);
                }
                self.pos += 1;
            }

            Err(anyhow!("Unterminated string"))
        } else {
            Err(anyhow!("Expected quoted string"))
        }
    }
}

pub struct QueryExecutor<'a> {
    db: &'a Database,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn execute(&self, query: &Query) -> Vec<&'a Note> {
        match query {
            Query::All => self.db.get_all(),
            Query::Tag(tag) => self.db.get_by_tag(tag),
            Query::Contains(text) => self.search_content(text),
            Query::LinkedTo(note_id) => self.db.get_backlinks(note_id),
            Query::DateRange(start, end) => self.db.get_in_date_range(*start, *end),
            Query::And(left, right) => {
                let left_results = self.execute(left);
                let right_results = self.execute(right);
                self.intersect(left_results, right_results)
            }
            Query::Or(left, right) => {
                let left_results = self.execute(left);
                let right_results = self.execute(right);
                self.union(left_results, right_results)
            }
            Query::Not(inner) => {
                let inner_results = self.execute(inner);
                let all_results = self.db.get_all();
                self.difference(all_results, inner_results)
            }
        }
    }

    fn search_content(&self, text: &str) -> Vec<&'a Note> {
        let file_paths = self.db.get_all_file_paths();
        if file_paths.is_empty() {
            return Vec::new();
        }

        // Try ripgrep first, then grep
        let matching_paths = if let Ok(paths) = self.search_with_ripgrep(text, &file_paths) {
            paths
        } else if let Ok(paths) = self.search_with_grep(text, &file_paths) {
            paths
        } else {
            // Fallback: search in titles/aliases only
            return self.db.get_all().into_iter()
                .filter(|note| {
                    note.title.to_lowercase().contains(&text.to_lowercase()) ||
                    note.aliases.iter().any(|a| a.to_lowercase().contains(&text.to_lowercase()))
                })
                .collect();
        };

        self.db.get_notes_by_paths(&matching_paths)
    }

    fn search_with_ripgrep(&self, text: &str, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let output = Command::new("rg")
            .arg("-i") // case insensitive
            .arg("-l") // files with matches only
            .arg("--")
            .arg(text)
            .args(files)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("ripgrep failed"));
        }

        let paths = String::from_utf8(output.stdout)?
            .lines()
            .map(PathBuf::from)
            .collect();

        Ok(paths)
    }

    fn search_with_grep(&self, text: &str, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let output = Command::new("grep")
            .arg("-i") // case insensitive
            .arg("-l") // files with matches only
            .arg("--")
            .arg(text)
            .args(files)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("grep failed"));
        }

        let paths = String::from_utf8(output.stdout)?
            .lines()
            .map(PathBuf::from)
            .collect();

        Ok(paths)
    }

    fn intersect(&self, left: Vec<&'a Note>, right: Vec<&'a Note>) -> Vec<&'a Note> {
        let right_ids: HashSet<_> = right.iter().map(|n| &n.id).collect();
        left.into_iter()
            .filter(|n| right_ids.contains(&n.id))
            .collect()
    }

    fn union(&self, mut left: Vec<&'a Note>, right: Vec<&'a Note>) -> Vec<&'a Note> {
        let left_ids: HashSet<_> = left.iter().map(|n| &n.id).collect();
        for note in right {
            if !left_ids.contains(&note.id) {
                left.push(note);
            }
        }
        left
    }

    fn difference(&self, all: Vec<&'a Note>, exclude: Vec<&'a Note>) -> Vec<&'a Note> {
        let exclude_ids: HashSet<_> = exclude.iter().map(|n| &n.id).collect();
        all.into_iter()
            .filter(|n| !exclude_ids.contains(&n.id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_query() {
        let query = Query::parse("tags CONTAINS 'work'").unwrap();
        assert_eq!(query, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_parse_contains_query() {
        let query = Query::parse("content LIKE '%meeting%'").unwrap();
        assert_eq!(query, Query::Contains("meeting".to_string()));
    }

    #[test]
    fn test_parse_and_query() {
        let query = Query::parse("tags CONTAINS 'work' AND content LIKE '%meeting%'").unwrap();
        assert!(matches!(query, Query::And(_, _)));
    }

    #[test]
    fn test_parse_or_query() {
        let query = Query::parse("tags CONTAINS 'work' OR tags CONTAINS 'personal'").unwrap();
        assert!(matches!(query, Query::Or(_, _)));
    }

    #[test]
    fn test_parse_full_sql() {
        let query = Query::parse("SELECT * FROM notes WHERE tags CONTAINS 'work'").unwrap();
        assert_eq!(query, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_parse_with_where() {
        let query = Query::parse("WHERE content LIKE '%important%'").unwrap();
        assert_eq!(query, Query::Contains("important".to_string()));
    }

    #[test]
    fn test_parse_date_range() {
        let query = Query::parse("modified_date BETWEEN '2025-01-01' AND '2025-01-31'").unwrap();
        assert!(matches!(query, Query::DateRange(_, _)));
    }

    #[test]
    fn test_parse_links_to() {
        let query = Query::parse("links_to = 'project-plan'").unwrap();
        assert_eq!(query, Query::LinkedTo("project-plan".to_string()));
    }
}
