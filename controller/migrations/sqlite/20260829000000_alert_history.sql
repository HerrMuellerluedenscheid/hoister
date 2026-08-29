-- History of alert transitions (fired / still firing / resolved), so the
-- dashboard can show what triggered when and on which system. The rule's
-- condition and target are denormalised into every row: rules get edited and
-- deleted, and a history entry has to keep reading correctly afterwards.

CREATE TABLE alert_event (
    id TEXT PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- NULL once the rule is gone: deleting a rule must not erase its history.
    rule_id TEXT REFERENCES alert_rule(id) ON DELETE SET NULL,
    kind VARCHAR(16) NOT NULL,
    metric VARCHAR(32) NOT NULL,
    threshold REAL NOT NULL,
    for_seconds INTEGER NOT NULL,
    value REAL NOT NULL,
    hostname VARCHAR(253) NOT NULL,
    project VARCHAR(255) NOT NULL,
    service VARCHAR(255) NOT NULL,
    triggered_at TEXT NOT NULL
);

CREATE INDEX alert_event_user_time_idx ON alert_event(user_id, triggered_at DESC);

-- Watermark behind the dashboard's "new since your last login" highlight.
-- `session_started_at` is when the current login first opened the alert
-- history; `previous_session_started_at` is the same for the login before it
-- and is what events are compared against. Rotating only when `session_id`
-- changes keeps the highlight stable while the user navigates around.
CREATE TABLE alert_seen (
    user_id VARCHAR(128) PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(128) NOT NULL,
    session_started_at TEXT NOT NULL,
    previous_session_started_at TEXT
);
