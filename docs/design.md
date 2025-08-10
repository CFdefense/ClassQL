# ClassQL — Initial Technical Design Document

---

## 1. Introduction

Classy is a terminal-based natural language query engine for academic course scheduling data stored locally in SQLite. It features a rich Text User Interface built with **ratatui** and leverages **sqlx** for safe, asynchronous database access. The system supports flexible, fuzzy, and semantic search over courses, sections, and instructors.

---

## 2. Goals and Scope

- Deliver a fast, interactive TUI for course data exploration.  
- Support natural language queries with fuzzy matching and semantic understanding.  
- Integrate seamlessly with `classy-sync` for data synchronization.  
- Provide an extensible query parsing system for complex yet simple to understand search queries.  
- Use robust Rust libraries for performance and safety.

---

## 3. System Architecture

### 3.1 Components Overview

| Component        | Description                                                    |
|------------------|----------------------------------------------------------------|
| TUI Client       | Built with `ratatui`, handles input, rendering, and navigation.|
| Query Processor  | Tokenizes, parses, semantically analyzes, and builds SQL queries. |
| Database Layer   | Uses `sqlx` to perform async, parameterized queries on SQLite. |
| Data Sync Module | Syncs data from external sources using `classy-sync`.          |

---

### 3.2 Data Flow Diagram

User Input (TUI)
↓
Lexer / Tokenizer
↓
Parser (AST/structured query)
↓
Semantic Analyzer (normalize & validate)
↓
SQL Query Generator (safe, parameterized SQL)
↓
sqlx Async Query Execution
↓
Format Results
↓
Display in TUI

---

## 4. Detailed Design

### 4.1 TUI Client (ratatui)

- Accepts live text input and dynamically updates results.  
- Enables keyboard navigation and selection of results.  
- Displays tables with highlights and detailed record views.  
- Uses async Rust to keep UI responsive during database operations.

### 4.2 Query Processor

- **Lexer:** Splits input into tokens (words, punctuation).  
- **Tokenizer:** Categorizes tokens (`course_code`, `instructor`, `day`, `time`, `keywords`).  
- **Parser:** Builds structured queries supporting boolean logic.  
- **Semantic Analyzer:** Normalizes synonyms (e.g., `thur` → `thursday`), interprets time ranges, validates tokens.  
- **Query Builder:** Build query using structured fields to refine search parameters
- **Semantic Search** Non structured fields are then additionally filtered with semantic search
### 4.3 Database Layer (sqlx)

- Manages SQLite connections asynchronously.  
- Executes raw and parameterized SQL queries.  
- Integrates SQLite extensions (e.g., fuzzy search).  
- Provides safe, efficient access to data.

### 4.4 Data Sync Module

- Interfaces with `classy-sync` to pull latest course, section, and instructor data.  
- Supports choosing school and academic term.  
- Maintains data integrity and schema updates.
- Runs a along side the main thread.

---

## 5. Data Model Snapshot (Detailed)

| Table            | Key Columns / Attributes                                                                                          |
|------------------|------------------------------------------------------------------------------------------------------------------|
| **schools**      | id (TEXT, PK), name (TEXT)                                                                                        |
| **terms**        | year (INTEGER, PK), season (TEXT, PK, CHECK in 'Spring','Fall','Winter','Summer')                                 |
| **term_collections** | id (TEXT, PK), school_id (TEXT, FK), year (INTEGER), season (TEXT, CHECK), name (TEXT), still_collecting (BOOL)   |
| **professors**   | id (TEXT, PK), school_id (TEXT, PK, FK), name (TEXT), email_address (TEXT), first_name (TEXT), last_name (TEXT), other (JSON TEXT) |
| **courses**      | school_id (TEXT, PK, FK), subject_code (TEXT, PK), number (TEXT, PK), subject_description (TEXT), title (TEXT), description (TEXT), credit_hours (REAL), prerequisites (TEXT), corequisites (TEXT), other (JSON TEXT) |
| **sections**     | sequence (TEXT, PK), term_collection_id (TEXT, PK, FK), subject_code (TEXT, PK, FK), course_number (TEXT, PK, FK), school_id (TEXT, PK, FK), max_enrollment (INTEGER), instruction_method (TEXT), campus (TEXT), enrollment (INTEGER), primary_professor_id (TEXT, FK), other (JSON TEXT), location_id (TEXT, optional) |
| **meeting_times**| sequence (INTEGER, PK), section_sequence (TEXT, PK, FK), term_collection_id (TEXT, PK, FK), subject_code (TEXT, PK, FK), course_number (TEXT, PK, FK), school_id (TEXT, PK, FK), start_date (TEXT), end_date (TEXT), meeting_type (TEXT), start_minutes (TEXT), end_minutes (TEXT), is_monday (BOOL), is_tuesday (BOOL), is_wednesday (BOOL), is_thursday (BOOL), is_friday (BOOL), is_saturday (BOOL), is_sunday (BOOL), other (JSON TEXT) |
---

## 6. Example Queries
User queries should be rich enough to filter record attributes as well as a record's related objects.
Related objects could 1:1 or 1:many which means some of these filters will be over aggregates.

| User Input            | SQL Translation Example                                                                                      |
|-----------------------|-------------------------------------------------------------------------------------------------------------|
| `CMPT 120L Alan Thursday` | WHERE LOWER(course_code) LIKE '%cmpt%' AND LOWER(course_code) LIKE '%120l%' AND LOWER(instructor) LIKE '%alan%' AND LOWER(meeting_days) LIKE '%thursday%' |
| `sections Alan`       | WHERE LOWER(instructor) LIKE '%alan%'                                                                        |
| `after 2pm meetings`  | WHERE meeting_time > '14:00'                                                                                  |

---

### 6.1 Section related objects
- Professors (all attributes)
- Meeting times aggregates
    - has ANY day
    - does not have day
    - has ANY (time before or after TIME) 
    - does not have time
    - has ALL non-null values for day / time


### 6.2 Professor Requirements
- Arguments filtered against professor attributes.
- Section/Course aggregates
    - teaches ANY (section/course predicate)

### 6.3 Course Requirements
- Arguments filtered against course attributes.
- Course/ Meeting times aggregates
    - has ANY (section/meeting times predicate)
- Professor aggregates
    - has ANY (professor aggregate)

## 7. Error Handling & Edge Cases

- Friendly messages for invalid syntax or unrecognized tokens.  
- Graceful handling of empty result sets.  
- Validation of time and day inputs.  
- Defensive programming against injection or malformed input.

---

## 8. Extensibility and Future Enhancements

- Full boolean expression parsing: AND, OR, NOT.  
- Rich semantic search via ML models or advanced heuristics.  
- Autocomplete and query suggestions in the TUI.  
- Expand sync support for additional schools and terms.  
- Export search results (CSV, calendar integration).

---

## 9. Development Roadmap

1. Initialize Rust project with `ratatui` and `sqlx`.  
2. Define and create SQLite schema; import sample data via `classy-sync`.  
3. Build a basic TUI accepting input and displaying query results.  
4. Develop lexer and tokenizer for user queries.  
5. Implement parser and semantic analyzer.  
6. Integrate SQL query generation and database querying.  
7. Enhance search with fuzzy matching and synonym normalization.  
8. Refine TUI UX and add detailed views and navigation.  
9. Iterate with feedback and extend query capabilities.
