-- Per-user notifier configurations. `config` is a JSON blob whose shape
-- depends on `kind` (slack / telegram / discord / gotify / email).
CREATE TABLE IF NOT EXISTS notifier (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    config TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS notifier_user_id_idx ON notifier(user_id);
