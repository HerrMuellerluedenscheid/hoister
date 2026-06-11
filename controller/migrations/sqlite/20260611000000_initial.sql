-- SQLite ignores VARCHAR(n) length constraints (all text is stored at full
-- length), but the limits here document intent and are enforced in Postgres.

CREATE TABLE users (
    id VARCHAR(128) PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE host (
    id TEXT PRIMARY KEY,
    hostname VARCHAR(253) NOT NULL,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, hostname)
);

CREATE TABLE project (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    host_id TEXT NOT NULL REFERENCES host(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, name)
);

CREATE TABLE service (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    image VARCHAR(1024) NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, name)
);

CREATE TABLE deployment (
    id TEXT PRIMARY KEY,
    digest VARCHAR(255) NOT NULL,
    status INTEGER NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5, 6)),
    service_id TEXT NOT NULL REFERENCES service(id) ON DELETE CASCADE,
    host_id TEXT REFERENCES host(id) ON DELETE SET NULL,
    logs TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE api_token (
    id TEXT PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    token_prefix VARCHAR(12) NOT NULL,
    comment VARCHAR(255),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX api_token_user_id_idx ON api_token(user_id);

CREATE TABLE notifier (
    id TEXT PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind VARCHAR(64) NOT NULL,
    config TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX notifier_user_id_idx ON notifier(user_id);

CREATE TABLE user_plan (
    user_id VARCHAR(128) PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    plan VARCHAR(16) NOT NULL DEFAULT 'free',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE compose_state (
    project_id TEXT NOT NULL PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
    services TEXT NOT NULL,
    last_updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE service_metrics (
    service_id TEXT NOT NULL REFERENCES service(id) ON DELETE CASCADE,
    recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cpu_pct REAL NOT NULL,
    mem_bytes INTEGER NOT NULL,
    mem_limit_bytes INTEGER NOT NULL,
    PRIMARY KEY (service_id, recorded_at)
);
