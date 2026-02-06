use chrono::{DateTime, NaiveDate, Utc};

use super::ast::Query;
use crate::error::{Result, SnotError};

/// Parse a query string into a Query AST.
/// Auto-detects whether to use shorthand or SQL-style parsing.
pub fn parse(input: &str) -> Result<Query> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(Query::All);
    }

    if is_sql_syntax(input) {
        SqlParser::new(input).parse()
    } else {
        ShorthandParser::new(input).parse()
    }
}

/// Detect whether the input uses SQL-style syntax.
fn is_sql_syntax(input: &str) -> bool {
    let upper = input.to_uppercase();

    // Starts with SELECT
    if upper.starts_with("SELECT") {
        return true;
    }

    // Contains SQL operators as standalone words
    for keyword in &["CONTAINS", "LIKE", "BETWEEN"] {
        if upper.contains(keyword) {
            return true;
        }
    }

    // "column = 'value'" pattern (SQL-style equality with quoted value)
    let re = regex::Regex::new(r"(?i)\b(links_to|links_from)\s*=\s*'").unwrap();
    if re.is_match(input) {
        return true;
    }

    // neighbors('id', n) function syntax
    if regex::Regex::new(r"(?i)\bneighbors\s*\(")
        .unwrap()
        .is_match(input)
    {
        return true;
    }

    // "NOT column" pattern (standalone NOT before a column name, not as -prefix)
    if regex::Regex::new(
        r"(?i)\bNOT\s+(tags|content|title|alias|fuzzy|links_to|links_from|orphans|modified_date)\b",
    )
    .unwrap()
    .is_match(input)
    {
        return true;
    }

    // Check for " AND " or " OR " with SQL column names
    if upper.contains(" AND ") || upper.contains(" OR ") {
        for col in &[
            "TAGS",
            "CONTENT",
            "TITLE",
            "ALIAS",
            "LINKS_TO",
            "LINKS_FROM",
            "MODIFIED_DATE",
            "FUZZY",
            "NEIGHBORS",
            "ORPHANS",
        ] {
            if upper.contains(col) {
                return true;
            }
        }
    }

    false
}

// =============================================================================
// Shorthand Parser
// =============================================================================

/// Parses the shorthand query syntax:
///   tag:work #work title:text ~fuzzy content:text
///   links_to:id links_from:id orphans neighbors:id:2
///   Spaces = implicit AND, OR is explicit, - prefix = NOT
struct ShorthandParser {
    chars: Vec<char>,
    pos: usize,
}

impl ShorthandParser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Query> {
        let result = self.parse_or()?;
        self.skip_whitespace();
        if !self.at_end() {
            return Err(self.error(&format!("unexpected character '{}'", self.peek().unwrap())));
        }
        Ok(result)
    }

    fn parse_or(&mut self) -> Result<Query> {
        let mut left = self.parse_implicit_and()?;

        while self.try_consume_word("OR") {
            let right = self.parse_implicit_and()?;
            left = Query::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// Terms separated by spaces are implicit AND.
    fn parse_implicit_and(&mut self) -> Result<Query> {
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();
            if self.at_end() || self.check_word("OR") {
                break;
            }
            let right = self.parse_term()?;
            left = Query::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Query> {
        self.skip_whitespace();

        // NOT prefix: -
        if self.peek() == Some('-') {
            self.advance();
            let inner = self.parse_term()?;
            return Ok(Query::Not(Box::new(inner)));
        }

        // Fuzzy prefix: ~
        if self.peek() == Some('~') {
            self.advance();
            let value = self.consume_until_whitespace();
            if value.is_empty() {
                return Err(self.error("expected fuzzy search term after ~"));
            }
            return Ok(Query::Fuzzy(value));
        }

        // Tag shorthand: #tag
        if self.peek() == Some('#') {
            self.advance();
            let value = self.consume_until_whitespace();
            if value.is_empty() {
                return Err(self.error("expected tag name after #"));
            }
            return Ok(Query::Tag(value));
        }

        // Keyword: orphans
        if self.check_word("orphans") {
            self.consume_word("orphans");
            return Ok(Query::Orphans);
        }

        // prefix:value patterns
        let word = self.consume_until_whitespace();
        if word.is_empty() {
            return Err(self.error("expected query term"));
        }

        if let Some((prefix, value)) = word.split_once(':') {
            match prefix {
                "tag" => Ok(Query::Tag(value.to_string())),
                "title" => Ok(Query::Title(value.to_string())),
                "alias" => Ok(Query::Alias(value.to_string())),
                "content" => Ok(Query::Content(value.to_string())),
                "links_to" => Ok(Query::LinksTo(value.to_string())),
                "links_from" => Ok(Query::LinksFrom(value.to_string())),
                "neighbors" => {
                    // neighbors:note-id or neighbors:note-id:2
                    if let Some((note_id, depth_str)) = value.rsplit_once(':') {
                        let depth = depth_str
                            .parse::<usize>()
                            .map_err(|_| self.error(&format!("invalid depth: {}", depth_str)))?;
                        Ok(Query::Neighborhood(note_id.to_string(), depth))
                    } else {
                        Ok(Query::Neighborhood(value.to_string(), 1))
                    }
                }
                _ => Err(self.error(&format!("unknown query prefix: {}", prefix))),
            }
        } else {
            // Bare word — treat as tag search
            Ok(Query::Tag(word))
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) {
        if self.pos < self.chars.len() {
            self.pos += 1;
        }
    }

    fn consume_until_whitespace(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.chars.len() && !self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn check_word(&self, word: &str) -> bool {
        let remaining: String = self.chars[self.pos..].iter().collect();
        let upper = remaining.to_uppercase();
        let word_upper = word.to_uppercase();

        if upper.starts_with(&word_upper) {
            let end = self.pos + word.len();
            end >= self.chars.len() || self.chars[end].is_whitespace()
        } else {
            false
        }
    }

    fn try_consume_word(&mut self, word: &str) -> bool {
        self.skip_whitespace();
        if self.check_word(word) {
            self.pos += word.len();
            true
        } else {
            false
        }
    }

    fn consume_word(&mut self, word: &str) {
        self.pos += word.len();
    }

    fn error(&self, message: &str) -> SnotError {
        SnotError::ParseError {
            position: self.pos,
            message: message.to_string(),
        }
    }
}

// =============================================================================
// SQL Parser
// =============================================================================

/// Parses SQL-style queries using Vec<char> for O(n) access.
struct SqlParser {
    chars: Vec<char>,
    pos: usize,
}

impl SqlParser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Query> {
        self.skip_whitespace();

        // Optional: SELECT * FROM notes
        if self.try_keyword("SELECT") {
            self.skip_whitespace();
            self.expect_char('*')?;
            self.skip_whitespace();
            self.expect_keyword("FROM")?;
            self.skip_whitespace();
            self.expect_keyword("notes")?;
            self.skip_whitespace();
        }

        // Optional: WHERE
        self.try_keyword("WHERE");
        self.skip_whitespace();

        if self.at_end() {
            return Ok(Query::All);
        }

        let result = self.parse_or()?;
        self.skip_whitespace();
        if !self.at_end() {
            return Err(self.error(&format!("unexpected character '{}'", self.peek().unwrap())));
        }
        Ok(result)
    }

    fn parse_or(&mut self) -> Result<Query> {
        let mut left = self.parse_and()?;

        while self.try_keyword("OR") {
            let right = self.parse_and()?;
            left = Query::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Query> {
        let mut left = self.parse_unary()?;

        while self.try_keyword("AND") {
            let right = self.parse_unary()?;
            left = Query::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Query> {
        self.skip_whitespace();

        if self.try_keyword("NOT") {
            let inner = self.parse_unary()?;
            return Ok(Query::Not(Box::new(inner)));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Query> {
        self.skip_whitespace();

        // Parenthesized expression
        if self.peek() == Some('(') {
            self.advance();
            let query = self.parse_or()?;
            self.skip_whitespace();
            self.expect_char(')')?;
            return Ok(query);
        }

        // neighbors('note-id', depth) function syntax
        if self.check_keyword("neighbors") {
            self.consume_keyword("neighbors");
            self.skip_whitespace();

            // Check for function syntax: neighbors('id', depth)
            if self.peek() == Some('(') {
                self.advance();
                let note_id = self.consume_string_value()?;
                self.skip_whitespace();
                self.expect_char(',')?;
                self.skip_whitespace();
                let depth_str = self.consume_number()?;
                let depth = depth_str
                    .parse::<usize>()
                    .map_err(|_| self.error(&format!("invalid depth: {}", depth_str)))?;
                self.skip_whitespace();
                self.expect_char(')')?;
                return Ok(Query::Neighborhood(note_id, depth));
            }

            // Also accept: neighbors = 'note-id' (depth defaults to 1)
            self.skip_whitespace();
            self.expect_char('=')?;
            let note_id = self.consume_string_value()?;
            return Ok(Query::Neighborhood(note_id, 1));
        }

        // orphans keyword
        if self.check_keyword("orphans") {
            self.consume_keyword("orphans");
            return Ok(Query::Orphans);
        }

        // column OPERATOR value
        let column = self.consume_identifier()?;
        self.skip_whitespace();

        match column.to_lowercase().as_str() {
            "tags" => {
                self.expect_keyword("CONTAINS")?;
                let value = self.consume_string_value()?;
                Ok(Query::Tag(value))
            }
            "title" => {
                self.expect_keyword("LIKE")?;
                let value = self.consume_string_value()?;
                Ok(Query::Title(value.trim_matches('%').to_string()))
            }
            "alias" => {
                self.expect_keyword("LIKE")?;
                let value = self.consume_string_value()?;
                Ok(Query::Alias(value.trim_matches('%').to_string()))
            }
            "fuzzy" => {
                self.expect_keyword("LIKE")?;
                let value = self.consume_string_value()?;
                Ok(Query::Fuzzy(value.trim_matches('%').to_string()))
            }
            "content" => {
                self.expect_keyword("LIKE")?;
                let value = self.consume_string_value()?;
                Ok(Query::Content(value.trim_matches('%').to_string()))
            }
            "links_to" => {
                self.skip_whitespace();
                if self.peek() == Some('=') {
                    self.advance();
                } else {
                    self.expect_keyword("LIKE")?;
                }
                let value = self.consume_string_value()?;
                Ok(Query::LinksTo(value.trim_matches('%').to_string()))
            }
            "links_from" => {
                self.skip_whitespace();
                if self.peek() == Some('=') {
                    self.advance();
                } else {
                    self.expect_keyword("LIKE")?;
                }
                let value = self.consume_string_value()?;
                Ok(Query::LinksFrom(value.trim_matches('%').to_string()))
            }
            "modified_date" => {
                self.expect_keyword("BETWEEN")?;
                let start_str = self.consume_string_value()?;
                self.skip_whitespace();
                self.expect_keyword("AND")?;
                let end_str = self.consume_string_value()?;
                let start = parse_date(&start_str)
                    .map_err(|_| self.error(&format!("invalid date: {}", start_str)))?;
                let end = parse_date(&end_str)
                    .map_err(|_| self.error(&format!("invalid date: {}", end_str)))?;
                Ok(Query::DateRange(start, end))
            }
            _ => Err(self.error(&format!("unknown column: {}", column))),
        }
    }

    // --- Helper methods ---

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) {
        if self.pos < self.chars.len() {
            self.pos += 1;
        }
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        let remaining: String = self.chars[self.pos..].iter().collect();
        let upper = remaining.to_uppercase();
        let kw_upper = keyword.to_uppercase();

        if upper.starts_with(&kw_upper) {
            let end = self.pos + keyword.len();
            end >= self.chars.len() || !self.chars[end].is_alphanumeric()
        } else {
            false
        }
    }

    fn try_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        if self.check_keyword(keyword) {
            self.pos += keyword.len();
            true
        } else {
            false
        }
    }

    fn consume_keyword(&mut self, keyword: &str) {
        self.pos += keyword.len();
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<()> {
        self.skip_whitespace();
        if self.check_keyword(keyword) {
            self.pos += keyword.len();
            Ok(())
        } else {
            Err(self.error(&format!("expected keyword '{}'", keyword)))
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<()> {
        self.skip_whitespace();
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&format!("expected '{}', found {:?}", expected, self.peek())))
        }
    }

    fn consume_identifier(&mut self) -> Result<String> {
        self.skip_whitespace();
        let start = self.pos;

        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_')
        {
            self.pos += 1;
        }

        if start == self.pos {
            return Err(self.error("expected identifier"));
        }

        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn consume_string_value(&mut self) -> Result<String> {
        self.skip_whitespace();

        match self.peek() {
            Some(q @ '\'') | Some(q @ '"') => {
                self.advance();
                let start = self.pos;

                while self.pos < self.chars.len() && self.chars[self.pos] != q {
                    self.pos += 1;
                }

                if self.pos >= self.chars.len() {
                    return Err(self.error("unterminated string"));
                }

                let value: String = self.chars[start..self.pos].iter().collect();
                self.advance(); // consume closing quote
                Ok(value)
            }
            _ => Err(self.error("expected quoted string")),
        }
    }

    fn consume_number(&mut self) -> Result<String> {
        self.skip_whitespace();
        let start = self.pos;

        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        if start == self.pos {
            return Err(self.error("expected number"));
        }

        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn error(&self, message: &str) -> SnotError {
        SnotError::ParseError {
            position: self.pos,
            message: message.to_string(),
        }
    }
}

fn parse_date(s: &str) -> std::result::Result<DateTime<Utc>, ()> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Shorthand syntax tests ---

    #[test]
    fn test_shorthand_tag() {
        let q = parse("tag:work").unwrap();
        assert_eq!(q, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_shorthand_hash_tag() {
        let q = parse("#work").unwrap();
        assert_eq!(q, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_shorthand_title() {
        let q = parse("title:meeting").unwrap();
        assert_eq!(q, Query::Title("meeting".to_string()));
    }

    #[test]
    fn test_shorthand_fuzzy() {
        let q = parse("~meting").unwrap();
        assert_eq!(q, Query::Fuzzy("meting".to_string()));
    }

    #[test]
    fn test_shorthand_content() {
        let q = parse("content:quarterly").unwrap();
        assert_eq!(q, Query::Content("quarterly".to_string()));
    }

    #[test]
    fn test_shorthand_links_to() {
        let q = parse("links_to:note-id").unwrap();
        assert_eq!(q, Query::LinksTo("note-id".to_string()));
    }

    #[test]
    fn test_shorthand_links_from() {
        let q = parse("links_from:note-id").unwrap();
        assert_eq!(q, Query::LinksFrom("note-id".to_string()));
    }

    #[test]
    fn test_shorthand_orphans() {
        let q = parse("orphans").unwrap();
        assert_eq!(q, Query::Orphans);
    }

    #[test]
    fn test_shorthand_neighbors_with_depth() {
        let q = parse("neighbors:note-id:2").unwrap();
        assert_eq!(q, Query::Neighborhood("note-id".to_string(), 2));
    }

    #[test]
    fn test_shorthand_neighbors_default_depth() {
        let q = parse("neighbors:note-id").unwrap();
        assert_eq!(q, Query::Neighborhood("note-id".to_string(), 1));
    }

    #[test]
    fn test_shorthand_implicit_and() {
        let q = parse("tag:work title:meeting").unwrap();
        assert!(matches!(q, Query::And(_, _)));
    }

    #[test]
    fn test_shorthand_explicit_or() {
        let q = parse("tag:work OR tag:personal").unwrap();
        assert!(matches!(q, Query::Or(_, _)));
    }

    #[test]
    fn test_shorthand_not() {
        let q = parse("-tag:archived").unwrap();
        assert!(matches!(q, Query::Not(_)));
        if let Query::Not(inner) = q {
            assert_eq!(*inner, Query::Tag("archived".to_string()));
        }
    }

    #[test]
    fn test_shorthand_complex() {
        let q = parse("tag:work -tag:archived title:meeting").unwrap();
        // Should be: (tag:work AND (NOT tag:archived)) AND title:meeting
        assert!(matches!(q, Query::And(_, _)));
    }

    // --- SQL syntax tests ---

    #[test]
    fn test_sql_tag() {
        let q = parse("tags CONTAINS 'work'").unwrap();
        assert_eq!(q, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_sql_title() {
        let q = parse("title LIKE '%meeting%'").unwrap();
        assert_eq!(q, Query::Title("meeting".to_string()));
    }

    #[test]
    fn test_sql_alias() {
        let q = parse("alias LIKE '%standup%'").unwrap();
        assert_eq!(q, Query::Alias("standup".to_string()));
    }

    #[test]
    fn test_sql_fuzzy() {
        let q = parse("fuzzy LIKE 'meting'").unwrap();
        assert_eq!(q, Query::Fuzzy("meting".to_string()));
    }

    #[test]
    fn test_sql_content() {
        let q = parse("content LIKE '%quarterly%'").unwrap();
        assert_eq!(q, Query::Content("quarterly".to_string()));
    }

    #[test]
    fn test_sql_links_to() {
        let q = parse("links_to = 'project-plan'").unwrap();
        assert_eq!(q, Query::LinksTo("project-plan".to_string()));
    }

    #[test]
    fn test_sql_links_from() {
        let q = parse("links_from = 'project-plan'").unwrap();
        assert_eq!(q, Query::LinksFrom("project-plan".to_string()));
    }

    #[test]
    fn test_sql_orphans() {
        let q = parse("orphans AND tags CONTAINS 'draft'").unwrap();
        if let Query::And(left, right) = q {
            assert_eq!(*left, Query::Orphans);
            assert_eq!(*right, Query::Tag("draft".to_string()));
        } else {
            panic!("expected AND");
        }
    }

    #[test]
    fn test_sql_neighbors() {
        let q = parse("neighbors('project-plan', 2)").unwrap();
        assert_eq!(q, Query::Neighborhood("project-plan".to_string(), 2));
    }

    #[test]
    fn test_sql_and() {
        let q = parse("tags CONTAINS 'work' AND title LIKE '%meeting%'").unwrap();
        assert!(matches!(q, Query::And(_, _)));
    }

    #[test]
    fn test_sql_or() {
        let q = parse("tags CONTAINS 'work' OR tags CONTAINS 'personal'").unwrap();
        assert!(matches!(q, Query::Or(_, _)));
    }

    #[test]
    fn test_sql_not() {
        let q = parse("NOT tags CONTAINS 'archived'").unwrap();
        assert!(matches!(q, Query::Not(_)));
    }

    #[test]
    fn test_sql_parens() {
        let q = parse(
            "(tags CONTAINS 'work' OR tags CONTAINS 'personal') AND NOT tags CONTAINS 'archived'",
        )
        .unwrap();
        assert!(matches!(q, Query::And(_, _)));
    }

    #[test]
    fn test_sql_full_select() {
        let q = parse("SELECT * FROM notes WHERE tags CONTAINS 'work'").unwrap();
        assert_eq!(q, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_sql_date_range() {
        let q = parse("modified_date BETWEEN '2025-01-01' AND '2025-01-31'").unwrap();
        assert!(matches!(q, Query::DateRange(_, _)));
    }

    #[test]
    fn test_empty_query() {
        let q = parse("").unwrap();
        assert_eq!(q, Query::All);
    }

    #[test]
    fn test_whitespace_only() {
        let q = parse("   ").unwrap();
        assert_eq!(q, Query::All);
    }
}
