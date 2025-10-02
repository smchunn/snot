mod db;
mod parser;
mod watcher;
mod picker;
mod commands;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "snot")]
#[command(about = "A Rust-based note management system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault
    Init {
        /// Path to the vault directory
        vault_path: PathBuf,
    },
    /// Index all notes in the vault
    Index {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Force reindex of all files
        #[arg(short, long)]
        force: bool,
    },
    /// Query notes using the query language
    Query {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Query string (e.g., "tag:work AND contains:meeting")
        query: String,
    },
    /// Create a new note
    Create {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Note name
        name: String,
    },
    /// Get backlinks for a note
    Backlinks {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Path to the note file
        file: PathBuf,
    },
    /// Watch vault for changes
    Watch {
        /// Path to the vault directory
        vault_path: PathBuf,
    },
    /// List all notes (for FZF integration)
    List {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Optional query to filter notes
        #[arg(short, long)]
        query: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { vault_path } => {
            commands::init_vault(&vault_path)?;
        }
        Commands::Index { vault_path, force } => {
            commands::index_vault(&vault_path, force)?;
        }
        Commands::Query { vault_path, query } => {
            commands::query_notes(&vault_path, &query)?;
        }
        Commands::Create { vault_path, name } => {
            commands::create_note(&vault_path, &name)?;
        }
        Commands::Backlinks { vault_path, file } => {
            commands::get_backlinks(&vault_path, &file)?;
        }
        Commands::Watch { vault_path } => {
            commands::watch_vault(&vault_path)?;
        }
        Commands::List { vault_path, query } => {
            list_notes(&vault_path, query.as_deref())?;
        }
    }

    Ok(())
}

fn list_notes(vault_path: &PathBuf, query: Option<&str>) -> Result<()> {
    use db::{Database, Query, QueryExecutor};

    let db_path = vault_path.join(".snot/db.bin");
    let db = Database::with_path(db_path)?;

    let notes = if let Some(query_str) = query {
        let parsed_query = Query::parse(query_str)?;
        let executor = QueryExecutor::new(&db);
        executor.execute(&parsed_query)
    } else {
        db.get_all()
    };

    // Output one path per line for FZF
    for note in notes {
        println!("{}", note.file_path.display());
    }

    Ok(())
}
