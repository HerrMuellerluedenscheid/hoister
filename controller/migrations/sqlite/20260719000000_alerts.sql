-- Metric alert rules configured through the cloud dashboard, evaluated by the
-- controller on every ingested metrics batch. Scope columns are NULL for
-- "any": a rule may target everything, one host, one project, or one service.

CREATE TABLE alert_rule (
    id TEXT PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    metric VARCHAR(32) NOT NULL,
    threshold REAL NOT NULL,
    for_seconds INTEGER NOT NULL,
    cooldown_seconds INTEGER NOT NULL DEFAULT 0,
    hostname VARCHAR(253),
    project VARCHAR(255),
    service VARCHAR(255),
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX alert_rule_user_id_idx ON alert_rule(user_id);

-- Per-(rule, container) firing state, persisted so a controller restart keeps
-- cooldowns and pending windows instead of re-notifying every firing alert.
-- Keyed by names rather than service ids: state must survive a service row
-- being recreated on redeploy. `state` is the JSON-serialized
-- hoister_shared::alerts::AlertState.
CREATE TABLE alert_state (
    rule_id TEXT NOT NULL REFERENCES alert_rule(id) ON DELETE CASCADE,
    hostname VARCHAR(253) NOT NULL,
    project VARCHAR(255) NOT NULL,
    service VARCHAR(255) NOT NULL,
    state TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (rule_id, hostname, project, service)
);
