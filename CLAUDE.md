# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
Below this line, if you see !!, treat that as a command to update the section of markdown with whatever prompt follows !! and delete the prompt after

## Project Overview

SNOT (Simple Note Organization Tool) is a Rust-based note management system designed as a fast, lightweight alternative to Obsidian. Features a custom database with link graph, dual query language (shorthand + SQL-style), fuzzy search, and real-time file watching.

## Commands

### Build and Development

```bash
# Build the project
cargo build

# Build optimized release version
cargo build --release

# Run tests (94 unit tests)
cargo test

# Run a specific test
cargo test test_name

# Run the binary
cargo run -- <subcommand> [args]

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Common Operations

```bash
# Initialize a test vault
cargo run -- init ./test_vault

# Index notes
cargo run -- index ./test_vault

# Query notes (shorthand syntax)
cargo run -- query ./test_vault "tag:work"
cargo run -- query ./test_vault "#work title:meeting"
cargo run -- query ./test_vault "~meting"
cargo run -- query ./test_vault "tag:work OR tag:personal"
cargo run -- query ./test_vault "-tag:archived"

# Query notes (SQL-style syntax)
cargo run -- query ./test_vault "tags CONTAINS 'work' AND title LIKE '%meeting%'"
cargo run -- query ./test_vault "fuzzy LIKE 'meting'"
cargo run -- query ./test_vault "neighbors('project-plan', 2)"

# Create a note
cargo run -- create ./test_vault "Test Note"

# List notes
cargo run -- list ./test_vault

# List all tags
cargo run -- tags ./test_vault

# Graph operations
cargo run -- graph neighbors ./test_vault some-note --depth 2
cargo run -- graph orphans ./test_vault
cargo run -- graph path ./test_vault note-a note-b
cargo run -- graph stats ./test_vault

# Watch vault for changes
cargo run -- watch ./test_vault
```

## Architecture

### Module Structure

```
src/
  main.rs                 -- CLI (clap) + command dispatch
  lib.rs                  -- Public API re-exports
  error.rs                -- SnotError (thiserror) + Result type alias
  note.rs                 -- Note, NoteId, normalize_note_id, note_id_from_path
  vault.rs                -- Vault: coordinates db, parser, watcher
  db/
    mod.rs
    storage.rs            -- Database: HashMap + indexes, CRUD, persistence
    schema.rs             -- Binary header (magic "SNOT" + version), validation
    index.rs              -- TagIndex, DateIndex, PathIndex, AliasIndex
    graph.rs              -- LinkGraph: adjacency lists, BFS, shortest path,
                             connected components, orphan detection
  query/
    mod.rs
    ast.rs                -- Query AST (Tag, Title, Alias, Fuzzy, Content,
                             LinksTo, LinksFrom, Neighborhood, Orphan, DateRange)
    parser.rs             -- Dual parser: shorthand auto-detect + SQL recursive descent
    executor.rs           -- Set-based query execution, graph-aware
    fuzzy.rs              -- Trigram similarity matching
  parser/
    mod.rs
    markdown.rs           -- parse() -> ParsedNote (regex for inline tags/links)
    frontmatter.rs        -- Frontmatter struct with serde_yaml deserialization
  watcher/
    mod.rs
    handler.rs            -- VaultWatcher: owned lifecycle (Drop), debounced poll()
    scanner.rs            -- scan_vault(), calculate_checksum()
  commands/
    mod.rs
    init.rs, index.rs, query.rs, create.rs, update.rs, backlinks.rs,
    watch.rs, list.rs, tags.rs, graph.rs
```

### Key Design Decisions

1. **Vault as Central Coordinator**: All commands receive `Vault` instead of raw paths. Centralizes DB path computation, note ID generation, and file ingestion logic. No more duplication across commands.

2. **Links in Graph, Not Note**: The `Note` struct stores only metadata (title, tags, aliases, checksum). Links and backlinks are managed by `LinkGraph` with forward/reverse adjacency lists. Backlink maintenance is O(k) where k = links in the note.

3. **serde_yaml for Frontmatter**: Replaces hand-rolled YAML parsing. Handles quoted values, nested structures, multi-line strings correctly. The `Frontmatter` struct uses `#[serde(flatten)]` for user-defined fields.

4. **Dual Query Syntax**: Shorthand for quick CLI use (`tag:work`, `#work`, `~fuzzy`), SQL-style for complex queries (`tags CONTAINS 'work' AND title LIKE '%meeting%'`). Auto-detected by presence of SQL keywords. Both parse into the same AST.

5. **Schema-Versioned Database**: Binary format: 4 bytes magic (`SNOT`) + 4 bytes version (u32 LE) + bincode payload. Returns `SnotError::SchemaVersionMismatch` on version mismatch instead of silent corruption.

6. **Proper Watcher Lifecycle**: `VaultWatcher` owns the `RecommendedWatcher` as a struct field (dropped via `Drop`). `poll(debounce_duration)` coalesces rapid events for the same file and returns batches. Watch command saves once per batch.

7. **Error Strategy**: `thiserror` for structured library errors (`SnotError` in `error.rs`). `anyhow` only at CLI command level for context wrapping. Structured errors enable consumers to distinguish error types from JSON output.

8. **No Unnecessary Traits**: Concrete types tested with `tempfile::TempDir`. No `Storage` trait, no `Parser` trait.

### Data Flow

1. **Indexing**: `scan_vault()` -> for each `.md` file -> `calculate_checksum()` -> skip if unchanged -> `markdown::parse()` -> `Vault::ingest_file()` -> `Database::insert/update(note, links)` -> indexes + graph updated -> `vault.save()`
2. **Querying**: CLI input -> `query::parse()` (auto-detects syntax) -> `Query` AST -> `QueryExecutor::execute()` -> set operations on indexes/graph -> JSON output
3. **Watching**: `VaultWatcher::poll(debounce)` -> batch of `FileEvent`s -> `Vault::ingest_file()`/`delete_file()` per event -> single `vault.save()` per batch

## Important Implementation Details

### Note ID Generation (centralized in `note.rs`)

Notes are identified by their relative path within the vault, converted to kebab-case:

- `vault/work/meeting-notes.md` -> ID: `work-meeting-notes`
- Path separators become hyphens
- Extension removed
- Normalized to lowercase via `normalize_note_id()`

### Link Resolution

Wiki-links `[[note-name]]` are resolved using normalized IDs:

- `[[Work Meeting]]` -> links to note with ID `work-meeting`
- Display text supported: `[[note-name|Custom Text]]`
- Links stored in `LinkGraph`, not on `Note` struct
- Backlinks maintained automatically via reverse adjacency list

### Query Syntax Auto-Detection

The parser checks for SQL keywords (`CONTAINS`, `LIKE`, `BETWEEN`, `NOT column`, `links_to = '...'`, `neighbors(...)`) to decide which parser to use. If none found, shorthand parser is used.

### Database Persistence

Load: `fs::read()` -> `schema::read_header()` (validate magic+version) -> `bincode::deserialize()` -> `Database`
Save: `schema::write_header()` -> `bincode::serialize()` -> `fs::write()`

## Testing Strategy

### Unit Tests (94 tests)

Each module has inline tests:

```bash
cargo test parser::markdown::tests       # Markdown parsing tests
cargo test parser::frontmatter::tests    # serde_yaml frontmatter tests
cargo test db::storage::tests            # Database CRUD tests
cargo test db::graph::tests              # Link graph traversal tests
cargo test db::schema::tests             # Schema versioning tests
cargo test query::parser::tests          # Both shorthand + SQL parser tests
cargo test query::executor::tests        # Query execution tests
cargo test query::fuzzy::tests           # Trigram similarity tests
cargo test note::tests                   # Note ID generation tests
cargo test vault::tests                  # Vault integration tests
cargo test watcher::scanner::tests       # File scanning tests
```

### Integration Testing

Manual testing workflow:

1. Create test vault: `cargo run -- init ./test_vault`
2. Add sample markdown files with frontmatter, tags, wiki-links
3. Index: `cargo run -- index ./test_vault`
4. Query with both syntaxes: `cargo run -- query ./test_vault "tag:work"`
5. Test graph: `cargo run -- graph stats ./test_vault`
6. Verify all output is valid JSON

## Common Development Tasks

### Adding a New Query Operator

1. Add variant to `Query` enum in `src/query/ast.rs`
2. Add shorthand syntax in `ShorthandParser::parse_term()` in `src/query/parser.rs`
3. Add SQL syntax in `SqlParser::parse_primary()` in `src/query/parser.rs`
4. Update `is_sql_syntax()` detection if needed
5. Implement execution logic in `QueryExecutor::execute_ids()` in `src/query/executor.rs`
6. Add tests for both syntaxes

### Modifying Database Schema

When changing `Note` struct or indexes:

1. Update `Note` definition in `src/note.rs`
2. Update indexes in `src/db/index.rs` if new indexed field
3. Modify `Database::insert()`, `update()`, `delete()` in `src/db/storage.rs`
4. **Important**: Bump `VERSION` in `src/db/schema.rs` - old databases will return `SchemaVersionMismatch`
5. Update `Vault::ingest_file()` if note creation changes
6. Update JSON serialization in command outputs

### Extending Markdown Parser

1. Add extraction logic to `src/parser/markdown.rs`
2. Update `ParsedNote` struct with new fields
3. Update `Vault::ingest_file()` to map new fields
4. Add tests for new parsing logic

## Code Style and Conventions

- Use `SnotError` (thiserror) for library-level errors in `src/error.rs`
- Use `anyhow::Result` only at CLI command level for context wrapping
- Prefer `&Path` over `&PathBuf` in function signatures
- All CLI output for consumption should be JSON via `serde_json`
- Human-readable output uses `println!` with descriptive messages
- No unnecessary traits - concrete types only

## Known Limitations and Future Work

1. **Conflict Resolution**: No handling of concurrent modifications
2. **Graph Visualization**: Data structures support it, but no export command
3. **Template Support**: Mentioned in README but not implemented
4. **Daily Notes**: Common feature not yet implemented
5. **Database Migration**: Schema version checked but no migration path (requires reindex)
