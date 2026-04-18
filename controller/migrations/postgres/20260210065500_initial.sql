CREATE TABLE IF NOT EXISTS project (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS service (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES project(id),
    name TEXT NOT NULL,
    image TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, name)
);

CREATE TABLE IF NOT EXISTS deployment (
    id BIGSERIAL PRIMARY KEY,
    digest TEXT NOT NULL,
    status SMALLINT NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5, 6)),
    service_id BIGINT NOT NULL REFERENCES service(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
