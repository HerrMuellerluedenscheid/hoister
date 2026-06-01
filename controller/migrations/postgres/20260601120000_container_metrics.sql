-- Append-only time series of per-container resource usage. One row per
-- (user, host, project, service) sample, written each time an agent that has
-- opted into metrics reports (~once a minute). Rows older than the retention
-- window are pruned opportunistically on insert.
CREATE TABLE IF NOT EXISTS container_metrics (
    user_id         TEXT             NOT NULL,
    hostname        TEXT             NOT NULL,
    project_name    TEXT             NOT NULL,
    service_name    TEXT             NOT NULL,
    recorded_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    cpu_pct         DOUBLE PRECISION NOT NULL,
    mem_bytes       BIGINT           NOT NULL,
    mem_limit_bytes BIGINT           NOT NULL
);

CREATE INDEX IF NOT EXISTS container_metrics_lookup_idx
    ON container_metrics(user_id, hostname, project_name, service_name, recorded_at);
