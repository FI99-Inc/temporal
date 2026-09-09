CREATE TABLE trace_source_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    payload TEXT NOT NULL
);
CREATE TABLE trace_tasks (
    external_id TEXT PRIMARY KEY NOT NULL,
    canonical_id TEXT UNIQUE NOT NULL,
    source_row TEXT NOT NULL,
    normalized TEXT NOT NULL
);
PRAGMA user_version = 1;
