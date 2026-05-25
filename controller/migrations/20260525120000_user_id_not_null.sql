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

-- SQLite doesn't support ALTER COLUMN; rebuild the tables.
CREATE TABLE project_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO project_new (id, name, user_id, created_at)
    SELECT id, name, user_id, created_at FROM project;
DROP TABLE project;
ALTER TABLE project_new RENAME TO project;

CREATE TABLE host_new (
    id BLOB PRIMARY KEY,
    hostname TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL
);
INSERT INTO host_new (id, hostname, user_id)
    SELECT id, hostname, user_id FROM host;
DROP TABLE host;
ALTER TABLE host_new RENAME TO host;
