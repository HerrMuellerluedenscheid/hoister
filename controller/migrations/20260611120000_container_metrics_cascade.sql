-- Tie container_metrics to its owning container_state row so deleting a
-- project removes its time series automatically (ON DELETE CASCADE) instead of
-- relying on a second application-level delete.
--
-- SQLite cannot add a foreign key to an existing table, so the table is
-- rebuilt. Pre-existing samples whose (user, host, project) has no
-- container_state row are dropped during the copy — under the new foreign key
-- such orphans can no longer exist, and metrics ingestion already skips any
-- sample that has no parent state row.

CREATE TABLE container_metrics_new (
    user_id         TEXT    NOT NULL,
    hostname        TEXT    NOT NULL,
    project_name    TEXT    NOT NULL,
    service_name    TEXT    NOT NULL,
    recorded_at     TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cpu_pct         REAL    NOT NULL,
    mem_bytes       INTEGER NOT NULL,
    mem_limit_bytes INTEGER NOT NULL,
    FOREIGN KEY (user_id, hostname, project_name)
        REFERENCES container_state (user_id, hostname, project_name)
        ON DELETE CASCADE
);

INSERT INTO container_metrics_new
    (user_id, hostname, project_name, service_name, recorded_at,
     cpu_pct, mem_bytes, mem_limit_bytes)
SELECT m.user_id, m.hostname, m.project_name, m.service_name, m.recorded_at,
       m.cpu_pct, m.mem_bytes, m.mem_limit_bytes
FROM container_metrics m
WHERE EXISTS (
    SELECT 1 FROM container_state s
    WHERE s.user_id = m.user_id
      AND s.hostname = m.hostname
      AND s.project_name = m.project_name
);

DROP TABLE container_metrics;
ALTER TABLE container_metrics_new RENAME TO container_metrics;

CREATE INDEX IF NOT EXISTS container_metrics_lookup_idx
    ON container_metrics(user_id, hostname, project_name, service_name, recorded_at);
