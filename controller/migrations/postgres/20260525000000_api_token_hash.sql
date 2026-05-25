-- Replace plaintext agent tokens with their SHA-256 hash.
-- Existing rows are dropped: tokens cannot be recovered to compute their
-- hash, and no production data exists yet.
DROP TABLE IF EXISTS api_token;

CREATE TABLE api_token (
    id BIGSERIAL PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
