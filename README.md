# SNOT - Simple Note Organization Tool

A fast, lightweight Rust-based note management system designed as an alternative to Obsidian. Features a custom database with link graph, dual query language (shorthand + SQL-style), fuzzy search, and real-time file watching.

## Features

- **Custom Database**: Fast, in-memory note storage with multiple indexes and binary serialization
- **Link Graph**: Track connections between notes with backlinks, neighbors, orphan detection, and shortest path
- **Dual Query Language**: Shorthand syntax for quick CLI use, SQL-style for complex queries
- **Fuzzy Search**: Trigram-based similarity matching for typo-tolerant searches
- **SHA-256 Checksum Caching**: Incremental updates — only reindex changed files
- **File Watcher**: Real-time updates when notes change
- **Smart Note Creation**: Automatic kebab-case naming with dates and YAML frontmatter
- **Markdown Parsing**: Extract tags, wiki-links, aliases, and frontmatter

## Documentation

- **[Query Syntax Guide](docs/query-syntax.md)** — Complete reference for search queries
- **[Documentation Index](docs/)** — All documentation and guides

## Installation

```bash
# Build from source
cargo build --release

# Install binary
cargo install --path .
```

## Quick Start

### 1. Initialize a Vault

```bash
snot init /path/to/vault
```

### 2. Index Your Notes

```bash
snot index /path/to/vault

# Force reindex all files
snot index /path/to/vault --force
```

### 3. Create Notes

```bash
snot create /path/to/vault "My New Note"
# Creates: my-new-note-2025-10-02.md
# With YAML frontmatter:
#   ---
#   id: my-new-note-2025-10-02
#   aliases:
#     - My New Note
#   tags: []
#   ---
```

### 4. Query Notes

```bash
# Shorthand syntax (quick CLI use)
snot query /path/to/vault "tag:work"
snot query /path/to/vault "#work title:meeting"
snot query /path/to/vault "~meting"                    # fuzzy search
snot query /path/to/vault "tag:work OR tag:personal"
snot query /path/to/vault "-tag:archived"               # negation

# SQL-style syntax (complex queries)
snot query /path/to/vault "tags CONTAINS 'work' AND title LIKE '%meeting%'"
snot query /path/to/vault "links_to = 'project-plan'"
snot query /path/to/vault "modified_date BETWEEN '2025-01-01' AND '2025-03-01'"
snot query /path/to/vault "neighbors('project-plan', 2)"
```

## Usage

### CLI Commands

#### Query Notes

```bash
# Search by tag
snot query /path/to/vault "tag:work"
snot query /path/to/vault "tags CONTAINS 'work'"

# Search by content
snot query /path/to/vault "content LIKE '%meeting%'"

# Fuzzy search (typo-tolerant)
snot query /path/to/vault "~meting"

# Find notes linking to another note
snot query /path/to/vault "links_to = 'project-plan'"

# Date range queries
snot query /path/to/vault "modified_date BETWEEN '2025-01-01' AND '2025-03-01'"

# Graph-aware queries
snot query /path/to/vault "neighbors('project-plan', 2)"

# Combine queries
snot query /path/to/vault "tags CONTAINS 'work' AND content LIKE '%meeting%'"
snot query /path/to/vault "tags CONTAINS 'work' OR tags CONTAINS 'personal'"
snot query /path/to/vault "tags CONTAINS 'work' AND NOT content LIKE '%done%'"
```

#### Get Backlinks

```bash
snot backlinks /path/to/vault /path/to/note.md
```

#### List Notes

```bash
# List all notes
snot list /path/to/vault

# List with query filter
snot list /path/to/vault --query "tag:work"

# Use with FZF
snot list /path/to/vault | fzf --preview 'cat {}'
```

#### List Tags

```bash
snot tags /path/to/vault
```

#### Graph Operations

```bash
# Find neighbors of a note (with depth)
snot graph neighbors /path/to/vault some-note --depth 2

# Find orphaned notes (no links in or out)
snot graph orphans /path/to/vault

# Find shortest path between two notes
snot graph path /path/to/vault note-a note-b

# Show graph statistics
snot graph stats /path/to/vault
```

#### Watch Vault

```bash
# Start file watcher daemon
snot watch /path/to/vault
```

### Query Language

SNOT supports two query syntaxes that parse into the same AST. The parser auto-detects which to use based on the presence of SQL keywords.

#### Shorthand Syntax

For quick CLI use:

```bash
tag:work              # search by tag
#work                 # shorthand tag
title:meeting         # search by title
~meting               # fuzzy search
-tag:archived         # negation
tag:work OR tag:personal  # boolean OR
```

#### SQL-Style Syntax

For complex queries:

```sql
-- Basic queries
tags CONTAINS 'work'
content LIKE '%meeting%'
links_to = 'project-plan'
modified_date BETWEEN '2025-01-01' AND '2025-01-31'

-- Boolean logic
tags CONTAINS 'work' AND content LIKE '%deadline%'
tags CONTAINS 'meeting' OR tags CONTAINS 'standup'
tags CONTAINS 'work' AND NOT tags CONTAINS 'archived'

-- Grouping with parentheses
(tags CONTAINS 'work' OR tags CONTAINS 'personal') AND NOT tags CONTAINS 'archived'

-- Graph queries
neighbors('project-plan', 2)

-- Optional: Full SQL syntax
SELECT * FROM notes WHERE tags CONTAINS 'urgent'
```

**See [Query Syntax Guide](docs/query-syntax.md) for complete documentation with examples.**

### Note Format

Notes are created with YAML frontmatter:

```markdown
---
id: note-name-2025-10-02
aliases:
  - Note Name
tags: []
---

# Note Title

Content here...
```

Notes support:

1. **Wiki Links**: `[[note-name]]` or `[[note-name|display text]]`
2. **Tags**: `#tag` inline or in frontmatter
3. **Frontmatter**: id, aliases, tags, and user-defined fields via `serde_yaml`

## Architecture

### Directory Structure

```
snot/
├── src/
│   ├── main.rs                 # CLI (clap) + command dispatch
│   ├── lib.rs                  # Public API re-exports
│   ├── error.rs                # SnotError (thiserror) + Result type alias
│   ├── note.rs                 # Note, NoteId, normalize_note_id
│   ├── vault.rs                # Vault: coordinates db, parser, watcher
│   ├── db/
│   │   ├── storage.rs          # Database: HashMap + indexes, CRUD, persistence
│   │   ├── schema.rs           # Binary header (magic "SNOT" + version)
│   │   ├── index.rs            # TagIndex, DateIndex, PathIndex, AliasIndex
│   │   └── graph.rs            # LinkGraph: adjacency lists, BFS, shortest path
│   ├── query/
│   │   ├── ast.rs              # Query AST (dual syntax)
│   │   ├── parser.rs           # Shorthand auto-detect + SQL recursive descent
│   │   ├── executor.rs         # Set-based query execution, graph-aware
│   │   └── fuzzy.rs            # Trigram similarity matching
│   ├── parser/
│   │   ├── markdown.rs         # parse() -> ParsedNote (tags, links, frontmatter)
│   │   └── frontmatter.rs      # Frontmatter struct with serde_yaml
│   ├── watcher/
│   │   ├── handler.rs          # VaultWatcher: owned lifecycle, debounced poll()
│   │   └── scanner.rs          # scan_vault(), calculate_checksum()
│   └── commands/
│       ├── init.rs, index.rs, query.rs, create.rs, update.rs
│       ├── backlinks.rs, watch.rs, list.rs, tags.rs
│       └── graph.rs
└── docs/
    ├── README.md               # Documentation index
    └── query-syntax.md         # Query language reference
```

### Database Design

- **In-memory HashMap** for O(1) note lookups
- **Tag index** for fast tag-based queries
- **Date index (BTree)** for efficient range queries
- **Path index** for file-to-note mapping
- **Alias index** for alternative note names
- **Link graph** with forward/reverse adjacency lists for backlink tracking
- **Binary serialization** (bincode) with schema versioning for fast load/save

### Performance

- Incremental updates using SHA-256 checksums
- Parallel file processing with Rayon
- Optimized for vaults with 1000+ notes
- Query responses typically under 50ms

## Editor Integration

The **[snot.nvim](https://github.com/yourusername/snot.nvim)** plugin provides Neovim integration with commands, file picking, and auto-completion. See its repository for setup instructions.

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Run

```bash
cargo run -- help
```

### Format

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```

## Dependencies

- `clap` — CLI argument parsing
- `serde` + `serde_json` + `serde_yaml` — Serialization (JSON output, YAML frontmatter)
- `bincode` — Binary database serialization
- `notify` — File watching
- `sha2` — Checksum calculation
- `chrono` — Date handling
- `rayon` — Parallel processing
- `regex` — Pattern matching
- `walkdir` — Directory traversal
- `thiserror` + `anyhow` — Error handling

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
