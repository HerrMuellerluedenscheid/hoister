CREATE TABLE users (
    id VARCHAR(128) PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE project (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE TABLE host (
    id UUID PRIMARY KEY,
    hostname VARCHAR(253) NOT NULL,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, hostname)
);

CREATE TABLE service (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    image VARCHAR(1024) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, name)
);

CREATE TABLE deployment (
    id UUID PRIMARY KEY,
    digest VARCHAR(255) NOT NULL,
    status SMALLINT NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5, 6)),
    service_id UUID NOT NULL REFERENCES service(id) ON DELETE CASCADE,
    host_id UUID REFERENCES host(id) ON DELETE SET NULL,
    logs TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_token (
    id UUID PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    token_prefix VARCHAR(12) NOT NULL,
    comment VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_token_user_id_idx ON api_token(user_id);

CREATE TABLE notifier (
    id UUID PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind VARCHAR(64) NOT NULL,
    config JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX notifier_user_id_idx ON notifier(user_id);

CREATE TABLE user_plan (
    user_id VARCHAR(128) PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    plan VARCHAR(16) NOT NULL DEFAULT 'free',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE container_state (
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    hostname VARCHAR(253) NOT NULL,
    project_name VARCHAR(255) NOT NULL,
    services JSONB NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, hostname, project_name)
);

CREATE INDEX container_state_user_idx ON container_state(user_id);

CREATE TABLE container_metrics (
    user_id VARCHAR(128) NOT NULL,
    hostname VARCHAR(253) NOT NULL,
    project_name VARCHAR(255) NOT NULL,
    service_name VARCHAR(255) NOT NULL,
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
