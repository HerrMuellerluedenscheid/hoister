-- Move from one-token-per-user to many-tokens-per-user with comments.
-- Old hashes are still valid HMAC hashes; we just drop the UNIQUE(user_id)
-- constraint and add token_prefix + comment columns. SQLite can't ALTER the
-- constraint in place, so rebuild.
DROP TABLE IF EXISTS api_token;

CREATE TABLE api_token (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_prefix TEXT NOT NULL,
    comment TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS api_token_user_id_idx ON api_token(user_id);
