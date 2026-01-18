# Natural Language Query Engine for Course Scheduling

A terminal-based interactive natural language query engine for academic course scheduling data, featuring a rich TUI built with Rust and ratatui. 

## Demo
<div align="center">
    <img src="https://github.com/CFdefense/ClassQL/blob/readme/docs/demo.gif" alt="Demo Video" width="750">
</div>

## Overview

ClassQL provides a complete solution for exploring academic course data stored locally in SQLite. Built with accuracy and usability in mind, it features a custom Domain-Specific Language (DSL) that translates natural language queries into efficient SQL, supporting complex boolean logic, time-based filtering all through a beautiful TUI interface

The application supports comprehensive course management features including live search, schedule generation with conflict detection, saved schedule management, and seamless data synchronization with external course databases. With its intuitive TUI interface, ClassQL makes finding and organizing your ideal class schedule effortless.

## Key Features

- **Natural Language Query Engine**
  - Custom DSL parser with lexer, parser, and semantic analyzer
  - Support for boolean logic (AND, OR, NOT)
  - Field-specific queries (professor, subject, course number, etc.)
  - Time-based filtering (before, after, between times)
  - Day-of-week filtering with synonym normalization
  - Tab completion for query suggestions

- **Interactive Terminal User Interface (TUI)**
  - Rich, keyboard-driven interface built with ratatui
  - Live search with real-time result updates
  - Multi-column result display with navigation
  - Detailed class information overlay
  - Context-sensitive help bar
  - Toast notifications for errors and success messages
  - Theme support with customizable color schemes

- **Schedule Management**
  - Automatic schedule generation from cart
  - Conflict detection between overlapping classes
  - Multiple schedule generation with filtering
  - Visual calendar display with time blocks
  - Schedule counter display (Schedule X of Y)
  - PageUp/PageDown navigation through generated schedules
  - Saved schedule persistence with .sav files
  - Load and view saved schedules

- **Data Management**
  - SQLite database for local course storage
  - Integration with classy-sync for data synchronization
  - Support for multiple schools and academic terms
  - Automatic schema migrations
  - Connection pooling for efficient database access
  - Last sync time tracking

- **Query Processing Pipeline**
  - Lexical analysis with token categorization
  - AST generation from parsed queries
  - Semantic analysis with normalization
  - SQL code generation with parameterized queries
  - Error reporting with precise position highlighting

- **Search Capabilities**
  - Case-insensitive string matching
  - Fuzzy matching support
  - Multi-field search across courses, sections, and professors
  - Real-time query execution
  - Result browsing with keyboard navigation
  - Detailed view for selected classes

- **Settings & Configuration**
  - School selection interface
  - Term selection (Spring, Fall, Winter, Summer)
  - Sync configuration management
  - Environment variable support via .env files

## Tech Stack

### Core
- Rust (2021 edition)
- ratatui - Terminal UI library
- crossterm - Cross-platform terminal manipulation
- rusqlite - SQLite database driver

### Dependencies
- clap - Command-line argument parsing
- regex - Pattern matching for query parsing
- serde/serde_json - Serialization support
- petgraph - Graph data structures for AST
- classy-sync - Course data synchronization
- dotenv - Environment variable management
- reqwest - HTTP client for data sync

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git
- SQLite (bundled with rusqlite)

### Installation

1. Clone the repository
```bash
git clone https://github.com/CFdefense/ClassQL.git
cd ClassQL
```

2. Configure environment variables (optional)
```bash
cp .env.example .env
# Edit .env with your sync server configuration
```

3. Build the project
```bash
cargo build --release
```

4. Run the application
```bash
cargo run --release
```

### Usage Examples

**Run in TUI mode:**
```bash
cargo run --release
```

**Compile and visualize AST for a query (Useful for debgging Parser):**
```bash
cargo run --release -- --query "CS courses with professor Smith on Monday"
```

**Sync course data (Useful for testing connection to classy):**
```bash
cargo run --release -- --sync
```

## Data Synchronization with Classy Servers

ClassQL interfaces and connects with [classy](https://github.com/Pjt727/classy) servers to fetch the latest class information. The application uses the [classy-sync](https://github.com/Pjt727/classy-sync) library to synchronize course data from classy servers into your local SQLite database.

### Default School Support

ClassQL comes with built-in support for the following schools by default:
- **Marist College** - Full course catalog support
- **Temple University** - Full course catalog support

### Self-Hosting and Custom Scrapers

You can self-host your own classy server and even write custom web scrapers for your own school! The classy project is designed to be extensible:

1. **Self-Host Classy**: Deploy your own classy server instance to have full control over your course data
2. **Custom Scrapers**: Write web scrapers for your specific school's course registration system
3. **Flexible Configuration**: Configure ClassQL to connect to any classy server via environment variables

To connect ClassQL to your own classy server, set the following environment variables in your `.env` file:
```bash
CLASSY_SERVER_URL=http://your-classy-server.com
CLASSY_SERVER_PORT=8080
```

For more information on setting up and running classy servers, see the [classy repository](https://github.com/Pjt727/classy).

## Query Syntax Examples

**Simple search:**
```
Subject is CS
```

**Professor search:**
```
professor contains Smith
```

**Time-based filtering:**
```
start < 12:00pm and monday
```

**Complex queries:**
```
course is CS and enrollment < 30 and credit hours = 3
```

**Day filtering:**
```
Monday Wednesday Friday
```

**Boolean logic:**
```
sub is (CS or MATH) and prof contains alan
```

## File Structure

```
ClassQL/
├── src/
│   ├── data/                  # Data Management Modules
│   │   ├── mod.rs             # Module declarations
│   │   ├── pool.rs            # Database connection pooling
│   │   ├── sql.rs             # SQL query functions
│   │   └── sync.rs            # Data synchronization
│   ├── data_stores/           # Database Storage
│   │   └── sqlite/            # SQLite implementation
│   │       └── migrations/    # Database schema migrations
│   ├── dsl/                   # Domain-Specific Language
│   │   ├── codegen.rs         # SQL code generation
│   │   ├── compiler.rs        # Main compiler interface
│   │   ├── lexer.rs           # Lexical analysis
│   │   ├── parser.rs          # AST parsing
│   │   ├── semantic.rs        # Semantic analysis
│   │   └── token.rs           # Token definitions
│   ├── tui/                   # Terminal User Interface
│   │   ├── app.rs             # Main TUI application
│   │   ├── errors.rs          # Error types
│   │   ├── save.rs            # Schedule persistence
│   │   ├── state.rs           # Application state
│   │   ├── themes.rs          # Color themes
│   │   └── widgets/           # UI Widgets
│   │       ├── completion.rs  # Tab completion dropdown
│   │       ├── detail_view.rs # Class detail overlay
│   │       ├── helpers.rs     # Helper functions
│   │       ├── logo.rs        # ASCII art logo
│   │       ├── menu.rs        # Main menu widget
│   │       ├── query_guide.rs # Query syntax guide
│   │       ├── results.rs     # Query results display
│   │       ├── schedule.rs    # Schedule generation & display
│   │       ├── search_bar.rs  # Search input widget
│   │       ├── settings.rs    # Settings interface
│   │       └── toast.rs       # Notification widget
│   ├── debug_utils/           # Development Tools
│   │   ├── visualizetree.rs   # AST visualization
│   │   └── tree-viz.sh        # Tree visualization script
│   ├── lib.rs                 # Library root
│   └── main.rs                # Application entry point
├── docs/                      # Documentation
│   ├── abstract.md            # Project abstract
│   ├── design.md              # Technical design document
│   ├── grammar.md             # Query grammar specification
│   └── schema.md              # Database schema documentation
├── tests/                     # Test Suite
│   ├── codegen/               # Code generation tests
│   ├── lexer/                 # Lexer tests
│   ├── parser/                # Parser tests
│   ├── query/                 # Query execution tests
│   └── semantic/              # Semantic analysis tests
├── classy/                    # Local Database Storage
│   ├── classes.db             # Main course database
│   └── test.db                # Test database
├── save/                      # Saved Schedules
│   └── *.sav                  # Schedule save files
└── Cargo.toml                 # Rust project configuration
```

## Development Features

- Comprehensive test suite with JSON test cases
- AST visualization for query debugging
- Detailed error reporting with position highlighting
- Query guide integrated into TUI
- Debug mode for query compilation
- Modular widget architecture
- Type-safe database interactions

## Database Schema

The application uses SQLite with the following main tables:
- **schools** - School information
- **terms** - Academic terms (year, season)
- **term_collections** - Term metadata
- **professors** - Instructor information
- **courses** - Course catalog data
- **sections** - Individual course sections
- **meeting_times** - Class meeting schedules

See `docs/schema.md` for complete schema documentation.

## Query Grammar

ClassQL supports a rich query syntax with:
- Field-specific queries (`professor:`, `subject:`, `course:`)
- Condition operators (`is`, `equals`, `contains`, `has`)
- Binary operators (`=`, `!=`, `<`, `>`, `<=`, `>=`)
- Boolean logic (`and`, `or`, `not`)
- Time expressions (`before`, `after`, `between`)
- Day expressions (`Monday`, `Tuesday`, etc. with synonyms)

See `docs/grammar.md` for complete grammar specification.

## References

- [ratatui Documentation](https://ratatui.rs/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [classy](https://github.com/Pjt727/classy) - Course data server and web scraper framework
- [classy-sync](https://github.com/Pjt727/classy-sync) - Data synchronization library for classy servers

## License

MIT License
