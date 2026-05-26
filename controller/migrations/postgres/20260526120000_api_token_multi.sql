-- Move from one-token-per-user to many-tokens-per-user with comments.
-- Postgres can drop the UNIQUE constraint and add columns in place.
DROP TABLE IF EXISTS api_token;

CREATE TABLE api_token (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_prefix TEXT NOT NULL,
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS api_token_user_id_idx ON api_token(user_id);
