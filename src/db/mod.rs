pub mod storage;
pub mod query;

pub use storage::{Database, Note, NoteId};
pub use query::{Query, QueryExecutor};
