# SNOT - Simple Note Organization Tool

A Rust-based note management system with Neovim integration, designed as a fast and efficient alternative to Obsidian.

## Features

- **Custom Database**: Fast, lightweight note storage with custom query language
- **SHA-256 Checksum Caching**: Incremental updates - only reindex changed files
- **Powerful Query Language**: Search notes by tags, content, links, and dates
- **File Watcher**: Real-time updates when notes change
- **FZF Integration**: Fast file picking with preview
- **Neovim Plugin**: Seamless integration with your editor
- **Smart Note Creation**: Automatic kebab-case naming with dates
- **Backlinks Support**: Track connections between notes
- **Markdown Parsing**: Extract tags, wiki-links, and frontmatter

## Documentation

- **[Query Syntax Guide](docs/query-syntax.md)** - Complete reference for search queries
- **[Documentation Index](docs/)** - All documentation and guides

## Installation

### Rust Backend

```bash
# Build from source
cargo build --release

# Install binary
cargo install --path .
```

### Neovim Plugin

The Neovim plugin is maintained in a separate repository: **[snot.nvim](https://github.com/yourusername/snot.nvim)**

Using [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{
  'yourusername/snot.nvim',
  opts = {
    vault_path = '~/notes',       -- supports ~ home expansion
    snot_bin = 'snot',            -- or full path like "/usr/local/bin/snot"
    picker = 'auto',              -- "auto", "fzf-lua", "telescope", or "select"
    enable_completion = true,     -- optional, default: true
  },
  keys = {
    { '<leader>nn', '<cmd>NoteNew<cr>', desc = 'New note' },
    { '<leader>nf', '<cmd>NoteFind<cr>', desc = 'Find note' },
    { '<leader>ns', '<cmd>NoteSearch<cr>', desc = 'Search notes' },
    { '<leader>nb', '<cmd>NoteBacklinks<cr>', desc = 'Show backlinks' },
    { '<leader>ni', '<cmd>NoteIndex<cr>', desc = 'Index vault' },
    { '<leader>nl', '<cmd>NoteLink<cr>', desc = 'Insert link' },
  },
  cmd = { 'NoteNew', 'NoteFind', 'NoteSearch', 'NoteBacklinks', 'NoteIndex', 'NoteInit', 'NoteLink' },
  ft = 'markdown',
}
```

See the [snot.nvim documentation](https://github.com/yourusername/snot.nvim) for more details.

## Quick Start

### 1. Initialize a Vault

```bash
# CLI
snot init /path/to/vault

# Or from Neovim
:NoteInit /path/to/vault
```

### 2. Index Your Notes

```bash
# CLI
snot index /path/to/vault

# Force reindex
snot index /path/to/vault --force

# From Neovim
:NoteIndex
:NoteIndex!  " Force reindex
```

### 3. Create Notes

```bash
# CLI
snot create /path/to/vault "My New Note"
# Creates: my-new-note-2025-10-02.md
# With YAML frontmatter:
#   ---
#   id: my-new-note-2025-10-02
#   aliases:
#     - My New Note
#   tags: []
#   ---

# From Neovim
:NoteNew My New Note
```

## Usage

### CLI Commands

#### Query Notes

```bash
# Search by tag
snot query /path/to/vault "tags CONTAINS 'work'"

# Search by content
snot query /path/to/vault "content LIKE '%meeting%'"

# Find notes linking to another note
snot query /path/to/vault "links_to = 'project-plan'"

# Date range queries
snot query /path/to/vault "modified_date BETWEEN '2025-01-01' AND '2025-03-01'"

# Combine queries (SQL-style)
snot query /path/to/vault "tags CONTAINS 'work' AND content LIKE '%meeting%'"
snot query /path/to/vault "tags CONTAINS 'work' OR tags CONTAINS 'personal'"
snot query /path/to/vault "tags CONTAINS 'work' AND NOT content LIKE '%done%'"

# Full SQL syntax (optional)
snot query /path/to/vault "SELECT * FROM notes WHERE tags CONTAINS 'urgent'"
```

#### Get Backlinks

```bash
snot backlinks /path/to/vault /path/to/note.md
```

#### List Notes (for FZF)

```bash
# List all notes
snot list /path/to/vault

# List with query filter
snot list /path/to/vault --query "tag:work"

# Use with FZF
snot list /path/to/vault | fzf --preview 'cat {}'
```

#### Watch Vault

```bash
# Start file watcher daemon
snot watch /path/to/vault
```

### Neovim Commands

- `:NoteNew [name]` - Create a new note
- `:NoteFind` - Open file picker to find notes
- `:NoteSearch [query]` - Search using query language
- `:NoteBacklinks` - Show backlinks to current note
- `:NoteIndex[!]` - Index vault (! to force)
- `:NoteInit [path]` - Initialize vault
- `:NoteLink` - Insert link to another note

### Query Language

SNOT uses **SQL-style queries** for searching notes:

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
2. **Tags**: `#tag` or in frontmatter
3. **Frontmatter**: id, aliases, and tags

### File Picker

The Neovim plugin supports multiple pickers (auto-detected):

- **fzf-lua** (recommended): Fast, Lua-native, with preview support
- **Telescope**: If you have telescope.nvim installed
- **vim.ui.select**: Fallback native picker

Configure in your setup:

```lua
opts = {
  picker = "auto",  -- or "fzf-lua", "telescope", "select"
}
```

## Architecture

### Directory Structure

```
snot/
├── src/
│   ├── db/               # Custom database implementation
│   │   ├── storage.rs    # Note storage and indexing
│   │   └── query.rs      # Query language parser
│   ├── parser/           # Markdown parsing
│   │   └── links.rs      # Extract links, tags
│   ├── watcher/          # File watching
│   │   └── checksum.rs   # SHA-256 caching
│   └── commands/         # CLI commands
└── nvim/
    └── lua/snot/
        ├── init.lua        # Plugin entry point
        ├── backend.lua     # Rust backend communication
        ├── commands.lua    # Neovim commands
        ├── ui.lua          # UI components
        ├── picker.lua      # File picker (FZF/Telescope/select)
        └── completion.lua  # Auto-completion
```

### Database Design

- **In-memory HashMap** for O(1) note lookups
- **Tag index** for fast tag-based queries
- **Date index (BTree)** for efficient range queries
- **Path index** for file-to-note mapping
- **Automatic backlink tracking**
- **Binary serialization** for fast load/save

### Performance

- Incremental updates using SHA-256 checksums
- Parallel file processing with Rayon
- Optimized for vaults with 1000+ notes
- Query responses typically under 50ms

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

- `serde` + `serde_json` - JSON serialization
- `notify` - File watching
- `sha2` - Checksum calculation
- `chrono` - Date handling
- `rayon` - Parallel processing
- `pulldown-cmark` - Markdown parsing
- `clap` - CLI argument parsing
- `regex` - Pattern matching
- `walkdir` - Directory traversal

## Neovim Requirements

- Neovim 0.7+
- Optional: [fzf-lua](https://github.com/ibhagwan/fzf-lua) (recommended for file picking)
- Optional: [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim) (alternative picker)
- Falls back to vim.ui.select if neither available

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
