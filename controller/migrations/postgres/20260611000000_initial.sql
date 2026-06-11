CREATE TABLE project (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE host (
    id BYTEA PRIMARY KEY,
    hostname TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL
);

CREATE TABLE service (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES project(id),
    name TEXT NOT NULL,
    image TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, name)
);

CREATE TABLE deployment (
    id BIGSERIAL PRIMARY KEY,
    digest TEXT NOT NULL,
    status SMALLINT NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5, 6)),
    service_id BIGINT NOT NULL REFERENCES service(id),
    host_id BYTEA REFERENCES host(id),
    logs TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_token (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_prefix TEXT NOT NULL,
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_token_user_id_idx ON api_token(user_id);

CREATE TABLE notifier (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    config JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX notifier_user_id_idx ON notifier(user_id);

CREATE TABLE user_plan (
    user_id TEXT PRIMARY KEY,
    plan TEXT NOT NULL DEFAULT 'free',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE container_state (
    user_id TEXT NOT NULL,
    hostname TEXT NOT NULL,
    project_name TEXT NOT NULL,
    services JSONB NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, hostname, project_name)
);

CREATE INDEX container_state_user_idx ON container_state(user_id);

CREATE TABLE container_metrics (
    user_id TEXT NOT NULL,
    hostname TEXT NOT NULL,
    project_name TEXT NOT NULL,
    service_name TEXT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_pct DOUBLE PRECISION NOT NULL,
    mem_bytes BIGINT NOT NULL,
    mem_limit_bytes BIGINT NOT NULL,
    FOREIGN KEY (user_id, hostname, project_name)
        REFERENCES container_state (user_id, hostname, project_name)
        ON DELETE CASCADE
);

CREATE INDEX container_metrics_lookup_idx
    ON container_metrics(user_id, hostname, project_name, service_name, recorded_at);
