#!/usr/bin/env bash
# Daily encrypted Postgres backup for the hoister.io stack.
#
# Runs `pg_dump` inside the running `backend-db` container, pipes the dump
# through gpg --symmetric (AES-256), writes to BACKUP_DIR with a timestamped
# filename, then deletes anything older than RETENTION_DAYS.
#
# Recommended cron entry (UTC, run as root or a dedicated backup user):
#   17 2 * * * /opt/hoister/deploy/backup.sh >> /var/log/hoister-backup.log 2>&1
#
# Required env (defaults in parentheses):
#   BACKUP_DIR        (/var/backups/hoister)
#   BACKUP_PASSPHRASE_FILE  (/etc/hoister/backup.passphrase)
#   RETENTION_DAYS    (30)
#   COMPOSE_FILE      (/opt/hoister/docker-compose.cloud.yaml)
#   POSTGRES_USER     (read from /etc/hoister/cloud.env)
#   POSTGRES_DB       (read from /etc/hoister/cloud.env)
#
# The passphrase file must be mode 0600 and owned by the cron user.
# Restore: `gpg --decrypt <file>.sql.gpg | psql -h ... -U ... -d ...`

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/var/backups/hoister}"
BACKUP_PASSPHRASE_FILE="${BACKUP_PASSPHRASE_FILE:-/etc/hoister/backup.passphrase}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
COMPOSE_FILE="${COMPOSE_FILE:-/opt/hoister/docker-compose.cloud.yaml}"

# Sanity checks before we touch anything.
[[ -r "$BACKUP_PASSPHRASE_FILE" ]] || {
    echo "FATAL: cannot read passphrase file $BACKUP_PASSPHRASE_FILE" >&2
    exit 2
}
[[ -d "$BACKUP_DIR" ]] || mkdir -p "$BACKUP_DIR"
[[ -f "$COMPOSE_FILE" ]] || {
    echo "FATAL: compose file $COMPOSE_FILE not found" >&2
    exit 2
}

# Source the deployment env so we know which DB / user to dump.
if [[ -r /etc/hoister/cloud.env ]]; then
    # shellcheck disable=SC1091
    set -a; source /etc/hoister/cloud.env; set +a
fi

: "${POSTGRES_USER:?POSTGRES_USER is required (cloud.env or env)}"
: "${POSTGRES_DB:?POSTGRES_DB is required (cloud.env or env)}"

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
TARGET="$BACKUP_DIR/hoister-$TIMESTAMP.sql.gpg"

echo "[$(date -u +%FT%TZ)] Starting backup → $TARGET"

# Run pg_dump inside the postgres container, stream stdout straight to gpg.
# --clean --if-exists lets the dump be replayed into a fresh database.
docker compose -f "$COMPOSE_FILE" exec -T backend-db \
    pg_dump -U "$POSTGRES_USER" --clean --if-exists "$POSTGRES_DB" |
    gpg --batch --yes --symmetric --cipher-algo AES256 \
        --passphrase-file "$BACKUP_PASSPHRASE_FILE" \
        --output "$TARGET"

SIZE="$(du -h "$TARGET" | cut -f1)"
echo "[$(date -u +%FT%TZ)] Wrote $TARGET ($SIZE)"

# Retention sweep. -mtime is in 24h units; +N matches strictly older than N days.
DELETED="$(find "$BACKUP_DIR" -maxdepth 1 -name 'hoister-*.sql.gpg' -mtime "+$RETENTION_DAYS" -print -delete | wc -l | tr -d ' ')"
if [[ "$DELETED" -gt 0 ]]; then
    echo "[$(date -u +%FT%TZ)] Pruned $DELETED backup(s) older than ${RETENTION_DAYS}d"
fi

echo "[$(date -u +%FT%TZ)] Backup complete"
