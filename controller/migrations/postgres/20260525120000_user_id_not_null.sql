-- Enforce NOT NULL on tenant ownership columns. If a code path ever forgets
-- to set user_id, the INSERT must error rather than silently creating an
-- orphan row visible to every "all-tenant" query.
-- Existing rows with NULL user_id are dropped: no production data yet, and
-- a NULL user_id already cannot be re-attributed to a real user.
DELETE FROM deployment WHERE service_id IN (
    SELECT id FROM service WHERE project_id IN (
        SELECT id FROM project WHERE user_id IS NULL
    )
);
DELETE FROM service WHERE project_id IN (
    SELECT id FROM project WHERE user_id IS NULL
);
DELETE FROM project WHERE user_id IS NULL;
DELETE FROM host WHERE user_id IS NULL;

ALTER TABLE project ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE host ALTER COLUMN user_id SET NOT NULL;
