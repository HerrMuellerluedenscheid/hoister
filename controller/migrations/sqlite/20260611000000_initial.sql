CREATE TABLE project (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE host (
    id BLOB PRIMARY KEY,
    hostname TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL
);

CREATE TABLE service (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES project(id),
    name TEXT NOT NULL,
    image TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, name)
);

CREATE TABLE deployment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    digest TEXT NOT NULL,
    status INTEGER NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5, 6)),
    service_id INTEGER NOT NULL REFERENCES service(id),
    host_id BLOB REFERENCES host(id),
    logs TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE api_token (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_prefix TEXT NOT NULL,
    comment TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX api_token_user_id_idx ON api_token(user_id);

CREATE TABLE notifier (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    config TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX notifier_user_id_idx ON notifier(user_id);

CREATE TABLE user_plan (
    user_id TEXT PRIMARY KEY,
    plan TEXT NOT NULL DEFAULT 'free',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE container_state (
    user_id TEXT NOT NULL,
    hostname TEXT NOT NULL,
    project_name TEXT NOT NULL,
    services TEXT NOT NULL,
    last_updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, hostname, project_name)
);

CREATE INDEX container_state_user_idx ON container_state(user_id);

CREATE TABLE container_metrics (
    user_id TEXT NOT NULL,
    hostname TEXT NOT NULL,
    project_name TEXT NOT NULL,
    service_name TEXT NOT NULL,
    recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cpu_pct REAL NOT NULL,
    mem_bytes INTEGER NOT NULL,
    mem_limit_bytes INTEGER NOT NULL,
    FOREIGN KEY (user_id, hostname, project_name)
        REFERENCES container_state (user_id, hostname, project_name)
        ON DELETE CASCADE
);

CREATE INDEX container_metrics_lookup_idx
    ON container_metrics(user_id, hostname, project_name, service_name, recorded_at);
