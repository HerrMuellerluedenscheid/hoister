CREATE TABLE IF NOT EXISTS host (
    id BLOB PRIMARY KEY,   -- uuid
    hostname TEXT NOT NULL UNIQUE
);

ALTER TABLE deployment ADD COLUMN host_id BLOB REFERENCES host(id);
