# Classy — Natural Language Course Search Design Doc

---

## Overview

**Classy** is a terminal-based interactive natural language query engine for academic course scheduling data, powered by a local SQLite database. It offers live, fuzzy, and semantic search over sections, courses, and professors using a simple, easy-to-understand query syntax.

---

## Core Components

### 1. Text-Based User Interface (TUI)

- Provides a responsive, keyboard-driven interface for entering search queries and browsing results.
- Displays search results in a clean, tabular format with selectable entries.
- Enables drilling down into detailed views of related records.

**Library:**

- **Rust:** [ratatui](https://github.com/ratatui/ratatui) — modern, flexible terminal UI library for building rich interactive applications.

---

### 2. Database

- Uses **SQLite** for local storage of course data.
- Syncs data with the [classy-sync](https://github.com/Pjt727/classy-sync) tool to fetch course information from different schools and terms.
- Uses [sqlx](https://github.com/launchbadge/sqlx) for asynchronous, type-safe database interactions.
- Primary querying will use **raw SQL** for flexibility and performance, with `sqlx` helping manage connections, queries, and safety.

---

## Requirements & Features

### Data Sync

- Integrate with `classy-sync` to pull course and schedule data.
- Allow users to select the school and optionally the academic term to load data from.

### Search Functionality

- **Live text search** that updates immediately as the user types, querying the local SQLite DB.
- Search is **case-insensitive** and supports:
  - Simple string matching
  - Fuzzy matching and semantic search possibilities (leveraging libraries such as [sqlean fuzzy](https://github.com/nalgeon/sqlean/blob/main/docs/fuzzy.md))
- Ability to drill into search results for detailed information.

### Search Domains

- **Sections:** Individual course sections including meeting times, locations.
- **Professors:** Search by instructor names.
- **Courses:** Search course titles, codes, and descriptions.

### Related Records & Syntax Enhancements

- Queries can implicitly or explicitly reference related data.
- Normalize synonyms for days (e.g., `"th"`, `"thur"`, `"thurs"`, `"thursday"` all map to Thursday).
- Support special keywords or field labels (e.g., `name:Alan`).
- Allow scoping with keywords like `"section"`, `"meetings"` to focus the query.

---

## Expanding Query Capabilities (Notes)

- **Boolean Logic:** Add support for `AND`, `OR`, `NOT` in queries.
- **Field-Specific Search:** Implement field labels to narrow searches, e.g., `instructor:Alan`, `course:CMPT`.
- **Range Queries:** Support time ranges like `after 2pm`, `before 10am`, or `between 1pm and 3pm`.
- **Fuzzy Matching & Semantic Search:** Integrate SQLite extensions or Rust crates for approximate matching and semantic understanding.
- **Synonym Handling:** Map common synonyms or abbreviations to normalized forms automatically.
- **Query Parsing:** Build a small parser that produces an AST from natural language input to improve flexibility and maintainability.
- **Related Records:** Implement “reverse keywords” or relation traversal, so users can query attributes of related tables seamlessly.

---