# Query Syntax Guide

SNOT features a powerful query language for searching and filtering notes. This guide covers all available operators and how to combine them.

## Table of Contents

- [Basic Operators](#basic-operators)
- [Boolean Logic](#boolean-logic)
- [Grouping with Parentheses](#grouping-with-parentheses)
- [Common Use Cases](#common-use-cases)
- [Tips and Best Practices](#tips-and-best-practices)

## Basic Operators

### `tag:tagname`

Find all notes containing a specific tag.

**Syntax:**
```
tag:tagname
```

**Examples:**
```bash
# Find all notes tagged with "work"
snot query ~/notes "tag:work"

# Find notes tagged "meeting"
snot query ~/notes "tag:meeting"

# From Neovim
:NoteSearch tag:project
```

**Note:** Tags can be defined in frontmatter or inline with `#tagname`.

---

### `contains:text`

Search for notes containing specific text in their content or title.

**Syntax:**
```
contains:text
contains:"multi word phrase"
```

**Examples:**
```bash
# Find notes containing "database"
snot query ~/notes "contains:database"

# Search for a phrase (use quotes)
snot query ~/notes 'contains:"customer feedback"'

# From Neovim
:NoteSearch contains:todo
```

**Note:** Searches are case-insensitive.

---

### `linked-to:note-id`

Find all notes that link to a specific note (backlinks).

**Syntax:**
```
linked-to:note-id
```

**Examples:**
```bash
# Find all notes linking to "project-plan"
snot query ~/notes "linked-to:project-plan"

# Find notes linking to a specific dated note
snot query ~/notes "linked-to:weekly-review-2025-10-02"

# From Neovim
:NoteSearch linked-to:main-index
```

**Note:** Use the note's ID (kebab-case filename without extension).

---

### `date:YYYY-MM-DD..YYYY-MM-DD`

Find notes modified within a date range.

**Syntax:**
```
date:start-date..end-date
```

**Examples:**
```bash
# Notes from January 2025
snot query ~/notes "date:2025-01-01..2025-01-31"

# Notes from the last quarter
snot query ~/notes "date:2025-01-01..2025-03-31"

# From Neovim
:NoteSearch date:2025-10-01..2025-10-31
```

**Note:** Dates must be in `YYYY-MM-DD` format. The range is inclusive.

---

## Boolean Logic

Combine queries using boolean operators for more powerful searches.

### `AND`

Both conditions must be true.

**Syntax:**
```
query1 AND query2
```

**Examples:**
```bash
# Notes tagged "work" AND containing "meeting"
snot query ~/notes "tag:work AND contains:meeting"

# Work notes from October
snot query ~/notes "tag:work AND date:2025-10-01..2025-10-31"

# Multiple conditions
snot query ~/notes "tag:project AND contains:deadline AND tag:urgent"
```

---

### `OR`

Either condition can be true.

**Syntax:**
```
query1 OR query2
```

**Examples:**
```bash
# Notes tagged "work" OR "project"
snot query ~/notes "tag:work OR tag:project"

# Notes containing "bug" OR "issue"
snot query ~/notes "contains:bug OR contains:issue"

# Multiple tags
snot query ~/notes "tag:meeting OR tag:standup OR tag:review"
```

---

### `NOT`

Exclude notes matching a condition.

**Syntax:**
```
NOT query
```

**Examples:**
```bash
# All work notes except meetings
snot query ~/notes "tag:work AND NOT tag:meeting"

# Notes without the "archived" tag
snot query ~/notes "NOT tag:archived"

# Exclude completed items
snot query ~/notes "tag:todo AND NOT contains:completed"
```

---

## Grouping with Parentheses

Use parentheses to control the order of operations and create complex queries.

**Syntax:**
```
(query1 OR query2) AND query3
```

**Examples:**
```bash
# Work or personal notes, but not archived
snot query ~/notes "(tag:work OR tag:personal) AND NOT tag:archived"

# Urgent items from specific projects
snot query ~/notes "tag:urgent AND (tag:project-a OR tag:project-b)"

# Complex date and tag combinations
snot query ~/notes "(tag:meeting OR tag:standup) AND date:2025-10-01..2025-10-31 AND NOT tag:cancelled"

# Multiple search terms
snot query ~/notes "tag:research AND (contains:ai OR contains:ml OR contains:llm)"
```

---

## Common Use Cases

### 1. **Finding Active Work Items**

```bash
# Work todos that aren't done
snot query ~/notes "tag:work AND tag:todo AND NOT contains:done"

# This week's tasks
snot query ~/notes "tag:todo AND date:2025-10-01..2025-10-07"
```

### 2. **Project Research**

```bash
# All research notes for a project
snot query ~/notes "tag:research AND tag:project-alpha"

# Research notes with specific keywords
snot query ~/notes "tag:research AND (contains:algorithm OR contains:optimization)"
```

### 3. **Meeting Notes**

```bash
# All meeting notes from October
snot query ~/notes "tag:meeting AND date:2025-10-01..2025-10-31"

# Client meetings only
snot query ~/notes "tag:meeting AND tag:client"

# Meetings that reference a specific project
snot query ~/notes "tag:meeting AND linked-to:project-plan"
```

### 4. **Content Organization**

```bash
# Find orphan notes (no backlinks, no tags)
snot query ~/notes "NOT tag:* AND NOT linked-to:*"

# Popular notes (linked from many places)
snot query ~/notes "linked-to:main-index"

# Recent untagged notes
snot query ~/notes "NOT tag:* AND date:2025-10-01..2025-10-31"
```

### 5. **Review and Cleanup**

```bash
# Old notes to review
snot query ~/notes "date:2020-01-01..2023-12-31 AND NOT tag:archived"

# Notes to archive
snot query ~/notes "tag:completed AND NOT tag:archived"

# Drafts to finish
snot query ~/notes "tag:draft AND NOT contains:published"
```

---

## Tips and Best Practices

### 1. **Quote Multi-Word Values**

Always use quotes for phrases or multi-word search terms:

```bash
# Correct
snot query ~/notes 'contains:"project update"'

# Incorrect (will fail)
snot query ~/notes "contains:project update"
```

### 2. **Use Specific Tags**

Create a tagging system for efficient queries:

```bash
# Status tags
tag:todo, tag:in-progress, tag:done, tag:blocked

# Category tags
tag:work, tag:personal, tag:project

# Priority tags
tag:urgent, tag:important, tag:low-priority
```

### 3. **Combine Date Ranges with Other Filters**

Date ranges are powerful when combined:

```bash
# Active work from this month
snot query ~/notes "tag:work AND NOT tag:done AND date:2025-10-01..2025-10-31"
```

### 4. **Build Complex Queries Incrementally**

Start simple and add filters:

```bash
# Step 1: Basic filter
tag:project

# Step 2: Add time range
tag:project AND date:2025-10-01..2025-10-31

# Step 3: Exclude completed
tag:project AND date:2025-10-01..2025-10-31 AND NOT tag:done
```

### 5. **Use Aliases in Neovim**

Create command aliases for frequently used queries:

```lua
-- In your Neovim config
vim.api.nvim_create_user_command("WorkTodos", function()
  require("snot.commands").search_notes("tag:work AND tag:todo AND NOT contains:done")
end, {})

vim.api.nvim_create_user_command("ThisWeekMeetings", function()
  require("snot.commands").search_notes("tag:meeting AND date:2025-10-01..2025-10-07")
end, {})
```

### 6. **Understand Operator Precedence**

When mixing AND/OR without parentheses:
- `NOT` has highest precedence
- `AND` is evaluated before `OR`

```bash
# This query: (tag:a AND tag:b) OR tag:c
tag:a AND tag:b OR tag:c

# Use parentheses for clarity
(tag:a AND tag:b) OR tag:c
```

---

## Query Language Grammar

For reference, here's the formal grammar:

```
query     := or_expr
or_expr   := and_expr ( "OR" and_expr )*
and_expr  := primary ( "AND" primary )*
primary   := "NOT" primary
           | "(" or_expr ")"
           | operator ":" value

operator  := "tag" | "contains" | "linked-to" | "date"
value     := quoted_string | unquoted_string
```

---

## Error Messages

Common errors and their meanings:

| Error | Cause | Fix |
|-------|-------|-----|
| `Expected ':' after key` | Missing colon | Use `tag:work` not `tag work` |
| `Expected identifier` | Invalid operator | Use valid operators: tag, contains, linked-to, date |
| `Invalid date format` | Wrong date format | Use YYYY-MM-DD format |
| `Date range must be in format` | Wrong range syntax | Use `start..end` format |
| `Unterminated quoted string` | Missing closing quote | Add closing quote to string |
| `Expected closing parenthesis` | Unmatched parentheses | Check parentheses matching |

---

## Examples from Neovim

Using the query language in Neovim commands:

```vim
" Search for work todos
:NoteSearch tag:work AND tag:todo

" Find recent meeting notes
:NoteSearch tag:meeting AND date:2025-10-01..2025-10-31

" Search with multi-word phrase
:NoteSearch contains:"important deadline"

" Complex query
:NoteSearch (tag:work OR tag:personal) AND NOT tag:archived AND date:2025-10-01..2025-10-31
```

---

## See Also

- [README.md](../README.md) - Main documentation
- [CLAUDE.md](../CLAUDE.md) - Development guide
- CLI help: `snot query --help`
