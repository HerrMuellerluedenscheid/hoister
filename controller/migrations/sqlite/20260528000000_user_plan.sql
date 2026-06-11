-- Per-user billing tier. Rows are created lazily — a missing row is
-- treated as the Free plan by the application layer. Stripe wiring will
-- write to this table via the Clerk webhook in a later iteration.
CREATE TABLE IF NOT EXISTS user_plan (
    user_id    TEXT PRIMARY KEY,
    plan       TEXT NOT NULL DEFAULT 'free',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
