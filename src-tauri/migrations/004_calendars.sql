CREATE TABLE calendar_sources (
    id TEXT PRIMARY KEY NOT NULL,
    payload TEXT NOT NULL,
    ics TEXT NOT NULL
);
PRAGMA user_version = 4;
