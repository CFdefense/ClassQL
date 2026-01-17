CREATE TABLE _previous_all_collections (
    synced_at INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE _previous_term_collections (
    synced_at INTEGER NOT NULL,
    school_id TEXT NOT NULL,
    term_collection_id TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE _previous_school_collections (
    synced_at INTEGER NOT NULL,
    school_id TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL
);


CREATE TABLE _school_strategies (
    school_id TEXT NOT NULL,
    term_collection_id TEXT,
    UNIQUE(school_id, term_collection_id)
);
