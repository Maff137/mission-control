CREATE TABLE persistence_test_records (
    id TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL CHECK (length(trim(value, char(9,10,11,12,13,32))) > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
