# Query Syntax Guide

SNOT supports two query syntaxes: **shorthand** for quick CLI use and **SQL-style** for complex queries. Both parse into the same AST and can be used interchangeably. This guide covers all available operations and how to combine them.

## Table of Contents

- [Basic SQL Syntax](#basic-sql-syntax)
- [Query Operators](#query-operators)
- [Boolean Logic](#boolean-logic)
- [Common Use Cases](#common-use-cases)
- [Tips and Best Practices](#tips-and-best-practices)

## Basic SQL Syntax

### Full SQL Format

```sql
SELECT * FROM notes WHERE <condition>
```

### Shortened Formats

For convenience, you can omit `SELECT * FROM notes`:

```sql
-- Full format
SELECT * FROM notes WHERE tags CONTAINS 'work'

-- With WHERE keyword
WHERE tags CONTAINS 'work'

-- Just the condition (simplest)
tags CONTAINS 'work'
```

All three formats are equivalent and produce the same results.

## Query Operators

### `tags CONTAINS 'value'`

Find all notes containing a specific tag.

**Syntax:**
```sql
tags CONTAINS 'tagname'
```

**Examples:**
```bash
# Find all notes tagged with "work"
snot query ~/notes "tags CONTAINS 'work'"

# Find notes tagged "meeting"
snot query ~/notes "WHERE tags CONTAINS 'meeting'"

# Full SQL
snot query ~/notes "SELECT * FROM notes WHERE tags CONTAINS 'project'"
```

**Note:** Tags can be defined in frontmatter or inline with `#tagname`.

---

### `content LIKE '%text%'`

Search for notes containing specific text in their content or title.

**Syntax:**
```sql
content LIKE '%search text%'
```

**Examples:**
```bash
# Find notes containing "database"
snot query ~/notes "content LIKE '%database%'"

# Search for a phrase
snot query ~/notes "content LIKE '%customer feedback%'"
```

**Note:** Use `%` as wildcards (optional - they're added automatically if omitted).

---

### `links_to = 'note-id'`

Find all notes that link to a specific note (backlinks).

**Syntax:**
```sql
links_to = 'note-id'
-- or
links_to LIKE '%pattern%'
```

**Examples:**
```bash
# Find all notes linking to "project-plan"
snot query ~/notes "links_to = 'project-plan'"

# Find notes linking to notes matching a pattern
snot query ~/notes "links_to LIKE '%weekly-review%'"
```

**Note:** Use the note's ID (kebab-case filename without extension).

---

### `modified_date BETWEEN 'start' AND 'end'`

Find notes modified within a date range.

**Syntax:**
```sql
modified_date BETWEEN 'YYYY-MM-DD' AND 'YYYY-MM-DD'
```

**Examples:**
```bash
# Notes from January 2025
snot query ~/notes "modified_date BETWEEN '2025-01-01' AND '2025-01-31'"

# Notes from the last quarter
snot query ~/notes "modified_date BETWEEN '2025-01-01' AND '2025-03-31'"
```

**Note:** Dates must be in `YYYY-MM-DD` format. Both dates are inclusive.

---

## Boolean Logic

Combine conditions using SQL boolean operators.

### `AND`

Both conditions must be true.

**Syntax:**
```sql
condition1 AND condition2
```

**Examples:**
```bash
# Notes tagged "work" AND containing "meeting"
snot query ~/notes "tags CONTAINS 'work' AND content LIKE '%meeting%'"

# Work notes from October
snot query ~/notes "tags CONTAINS 'work' AND modified_date BETWEEN '2025-10-01' AND '2025-10-31'"

# Multiple conditions
snot query ~/notes "tags CONTAINS 'project' AND content LIKE '%deadline%' AND tags CONTAINS 'urgent'"
```

---

### `OR`

Either condition can be true.

**Syntax:**
```sql
condition1 OR condition2
```

**Examples:**
```bash
# Notes tagged "work" OR "project"
snot query ~/notes "tags CONTAINS 'work' OR tags CONTAINS 'project'"

# Notes containing "bug" OR "issue"
snot query ~/notes "content LIKE '%bug%' OR content LIKE '%issue%'"

# Multiple tags
snot query ~/notes "tags CONTAINS 'meeting' OR tags CONTAINS 'standup' OR tags CONTAINS 'review'"
```

---

### `NOT`

Exclude notes matching a condition.

**Syntax:**
```sql
NOT condition
```

**Examples:**
```bash
# All work notes except meetings
snot query ~/notes "tags CONTAINS 'work' AND NOT tags CONTAINS 'meeting'"

# Notes without the "archived" tag
snot query ~/notes "NOT tags CONTAINS 'archived'"

# Exclude completed items
snot query ~/notes "tags CONTAINS 'todo' AND NOT content LIKE '%completed%'"
```

---

### Parentheses for Grouping

Use parentheses to control the order of operations.

**Syntax:**
```sql
(condition1 OR condition2) AND condition3
```

**Examples:**
```bash
# Work or personal notes, but not archived
snot query ~/notes "(tags CONTAINS 'work' OR tags CONTAINS 'personal') AND NOT tags CONTAINS 'archived'"

# Urgent items from specific projects
snot query ~/notes "tags CONTAINS 'urgent' AND (tags CONTAINS 'project-a' OR tags CONTAINS 'project-b')"

# Complex query
snot query ~/notes "(tags CONTAINS 'meeting' OR tags CONTAINS 'standup') AND modified_date BETWEEN '2025-10-01' AND '2025-10-31' AND NOT tags CONTAINS 'cancelled'"
```

---

## Common Use Cases

### 1. **Finding Active Work Items**

```sql
-- Work todos that aren't done
tags CONTAINS 'work' AND tags CONTAINS 'todo' AND NOT content LIKE '%done%'

-- This week's tasks
tags CONTAINS 'todo' AND modified_date BETWEEN '2025-10-01' AND '2025-10-07'
```

### 2. **Project Research**

```sql
-- All research notes for a project
tags CONTAINS 'research' AND tags CONTAINS 'project-alpha'

-- Research notes with specific keywords
tags CONTAINS 'research' AND (content LIKE '%algorithm%' OR content LIKE '%optimization%')
```

### 3. **Meeting Notes**

```sql
-- All meeting notes from October
tags CONTAINS 'meeting' AND modified_date BETWEEN '2025-10-01' AND '2025-10-31'

-- Client meetings only
tags CONTAINS 'meeting' AND tags CONTAINS 'client'

-- Meetings that reference a specific project
tags CONTAINS 'meeting' AND links_to = 'project-plan'
```

### 4. **Content Organization**

```sql
-- Find notes with specific tag
tags CONTAINS 'important'

-- Popular notes (linked from other places)
links_to = 'main-index'

-- Recent untagged notes
NOT tags CONTAINS '%' AND modified_date BETWEEN '2025-10-01' AND '2025-10-31'
```

### 5. **Review and Cleanup**

```sql
-- Old notes to review
modified_date BETWEEN '2020-01-01' AND '2023-12-31' AND NOT tags CONTAINS 'archived'

-- Notes to archive
tags CONTAINS 'completed' AND NOT tags CONTAINS 'archived'

-- Drafts to finish
tags CONTAINS 'draft' AND NOT content LIKE '%published%'
```

---

## Tips and Best Practices

### 1. **Always Quote String Values**

Use single or double quotes for all string values:

```sql
-- Correct
tags CONTAINS 'work'
content LIKE '%project update%'

-- Incorrect (will fail)
tags CONTAINS work
```

### 2. **Wildcards in LIKE**

The `%` wildcards are optional but recommended for clarity:

```sql
-- Both work the same
content LIKE '%meeting%'
content LIKE 'meeting'  -- automatically adds % wildcards
```

### 3. **Use Specific Tags**

Create a tagging system for efficient queries:

```sql
-- Status tags
tags CONTAINS 'todo', 'in-progress', 'done', 'blocked'

-- Category tags
tags CONTAINS 'work', 'personal', 'project'

-- Priority tags
tags CONTAINS 'urgent', 'important', 'low-priority'
```

### 4. **Build Complex Queries Incrementally**

Start simple and add filters:

```sql
-- Step 1: Basic filter
tags CONTAINS 'project'

-- Step 2: Add time range
tags CONTAINS 'project' AND modified_date BETWEEN '2025-10-01' AND '2025-10-31'

-- Step 3: Exclude completed
tags CONTAINS 'project' AND modified_date BETWEEN '2025-10-01' AND '2025-10-31' AND NOT tags CONTAINS 'done'
```

### 5. **Operator Precedence**

When mixing AND/OR without parentheses:
- `NOT` has highest precedence
- `AND` is evaluated before `OR`

```sql
-- This query: (A AND B) OR C
tags CONTAINS 'a' AND tags CONTAINS 'b' OR tags CONTAINS 'c'

-- Use parentheses for clarity
(tags CONTAINS 'a' AND tags CONTAINS 'b') OR tags CONTAINS 'c'
```

---

## SQL Reference

### Supported Columns

| Column | Type | Description | Operators |
|--------|------|-------------|-----------|
| `tags` | String | Note tags | `CONTAINS` |
| `content` | String | Note content/title | `LIKE` |
| `links_to` | String | Note ID this links to | `=`, `LIKE` |
| `modified_date` | Date | Last modified date | `BETWEEN ... AND ...` |

### Supported Operators

| Operator | Example | Description |
|----------|---------|-------------|
| `CONTAINS` | `tags CONTAINS 'work'` | Check if tag exists |
| `LIKE` | `content LIKE '%text%'` | Pattern match content |
| `=` | `links_to = 'note-id'` | Exact match |
| `BETWEEN ... AND ...` | `modified_date BETWEEN '2025-01-01' AND '2025-01-31'` | Date range |
| `AND` | `condition1 AND condition2` | Both must be true |
| `OR` | `condition1 OR condition2` | Either must be true |
| `NOT` | `NOT condition` | Negate condition |

---

## Error Messages

Common errors and their solutions:

| Error | Cause | Fix |
|-------|-------|-----|
| `Expected keyword 'CONTAINS'` | Wrong operator for tags | Use `tags CONTAINS 'value'` |
| `Expected keyword 'LIKE'` | Wrong operator for content | Use `content LIKE '%value%'` |
| `Expected quoted string` | Missing quotes | Quote all string values: `'value'` |
| `Invalid date format` | Wrong date format | Use YYYY-MM-DD format |
| `Expected keyword 'AND' after BETWEEN` | Missing AND in date range | Use `BETWEEN 'start' AND 'end'` |
| `Unknown column` | Invalid column name | Use: tags, content, links_to, modified_date |

---

## Examples from CLI

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
snot query ~/notes "SELECT * FROM notes WHERE tags CONTAINS 'urgent' AND (tags CONTAINS 'bug' OR tags CONTAINS 'issue')"
```

## See Also

- [README.md](../README.md) - Main documentation
- [CLAUDE.md](../CLAUDE.md) - Development guide
- CLI help: `snot query --help`
