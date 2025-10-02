use super::storage::{Database, Note};
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashSet;
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

        // Simple parser for our query language
        QueryParser::new(input).parse()
    }
}

struct QueryParser {
    input: String,
    pos: usize,
}

impl QueryParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Query> {
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
            if self.peek() != Some(')') {
                return Err(anyhow!("Expected closing parenthesis"));
            }
            self.advance();
            return Ok(query);
        }

        // Parse key:value format
        let key = self.consume_identifier()?;

        if self.peek() != Some(':') {
            return Err(anyhow!("Expected ':' after key '{}'", key));
        }
        self.advance();

        let value = self.consume_value()?;

        match key.as_str() {
            "tag" => Ok(Query::Tag(value)),
            "contains" => Ok(Query::Contains(value)),
            "linked-to" => Ok(Query::LinkedTo(value)),
            "date" => self.parse_date_range(&value),
            _ => Err(anyhow!("Unknown query key: {}", key)),
        }
    }

    fn parse_date_range(&self, value: &str) -> Result<Query> {
        if let Some((start_str, end_str)) = value.split_once("..") {
            let start = self.parse_date(start_str)?;
            let end = self.parse_date(end_str)?;
            Ok(Query::DateRange(start, end))
        } else {
            Err(anyhow!("Date range must be in format 'YYYY-MM-DD..YYYY-MM-DD'"))
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

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let remaining = &self.input[self.pos..];

        if remaining.starts_with(keyword) {
            let end_pos = self.pos + keyword.len();
            // Make sure it's followed by whitespace or end of string
            if end_pos >= self.input.len() || self.input.chars().nth(end_pos).unwrap().is_whitespace() {
                self.pos = end_pos;
                return true;
            }
        }
        false
    }

    fn consume_identifier(&mut self) -> Result<String> {
        self.skip_whitespace();
        let start = self.pos;

        while self.pos < self.input.len() {
            let ch = self.input.chars().nth(self.pos).unwrap();
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
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

    fn consume_value(&mut self) -> Result<String> {
        self.skip_whitespace();

        if self.peek() == Some('"') {
            self.consume_quoted_string()
        } else {
            self.consume_unquoted_value()
        }
    }

    fn consume_quoted_string(&mut self) -> Result<String> {
        self.advance(); // Skip opening quote
        let start = self.pos;

        while self.pos < self.input.len() {
            let ch = self.input.chars().nth(self.pos).unwrap();
            if ch == '"' {
                let result = self.input[start..self.pos].to_string();
                self.advance(); // Skip closing quote
                return Ok(result);
            }
            self.pos += 1;
        }

        Err(anyhow!("Unterminated quoted string"))
    }

    fn consume_unquoted_value(&mut self) -> Result<String> {
        let start = self.pos;

        while self.pos < self.input.len() {
            let ch = self.input.chars().nth(self.pos).unwrap();
            if ch.is_whitespace() || ch == ')' {
                break;
            }
            self.pos += 1;
        }

        if start == self.pos {
            return Err(anyhow!("Expected value"));
        }

        Ok(self.input[start..self.pos].to_string())
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
            Query::Contains(text) => self.db.search_content(text),
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
        let query = Query::parse("tag:work").unwrap();
        assert_eq!(query, Query::Tag("work".to_string()));
    }

    #[test]
    fn test_parse_contains_query() {
        let query = Query::parse("contains:meeting").unwrap();
        assert_eq!(query, Query::Contains("meeting".to_string()));
    }

    #[test]
    fn test_parse_and_query() {
        let query = Query::parse("tag:work AND contains:meeting").unwrap();
        assert!(matches!(query, Query::And(_, _)));
    }

    #[test]
    fn test_parse_or_query() {
        let query = Query::parse("tag:work OR tag:personal").unwrap();
        assert!(matches!(query, Query::Or(_, _)));
    }

    #[test]
    fn test_parse_quoted_value() {
        let query = Query::parse("contains:\"important meeting\"").unwrap();
        assert_eq!(query, Query::Contains("important meeting".to_string()));
    }
}
