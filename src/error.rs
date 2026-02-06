use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum SnotError {
    #[error("vault not found: {0}")]
    VaultNotFound(PathBuf),

    #[error("vault already initialized: {0}")]
    VaultAlreadyInitialized(PathBuf),

    #[error("database not found: {0}")]
    DatabaseNotFound(PathBuf),

    #[error("schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },

    #[error("invalid database: bad magic bytes")]
    InvalidMagic,

    #[error("note not found: {0}")]
    NoteNotFound(String),

    #[error("note already exists: {0}")]
    NoteAlreadyExists(String),

    #[error("file not in vault: {path}")]
    FileNotInVault { path: PathBuf },

    #[error("parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, SnotError>;
