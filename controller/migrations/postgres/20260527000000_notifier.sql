-- Per-user notifier configurations. `config` is a JSONB blob whose shape
-- depends on `kind` (slack / telegram / discord / gotify / email).
CREATE TABLE IF NOT EXISTS notifier (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    config JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS notifier_user_id_idx ON notifier(user_id);
