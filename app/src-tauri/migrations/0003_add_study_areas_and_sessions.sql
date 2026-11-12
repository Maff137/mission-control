CREATE TABLE study_areas (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
    color TEXT NOT NULL CHECK (
        length(color) = 7
        AND substr(color, 1, 1) = '#'
        AND substr(color, 2) NOT GLOB '*[^0-9A-Fa-f]*'
    ),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE study_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    area_id TEXT NOT NULL REFERENCES study_areas(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    duration_minutes INTEGER NOT NULL CHECK (typeof(duration_minutes) = 'integer' AND duration_minutes > 0),
    study_date TEXT NOT NULL CHECK (
        length(study_date) = 10
        AND study_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
    ),
    notes TEXT NOT NULL DEFAULT '' CHECK (length(notes) <= 10000),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX study_sessions_history_idx
    ON study_sessions(study_date DESC, created_at DESC, id DESC);
