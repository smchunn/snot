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

### Core Features
- [Query Language](query-syntax.md) - Search and filter notes (shorthand + SQL-style)
- [Graph Operations](../README.md#graph-operations) - Neighbors, orphans, shortest path
- [Note Format](../README.md#note-format) - YAML frontmatter and wiki-links

### CLI Reference

```bash
# Initialize vault
snot init <vault-path>

# Index notes
snot index <vault-path> [--force]

# Query notes (shorthand or SQL-style)
snot query <vault-path> <query>

# Create note
snot create <vault-path> <name>

# Get backlinks
snot backlinks <vault-path> <file>

# List notes
snot list <vault-path> [--query <query>]

# List tags
snot tags <vault-path>

# Graph operations
snot graph neighbors <vault-path> <note-id> [--depth N]
snot graph orphans <vault-path>
snot graph path <vault-path> <note-a> <note-b>
snot graph stats <vault-path>

# Watch for changes
snot watch <vault-path>
```

## Examples

### Query Examples

```bash
# Shorthand syntax
snot query ~/notes "tag:work"
snot query ~/notes "#work title:meeting"
snot query ~/notes "~meting"
snot query ~/notes "-tag:archived"

# SQL-style syntax
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

# Graph-aware queries
snot query ~/notes "neighbors('project-plan', 2)"

# Full SQL syntax
snot query ~/notes "SELECT * FROM notes WHERE tags CONTAINS 'project'"
```

See [Query Syntax Guide](query-syntax.md) for complete documentation.

## Architecture

SNOT consists of:

1. **Rust CLI** - Core note management, indexing, and query execution
2. **Custom Database** - Fast in-memory note storage with multiple indexes and link graph
3. **Dual Query Language** - Shorthand for quick use, SQL-style for complex queries

See [CLAUDE.md](../CLAUDE.md) for detailed architecture documentation.

## Editor Integration

The **[snot.nvim](https://github.com/yourusername/snot.nvim)** plugin provides Neovim integration with commands, file picking, and auto-completion. See its repository for setup instructions.

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/snot/issues)
- **Development Guide**: [CLAUDE.md](../CLAUDE.md)
- **Main Documentation**: [README.md](../README.md)
