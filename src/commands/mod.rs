pub mod init;
pub mod index;
pub mod query;
pub mod create;
pub mod backlinks;
pub mod watch;

pub use self::init::init_vault;
pub use self::index::index_vault;
pub use self::query::query_notes;
pub use self::create::create_note;
pub use self::backlinks::get_backlinks;
pub use self::watch::watch_vault;
