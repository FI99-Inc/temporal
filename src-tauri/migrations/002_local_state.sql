CREATE TABLE local_temporal_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    payload TEXT NOT NULL
);
PRAGMA user_version = 2;
