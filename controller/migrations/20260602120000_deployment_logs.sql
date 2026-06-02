-- Redacted log tail of the failed container, captured by the agent on
-- rollback/failure so the deployments dashboard can show why an update was
-- rolled back. NULL for successful deployments and agents without log
-- forwarding enabled.
ALTER TABLE deployment ADD COLUMN logs TEXT;
