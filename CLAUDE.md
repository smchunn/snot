# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
Below this line, if you see !!, treat that as a command to update the section of markdown with whatever prompt follows !! and delete the prompt after

## Project Overview

SNOT (Simple Note Organization Tool) is a Rust-based note management system with Neovim integration. It serves as a fast, lightweight alternative to Obsidian, featuring a custom database, query language, and real-time file watching.

## Commands

### Build and Development

```bash
# Build the project
cargo build

# Build optimized release version
cargo build --release

# Run tests
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

# Query notes
cargo run -- query ./test_vault "tag:work"

# Create a note
cargo run -- create ./test_vault "Test Note"

# List notes
cargo run -- list ./test_vault

# Watch vault for changes
cargo run -- watch ./test_vault
```

## Architecture

### High-Level Design

The project follows a modular architecture with clear separation of concerns:

1. **Database Layer (`src/db/`)**: Custom in-memory database with metadata-only storage
   - `storage.rs`: Core data structures (Note, Database) with CRUD operations
   - `query.rs`: SQL-style query parser and executor using recursive descent parsing
   - Stores only frontmatter (title, tags, aliases, links) - NOT full content
   - Uses HashMap for O(1) lookups, BTree for date ranges, maintains bidirectional link tracking
   - Content searches delegated to ripgrep/grep for better performance

2. **Parser Layer (`src/parser/`)**: Markdown frontmatter extraction
   - `links.rs`: Extracts frontmatter, wiki-links `[[note]]`, tags `#tag`, titles, and aliases
   - Handles both inline tags and YAML frontmatter tags/aliases
   - Normalizes note IDs to kebab-case format
   - Does NOT store full file content - only metadata

3. **Watcher Layer (`src/watcher/`)**: File system monitoring with caching
   - `checksum.rs`: SHA-256 based change detection for incremental updates
   - Uses `notify` crate for cross-platform file watching
   - Only reindexes files that have actually changed

4. **Commands Layer (`src/commands/`)**: CLI command implementations
   - Each command is in its own module (init, index, query, create, update, backlinks, watch)
   - `update.rs`: Updates single file metadata (used by Neovim on save)
   - All commands output JSON for easy consumption by the Neovim plugin

5. **Neovim Plugin** (separate repository: [snot.nvim](https://github.com/yourusername/snot.nvim))
   - Maintained separately for better separation of concerns
   - Communicates with Rust CLI via jobstart
   - Provides commands, completion, and picker integration
   - See snot.nvim repository for plugin-specific development

### Key Design Decisions

1. **Custom Database vs SQLite**: Built from scratch for learning and performance
   - Allows fine-grained control over indexing strategy
   - No SQL overhead for simple operations
   - Binary serialization with bincode for fast persistence

2. **Frontmatter-Only Storage**: Store metadata, not content
   - Only title, tags, aliases, links, and checksums stored in database
   - Dramatically reduces memory usage for large vaults
   - Content searches use ripgrep/grep instead of in-memory scan
   - Falls back to title/alias search if external tools unavailable

3. **Query Language**: SQL-style syntax instead of custom DSL
   - Supports complex boolean logic (AND, OR, NOT)
   - SQL-like operators: `tags CONTAINS 'work'`, `content LIKE '%text%'`
   - Optional full SQL syntax: `SELECT * FROM notes WHERE ...`
   - Recursive descent parser for clean implementation

4. **Incremental Indexing**: SHA-256 checksums prevent redundant work
   - Critical for large vaults (1000+ notes)
   - Checksum stored with each note in database
   - File watcher triggers selective reindexing
   - Auto-update on save via Neovim BufWritePost autocmd

5. **Bidirectional Links**: Automatically maintain backlinks
   - When note A links to note B, note B's backlinks include A
   - Updated atomically during insert/update/delete operations
   - No separate backlink indexing required

6. **Neovim Integration**: CLI-first design with editor as consumer
   - All functionality available via CLI
   - Neovim plugin is thin wrapper using jobstart
   - JSON output for easy parsing
   - Picker abstraction supports fzf-lua, telescope, or vim.ui.select

## Important Implementation Details

### Note ID Generation

Notes are identified by their relative path within the vault, converted to kebab-case:

- `vault/work/meeting-notes.md` → ID: `work-meeting-notes`
- Path separators become hyphens
- Extension removed
- Normalized to lowercase

### Link Resolution

Wiki-links `[[note-name]]` are resolved using normalized IDs:

- `[[Work Meeting]]` → links to note with ID `work-meeting`
- Display text supported: `[[note-name|Custom Text]]`
- Links are bidirectional - backlinks automatically tracked

### Database Update Strategy

The `update()` method has a specific pattern to avoid borrow checker issues:

1. Clone necessary data from old note (tags, links, paths, dates)
2. Remove old indexes using cloned data
3. Insert new note with updated indexes
4. This avoids simultaneous immutable and mutable borrows of `self.notes`

### File Watching Considerations

The file watcher uses `notify` which requires keeping the watcher alive:

- Current implementation leaks the watcher (`std::mem::forget`)
- In production, should use proper lifecycle management
- Watch mode runs indefinitely until Ctrl+C

### Query Execution Performance

Queries are executed in-memory with set operations:

- Tag queries: O(1) lookup in tag_index
- Content search: Delegated to ripgrep/grep (fast external tools with inverted indexes)
  - Falls back to title/alias search if external tools unavailable
- Date ranges: O(log n) BTree range iteration
- AND/OR: Set intersection/union operations
- Results are references to avoid cloning

## Neovim Plugin Notes

**Note**: The Neovim plugin has been separated into its own repository: [snot.nvim](https://github.com/yourusername/snot.nvim)

For plugin development, see the snot.nvim repository. The information below is kept for reference on how the plugin integrates with the CLI.

### Neovim Functions Reference (in snot.nvim repo)

**init.lua** - Plugin setup
- `M.setup(opts)` - Initialize plugin with configuration options
- `M.get_config()` - Get current plugin configuration

**backend.lua** - Rust CLI communication
- `M.run_command(args, callback)` - Execute snot CLI command asynchronously
- `M.init_vault(vault_path, callback)` - Initialize a new vault
- `M.index_vault(force, callback)` - Index vault (force=true to reindex all)
- `M.create_note(name, callback)` - Create new note with name
- `M.query_notes(query, callback)` - Execute SQL query
- `M.get_backlinks(file_path, callback)` - Get backlinks for a file
- `M.list_notes(query, callback)` - List all notes (optionally filtered)
- `M.update_note(file_path, callback)` - Update single file metadata

**commands.lua** - User command implementations
- `M.setup(config)` - Register all Neovim user commands
- `M.create_note(name)` - `:NoteNew` implementation
- `M.find_note(query)` - `:NoteFind` implementation
- `M.search_notes(query)` - `:NoteSearch` implementation
- `M.show_backlinks()` - `:NoteBacklinks` implementation
- `M.index_vault(force)` - `:NoteIndex` implementation
- `M.init_vault(vault_path)` - `:NoteInit` implementation
- `M.insert_link()` - `:NoteLink` implementation

**picker.lua** - File picker abstraction
- `M.pick(files, opts)` - Show picker with files (auto-detects fzf-lua/telescope/select)

**ui.lua** - UI components
- `M.show_results(results, title)` - Display query results in floating window

**completion.lua** - Auto-completion
- `M.setup()` - Initialize omnifunc completion
- `M.omnifunc(findstart, base)` - Vim's omnifunc implementation
- `M.setup_cmp()` - Initialize nvim-cmp integration (optional)

### Picker Integration

The plugin supports multiple pickers with auto-detection:

1. **fzf-lua** (preferred): Fast, native Lua implementation
   - Preview window with file content
   - Fuzzy matching with highlighting

2. **telescope.nvim** (alternative): If installed
   - Full telescope features and customization

3. **vim.ui.select** (fallback): Built-in Neovim picker
   - Always available, no dependencies

Configure with `opts.picker = "auto"` (default), `"fzf-lua"`, `"telescope"`, or `"select"`

### Completion System

Three completion modes supported:

1. **Omnifunc** (built-in): Works everywhere, activated with `<C-X><C-O>`
   - No dependencies required
   - Completes wiki-links and tags

2. **nvim-cmp** (optional): Better UX if user has nvim-cmp installed
   - Automatic popup on typing
   - Rich UI with icons and documentation

3. **blink.cmp** (optional): High-performance completion framework
   - Similar to nvim-cmp but faster
   - Setup via blink.cmp source registration

Completion triggers:

- `[[` triggers note name completion
- `#` triggers tag completion

To integrate with blink.cmp, add snot as a source in your blink.cmp config:
```lua
{
  'saghen/blink.cmp',
  opts = {
    sources = {
      default = { 'lsp', 'path', 'snippets', 'buffer', 'snot' },
      providers = {
        snot = {
          name = 'Snot',
          module = 'snot.completion.blink',
          enabled = function()
            return vim.bo.filetype == 'markdown'
          end,
        },
      },
    },
  },
}
```

### Async Communication

All backend calls use `vim.fn.jobstart` for async execution:

- Non-blocking - editor remains responsive
- stdout/stderr captured in buffers
- Callback invoked on job completion
- JSON parsed from stdout

## Testing Strategy

### Unit Tests

Each module has inline tests:

```bash
cargo test parser::tests::test_extract_tags
cargo test db::query::tests::test_parse_and_query
```

### Integration Testing

Manual testing workflow:

1. Create test vault: `cargo run -- init ./test_vault`
2. Add sample markdown files
3. Index: `cargo run -- index ./test_vault`
4. Query: `cargo run -- query ./test_vault "tag:test"`
5. Verify JSON output

### Performance Testing

For large vaults:

1. Generate 1000+ test notes with script
2. Measure initial index time
3. Modify one file
4. Measure reindex time (should only process changed file)
5. Query response time should be <50ms

## Common Development Tasks

### Adding a New Query Operator

1. Add variant to `Query` enum in `src/db/query.rs`
2. Update `QueryParser::parse_primary()` to recognize new keyword
3. Implement execution logic in `QueryExecutor::execute()`
4. Add tests in `query.rs` test module
5. Update README with new operator syntax

### Adding a Neovim Command

1. Add function in `nvim/lua/snot/commands.lua`
2. Register command in `setup()` with `nvim_create_user_command`
3. Add corresponding backend function in `backend.lua` if needed
4. Add CLI subcommand in Rust if new functionality required
5. Update README documentation

### Modifying Database Schema

When changing `Note` struct:

1. Update `Note` definition in `src/db/storage.rs`
2. Update all indexes that might use new field
3. Modify `insert()`, `update()`, `delete()` as needed
4. **Important**: Old database files will fail to deserialize - bump version or add migration
5. Update JSON serialization in command outputs

### Extending Markdown Parser

To support new markdown features:

1. Add extraction logic to `src/parser/links.rs`
2. Update `ParsedNote` struct with new fields
3. Update note processing in `src/commands/index.rs`
4. Add tests for new parsing logic

## Code Style and Conventions

- Use `anyhow::Result` for error handling in application code
- Use `thiserror` for library-level custom errors (currently not used but imported)
- Prefer `&Path` over `&PathBuf` in function signatures
- Clone only when necessary to satisfy borrow checker
- Use `rayon` par_iter for parallel processing of note collections
- All CLI output for consumption should be JSON
- Human-readable output uses `println!` with descriptive messages

## Known Limitations and Future Work

1. **Database Migration**: No migration strategy for schema changes
2. **Watcher Lifecycle**: File watcher is leaked in watch mode
3. **Tag Extraction**: Tags from all notes not easily queryable (would need tag aggregation command)
4. **Conflict Resolution**: No handling of concurrent modifications
5. **Graph Visualization**: Data structures support it, but no export command
6. **Template Support**: Mentioned in README but not implemented
7. **Daily Notes**: Common feature not yet implemented
