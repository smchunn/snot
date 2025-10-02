# SNOT Documentation

Welcome to the SNOT (Simple Note Organization Tool) documentation.

## User Guides

- **[Query Syntax Guide](query-syntax.md)** - Complete reference for the query language, including operators, boolean logic, and examples

## Reference Documentation

- **[Main README](../README.md)** - Project overview, installation, and quick start
- **[CLAUDE.md](../CLAUDE.md)** - Development guide for contributors

## Quick Links

### Getting Started
1. [Installation](../README.md#installation)
2. [Quick Start](../README.md#quick-start)
3. [Neovim Setup](../README.md#neovim-plugin)

### Core Features
- [Query Language](query-syntax.md) - Search and filter notes
- [File Picker](../README.md#file-picker) - Choose between fzf-lua, Telescope, or native picker
- [Note Format](../README.md#note-format) - YAML frontmatter and wiki-links
- [Auto-completion](../README.md#neovim-plugin) - Link and tag completion

### CLI Reference

```bash
# Initialize vault
snot init <vault-path>

# Index notes
snot index <vault-path> [--force]

# Query notes
snot query <vault-path> <query>

# Create note
snot create <vault-path> <name>

# Get backlinks
snot backlinks <vault-path> <file>

# List notes
snot list <vault-path> [--query <query>]

# Watch for changes
snot watch <vault-path>
```

### Neovim Commands

```vim
:NoteNew [name]        " Create new note
:NoteFind              " Open file picker
:NoteSearch [query]    " Search with query language
:NoteBacklinks         " Show backlinks to current note
:NoteIndex[!]          " Index vault (! to force)
:NoteInit [path]       " Initialize vault
:NoteLink              " Insert wiki-link to note
```

## Examples

### Query Examples

```bash
# Basic queries
snot query ~/notes "tags CONTAINS 'work'"
snot query ~/notes "content LIKE '%meeting%'"
snot query ~/notes "links_to = 'project-plan'"
snot query ~/notes "modified_date BETWEEN '2025-10-01' AND '2025-10-31'"

# Boolean logic
snot query ~/notes "tags CONTAINS 'work' AND content LIKE '%deadline%'"
snot query ~/notes "tags CONTAINS 'meeting' OR tags CONTAINS 'standup'"
snot query ~/notes "tags CONTAINS 'work' AND NOT tags CONTAINS 'done'"

# Complex queries
snot query ~/notes "(tags CONTAINS 'work' OR tags CONTAINS 'personal') AND NOT tags CONTAINS 'archived'"
snot query ~/notes "tags CONTAINS 'urgent' AND (tags CONTAINS 'bug' OR tags CONTAINS 'issue')"

# Full SQL syntax
snot query ~/notes "SELECT * FROM notes WHERE tags CONTAINS 'project'"
```

See [Query Syntax Guide](query-syntax.md) for complete documentation.

### Neovim Configuration

```lua
{
  dir = "~/dev/snot/nvim",
  name = "snot",
  opts = {
    vault_path = "~/notes",
    snot_bin = "snot",
    picker = "auto",  -- or "fzf-lua", "telescope", "select"
    enable_completion = true,
  },
  keys = {
    { "<leader>nn", "<cmd>NoteNew<cr>", desc = "New note" },
    { "<leader>nf", "<cmd>NoteFind<cr>", desc = "Find note" },
    { "<leader>ns", "<cmd>NoteSearch<cr>", desc = "Search notes" },
    { "<leader>nb", "<cmd>NoteBacklinks<cr>", desc = "Backlinks" },
  },
  cmd = { "NoteNew", "NoteFind", "NoteSearch", "NoteBacklinks", "NoteIndex", "NoteLink" },
  ft = "markdown",
}
```

## Architecture

SNOT consists of:

1. **Rust CLI** - Core note management, indexing, and query execution
2. **Neovim Plugin** - Editor integration with commands and pickers
3. **Custom Database** - Fast in-memory note storage with multiple indexes
4. **Query Language** - Powerful search with boolean logic

See [CLAUDE.md](../CLAUDE.md) for detailed architecture documentation.

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/snot/issues)
- **Development Guide**: [CLAUDE.md](../CLAUDE.md)
- **Main Documentation**: [README.md](../README.md)
