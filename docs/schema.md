# Database Schema

This document describes the expected database schema that ClassQL receives per school via **Classy Sync**. Each school's data should conform to this SQLite schema structure.

## Overview

The schema has been converted from PostgreSQL to SQLite format. When Classy Sync synchronizes data for a school, it should provide a SQLite database file containing tables structured according to this schema.

## Conversion Notes

### 1. ENUM Types
SQLite does not have a native ENUM type. The PostgreSQL ENUMs (`season_enum`, `term_collection_status_enum`) have been converted to TEXT columns with a CHECK constraint to simulate the same behavior.

### 2. JSONB Type
SQLite does not have a JSONB type. The `jsonb` columns have been converted to TEXT. A `CHECK(json_valid(column_name))` constraint has been added to ensure the text stored is valid JSON.

### 3. Regex Constraints
PostgreSQL's regex matching operator (`~`) is not available in SQLite. The CHECK constraints using regex have been removed. Application-level validation is recommended for these fields.

### 4. Data Types
- **BOOL** is converted to INTEGER with a CHECK constraint (0 for false, 1 for true).
- **TIMESTAMP** and **TIME** are converted to TEXT. It's common to store dates/times as ISO8601 strings (`'YYYY-MM-DD HH:MM:SS.SSS'`).

### 5. Foreign Keys
Foreign key constraints, including composite keys and `ON DELETE CASCADE`, are supported in SQLite and have been preserved. Note: `PRAGMA foreign_keys = ON;` must be executed for each connection to enforce foreign key constraints in SQLite.

---

## Schema Definition

### Table: `schools`

Stores basic school information.

```sql
CREATE TABLE schools (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
    -- The PostgreSQL constraint `id ~ '^[a-zA-Z]*$'` has been removed.
    -- SQLite does not have a built-in regex constraint like this.
    -- This check should be handled at the application level.
);
```

**Columns:**
- `id` (TEXT, PRIMARY KEY): Unique identifier for the school
- `name` (TEXT, NOT NULL): Name of the school

---

### Table: `terms`

Represents academic terms (semesters/quarters).

```sql
CREATE TABLE terms (
    year INTEGER,
    season TEXT CHECK( season IN ('Spring', 'Fall', 'Winter', 'Summer') ), -- Replaces season_enum

    PRIMARY KEY (year, season)
);
```

**Columns:**
- `year` (INTEGER): Academic year
- `season` (TEXT): Season of the term (Spring, Fall, Winter, or Summer)

**Constraints:**
- Composite PRIMARY KEY: `(year, season)`

---

### Table: `term_collections`

Groups terms together for a school.

```sql
CREATE TABLE term_collections (
    id TEXT,
    school_id TEXT,

    year INTEGER NOT NULL,
    season TEXT NOT NULL CHECK( season IN ('Spring', 'Fall', 'Winter', 'Summer') ), -- Replaces season_enum
    name TEXT,
    still_collecting INTEGER NOT NULL CHECK(still_collecting IN (0, 1)), -- Replaces BOOL
    FOREIGN KEY (school_id) REFERENCES schools(id),
    FOREIGN KEY (year, season) REFERENCES terms(year, season),
    PRIMARY KEY (id, school_id)
    -- The PostgreSQL constraint `id ~ '^[a-zA-Z0-9]*$'` has been removed.
    -- This check should be handled at the application level.
);
```

**Columns:**
- `id` (TEXT): Unique identifier for the term collection
- `school_id` (TEXT): Reference to the school
- `year` (INTEGER, NOT NULL): Academic year
- `season` (TEXT, NOT NULL): Season of the term
- `name` (TEXT): Name of the term collection
- `still_collecting` (INTEGER, NOT NULL): Whether data is still being collected (0 = false, 1 = true)

**Foreign Keys:**
- `school_id` → `schools(id)`
- `(year, season)` → `terms(year, season)`

**Constraints:**
- Composite PRIMARY KEY: `(id, school_id)`

---

### Table: `professors`

Stores professor/instructor information.

```sql
CREATE TABLE professors (
    id TEXT,
    school_id TEXT,

    name TEXT NOT NULL,
    email_address TEXT,
    first_name TEXT,
    last_name TEXT,
    other TEXT, -- This column should contain valid JSON.
    FOREIGN KEY (school_id) REFERENCES schools(id),
    PRIMARY KEY (id, school_id),
    CHECK(other IS NULL OR json_valid(other)) -- Ensures 'other' is valid JSON if not NULL
);
```

**Columns:**
- `id` (TEXT): Unique identifier for the professor
- `school_id` (TEXT): Reference to the school
- `name` (TEXT, NOT NULL): Full name of the professor
- `email_address` (TEXT): Email address
- `first_name` (TEXT): First name
- `last_name` (TEXT): Last name
- `other` (TEXT): Additional JSON data

**Foreign Keys:**
- `school_id` → `schools(id)`

**Constraints:**
- Composite PRIMARY KEY: `(id, school_id)`
- `other` must be valid JSON if not NULL

---

### Table: `courses`

Stores course catalog information.

```sql
CREATE TABLE courses (
    school_id TEXT,
    subject_code TEXT,
    number TEXT,

    subject_description TEXT,
    title TEXT,
    description TEXT,
    credit_hours REAL NOT NULL,
    prerequisites TEXT,
    corequisites TEXT,
    other TEXT, -- This column should contain valid JSON.

    FOREIGN KEY (school_id) REFERENCES schools(id),
    PRIMARY KEY (school_id, subject_code, number),
    CHECK(other IS NULL OR json_valid(other)) -- Ensures 'other' is valid JSON if not NULL
    -- The PostgreSQL constraints on `subject_code` and `number` using regex
    -- have been removed and should be handled at the application level.
);
```

**Columns:**
- `school_id` (TEXT): Reference to the school
- `subject_code` (TEXT): Subject code (e.g., "CS", "MATH")
- `number` (TEXT): Course number (e.g., "101", "301")
- `subject_description` (TEXT): Description of the subject
- `title` (TEXT): Course title
- `description` (TEXT): Course description
- `credit_hours` (REAL, NOT NULL): Number of credit hours
- `prerequisites` (TEXT): Prerequisites for the course
- `corequisites` (TEXT): Corequisites for the course
- `other` (TEXT): Additional JSON data

**Foreign Keys:**
- `school_id` → `schools(id)`

**Constraints:**
- Composite PRIMARY KEY: `(school_id, subject_code, number)`
- `other` must be valid JSON if not NULL

---

### Table: `sections`

Stores individual course sections (specific instances of courses).

```sql
CREATE TABLE sections (
    sequence TEXT,
    term_collection_id TEXT,
    subject_code TEXT,
    course_number TEXT,
    school_id TEXT,

    max_enrollment INTEGER,
    instruction_method TEXT,
    campus TEXT,
    enrollment INTEGER,
    primary_professor_id TEXT,
    other TEXT, -- This column should contain valid JSON.
    FOREIGN KEY (school_id, subject_code, course_number)
        REFERENCES courses(school_id, subject_code, number),
    FOREIGN KEY (primary_professor_id, school_id) REFERENCES professors(id, school_id),
    FOREIGN KEY (term_collection_id, school_id) REFERENCES term_collections(id, school_id),
    PRIMARY KEY (sequence, term_collection_id, subject_code, course_number, school_id),
    CHECK(other IS NULL OR json_valid(other)) -- Ensures 'other' is valid JSON if not NULL
    -- The PostgreSQL constraint on `sequence` using regex has been removed
    -- and should be handled at the application level.
);
```

**Columns:**
- `sequence` (TEXT): Section sequence/identifier
- `term_collection_id` (TEXT): Reference to the term collection
- `subject_code` (TEXT): Subject code
- `course_number` (TEXT): Course number
- `school_id` (TEXT): Reference to the school
- `max_enrollment` (INTEGER): Maximum enrollment capacity
- `instruction_method` (TEXT): Method of instruction (e.g., "In-Person", "Online")
- `campus` (TEXT): Campus location
- `enrollment` (INTEGER): Current enrollment count
- `primary_professor_id` (TEXT): Reference to the primary professor
- `other` (TEXT): Additional JSON data

**Foreign Keys:**
- `(school_id, subject_code, course_number)` → `courses(school_id, subject_code, number)`
- `(primary_professor_id, school_id)` → `professors(id, school_id)`
- `(term_collection_id, school_id)` → `term_collections(id, school_id)`

**Constraints:**
- Composite PRIMARY KEY: `(sequence, term_collection_id, subject_code, course_number, school_id)`
- `other` must be valid JSON if not NULL

---

### Table: `meeting_times`

Stores meeting time information for sections.

```sql
CREATE TABLE meeting_times (
    sequence INTEGER,
    section_sequence TEXT,
    term_collection_id TEXT,
    subject_code TEXT,
    course_number TEXT,
    school_id TEXT,

    start_date TEXT, -- Replaces TIMESTAMP, store as 'YYYY-MM-DD HH:MM:SS'
    end_date TEXT,   -- Replaces TIMESTAMP, store as 'YYYY-MM-DD HH:MM:SS'
    meeting_type TEXT,
    start_minutes TEXT, -- Replaces TIME, store as 'HH:MM:SS'
    end_minutes TEXT,   -- Replaces TIME, store as 'HH:MM:SS'
    is_monday INTEGER NOT NULL CHECK(is_monday IN (0, 1)),
    is_tuesday INTEGER NOT NULL CHECK(is_tuesday IN (0, 1)),
    is_wednesday INTEGER NOT NULL CHECK(is_wednesday IN (0, 1)),
    is_thursday INTEGER NOT NULL CHECK(is_thursday IN (0, 1)),
    is_friday INTEGER NOT NULL CHECK(is_friday IN (0, 1)),
    is_saturday INTEGER NOT NULL CHECK(is_saturday IN (0, 1)),
    is_sunday INTEGER NOT NULL CHECK(is_sunday IN (0, 1)),
    other TEXT, -- This column should contain valid JSON.

    FOREIGN KEY (section_sequence, term_collection_id, school_id, subject_code, course_number)
        REFERENCES sections(sequence, term_collection_id, school_id, subject_code, course_number) ON DELETE CASCADE,
    PRIMARY KEY (sequence, section_sequence, term_collection_id, subject_code, course_number, school_id),
    CHECK(other IS NULL OR json_valid(other)) -- Ensures 'other' is valid JSON if not NULL
);
```

**Columns:**
- `sequence` (INTEGER): Meeting time sequence number
- `section_sequence` (TEXT): Reference to section sequence
- `term_collection_id` (TEXT): Reference to term collection
- `subject_code` (TEXT): Subject code
- `course_number` (TEXT): Course number
- `school_id` (TEXT): Reference to the school
- `start_date` (TEXT): Start date/time (ISO8601 format: `'YYYY-MM-DD HH:MM:SS'`)
- `end_date` (TEXT): End date/time (ISO8601 format: `'YYYY-MM-DD HH:MM:SS'`)
- `meeting_type` (TEXT): Type of meeting
- `start_minutes` (TEXT): Start time (format: `'HH:MM:SS'`)
- `end_minutes` (TEXT): End time (format: `'HH:MM:SS'`)
- `is_monday` through `is_sunday` (INTEGER, NOT NULL): Boolean flags for each day of the week (0 = false, 1 = true)
- `other` (TEXT): Additional JSON data

**Foreign Keys:**
- `(section_sequence, term_collection_id, school_id, subject_code, course_number)` → `sections(sequence, term_collection_id, school_id, subject_code, course_number)` with `ON DELETE CASCADE`

**Constraints:**
- Composite PRIMARY KEY: `(sequence, section_sequence, term_collection_id, subject_code, course_number, school_id)`
- `other` must be valid JSON if not NULL

---

## Data Synchronization

When Classy Sync synchronizes data for a school, it should:

1. Create a SQLite database file for that school
2. Execute `PRAGMA foreign_keys = ON;` to enable foreign key constraints
3. Create all tables according to this schema
4. Populate the tables with the school's data
5. Ensure all foreign key relationships are valid
6. Ensure all CHECK constraints are satisfied

The resulting database file should be ready for use by ClassQL without requiring any schema modifications.

