-- Tie container_metrics to its owning container_state row so deleting a
-- project removes its time series automatically (ON DELETE CASCADE) instead of
-- relying on a second application-level delete.
--
-- Drop any pre-existing samples whose (user, host, project) has no
-- container_state row before adding the constraint — under the new foreign key
-- such orphans can no longer exist, and metrics ingestion already skips any
-- sample that has no parent state row.

DELETE FROM container_metrics
WHERE NOT EXISTS (
    SELECT 1 FROM container_state s
    WHERE s.user_id = container_metrics.user_id
      AND s.hostname = container_metrics.hostname
      AND s.project_name = container_metrics.project_name
);

ALTER TABLE container_metrics
    ADD CONSTRAINT container_metrics_state_fkey
    FOREIGN KEY (user_id, hostname, project_name)
    REFERENCES container_state (user_id, hostname, project_name)
    ON DELETE CASCADE;
