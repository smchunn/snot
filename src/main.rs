mod commands;
mod db;
mod error;
mod note;
mod parser;
mod query;
mod vault;
mod watcher;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
        /// Query string (shorthand or SQL-style)
        query: String,
    },
    /// Create a new note
    Create {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Note name
        name: String,
    },
    /// Update a single note's metadata in the cache
    Update {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Path to the note file
        file: PathBuf,
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
    /// List all tags
    Tags {
        /// Path to the vault directory
        vault_path: PathBuf,
    },
    /// Graph operations
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Find notes within N hops of a note
    Neighbors {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Note ID to find neighbors of
        note: String,
        /// Maximum distance (hops)
        #[arg(short, long, default_value = "1")]
        depth: usize,
    },
    /// Find notes with no links
    Orphans {
        /// Path to the vault directory
        vault_path: PathBuf,
    },
    /// Find shortest path between two notes
    Path {
        /// Path to the vault directory
        vault_path: PathBuf,
        /// Starting note ID
        from: String,
        /// Ending note ID
        to: String,
    },
    /// Show graph statistics
    Stats {
        /// Path to the vault directory
        vault_path: PathBuf,
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
        Commands::Update { vault_path, file } => {
            commands::update_note(&vault_path, &file)?;
        }
        Commands::Backlinks { vault_path, file } => {
            commands::get_backlinks(&vault_path, &file)?;
        }
        Commands::Watch { vault_path } => {
            commands::watch_vault(&vault_path)?;
        }
        Commands::List { vault_path, query } => {
            commands::list_notes(&vault_path, query.as_deref())?;
        }
        Commands::Tags { vault_path } => {
            commands::list_tags(&vault_path)?;
        }
        Commands::Graph { command } => match command {
            GraphCommands::Neighbors {
                vault_path,
                note,
                depth,
            } => {
                commands::graph_neighbors(&vault_path, &note, depth)?;
            }
            GraphCommands::Orphans { vault_path } => {
                commands::graph_orphans(&vault_path)?;
            }
            GraphCommands::Path {
                vault_path,
                from,
                to,
            } => {
                commands::graph_path(&vault_path, &from, &to)?;
            }
            GraphCommands::Stats { vault_path } => {
                commands::graph_stats(&vault_path)?;
            }
        },
    }

    Ok(())
}
