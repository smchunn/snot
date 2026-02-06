use crate::note::NoteId;
use chrono::{DateTime, Utc};

/// Query AST node types.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// Match notes with a specific tag.
    Tag(String),
    /// Match notes whose title contains the given substring.
    Title(String),
    /// Match notes with a matching alias.
    Alias(String),
    /// Fuzzy match on title + aliases (trigram similarity).
    Fuzzy(String),
    /// Content search (delegated to ripgrep/grep).
    Content(String),
    /// Notes that link TO the given note ID (i.e. backlinks of that note).
    LinksTo(NoteId),
    /// Notes that the given note links FROM (forward links).
    LinksFrom(NoteId),
    /// Notes within N hops of the given note.
    Neighborhood(NoteId, usize),
    /// Notes with no links (orphans).
    Orphans,
    /// Date range on modified_at.
    DateRange(DateTime<Utc>, DateTime<Utc>),
    /// Boolean AND.
    And(Box<Query>, Box<Query>),
    /// Boolean OR.
    Or(Box<Query>, Box<Query>),
    /// Boolean NOT.
    Not(Box<Query>),
    /// Match all notes.
    All,
}
