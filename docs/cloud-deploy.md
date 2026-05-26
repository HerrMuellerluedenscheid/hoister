# Cloud deployment — VPS bring-up

Operator runbook for deploying the hoister.io stack on a single VPS. The
stack is four containers behind Caddy: `hoister-controller`,
`hoister-frontend-cloud`, `backend-db` (Postgres 18), and `caddy` itself.

Read [`docs/cloud-rollout.md`](./cloud-rollout.md) first — it captures the
decisions baked into this stack (single-VPS, AGPL controller, opt-in log
forwarding, MIT agent, etc.).

## Prerequisites

- A VPS (Hetzner CX22 or similar — 2 vCPU, 4 GB RAM is plenty for the
  starting load).
- Debian 12 / Ubuntu 24.04 with `docker` and the `docker compose` plugin.
- Two DNS A/AAAA records pointing at the VPS:
  - `hoister.io` → dashboard
  - `api.hoister.io` → agent ingress
- A Clerk application configured at <https://dashboard.clerk.com>, with the
  redirect URL pointed at `https://hoister.io/dashboard`.
- Ports `80` and `443` reachable from the public internet (Caddy
  auto-provisions Let's Encrypt certificates on first start).

## Layout

```
/opt/hoister/
├── docker-compose.cloud.yaml      # checked in; copied from this repo
├── deploy/
│   ├── Caddyfile                  # checked in
│   └── backup.sh                  # checked in
/etc/hoister/
├── cloud.env                      # NOT in git — secrets live here
└── backup.passphrase              # NOT in git — mode 0600
/var/backups/hoister/              # nightly encrypted dumps
```

## One-time setup

```bash
# 1. Pull the repo (only the files listed above are used at runtime).
sudo mkdir -p /opt/hoister
sudo git clone https://github.com/HerrMuellerluedenscheid/hoister.git /opt/hoister
cd /opt/hoister

# 2. Create the deployment env file.
sudo install -d -m 0750 /etc/hoister
sudo install -m 0640 /dev/null /etc/hoister/cloud.env
sudoedit /etc/hoister/cloud.env
```

Populate `cloud.env`:

```env
HOISTER_CLOUD_DOMAIN=hoister.io
HOISTER_CLOUD_API_DOMAIN=api.hoister.io
HOISTER_CLOUD_LE_EMAIL=ops@hoister.io

POSTGRES_USER=hoister
POSTGRES_PASSWORD=<64-char-random>        # openssl rand -base64 48
POSTGRES_DB=hoister

PUBLIC_CLERK_PUBLISHABLE_KEY=pk_live_...
CLERK_SECRET_KEY=sk_live_...
CLERK_WEBHOOK_SECRET=whsec_...
```

```bash
# 3. Create the backup passphrase.
sudo install -m 0600 /dev/null /etc/hoister/backup.passphrase
openssl rand -base64 48 | sudo tee /etc/hoister/backup.passphrase > /dev/null

# 4. Boot the stack.
cd /opt/hoister
sudo docker compose --env-file /etc/hoister/cloud.env -f docker-compose.cloud.yaml up -d
```

Caddy will take 10-30 s on first boot to obtain certificates. Watch with:

```bash
sudo docker compose -f docker-compose.cloud.yaml logs -f caddy
```

Once `certificate obtained successfully` appears for both domains, the
dashboard is live at `https://hoister.io`.

## Bind layout (security summary)

| Service | Container port | Published? | Reachable from |
|---|---|---|---|
| caddy | 80, 443 | Yes | Public internet |
| hoister-controller | 3033 (agent) | No (only `expose`) | Caddy → docker bridge |
| hoister-controller | 3034 (internal) | No (only `expose`) | hoister-frontend-cloud → docker bridge |
| hoister-frontend-cloud | 3000 | No (only `expose`) | Caddy → docker bridge |
| backend-db | 5432 | No (only `expose`) | hoister-controller → docker bridge |

Nothing other than `:80` and `:443` is reachable from outside the VPS. The
controller's internal port (`3034`) is on the docker bridge network; the
host firewall plays no part. If the VPS provides a public IP and a private
network, set the firewall to drop everything except 80/443 on the public
interface as defense in depth.

## Backups

```bash
# 5. Wire the daily backup.
sudo crontab -e
```

Add:

```cron
17 2 * * * /opt/hoister/deploy/backup.sh >> /var/log/hoister-backup.log 2>&1
```

`backup.sh` runs `pg_dump` inside the running `backend-db` container, pipes
through `gpg --symmetric --cipher-algo AES256`, writes to
`/var/backups/hoister/`, and prunes anything older than 30 days. See the
header of [`deploy/backup.sh`](../deploy/backup.sh) for the restore
command.

Recommend rsyncing `/var/backups/hoister/` off-host nightly — a backup on
the same machine doesn't survive a full-host loss.

## Updates

The stack uses the `:latest` tag for each image. To pick up a new release:

```bash
cd /opt/hoister
sudo git pull
sudo docker compose --env-file /etc/hoister/cloud.env -f docker-compose.cloud.yaml pull
sudo docker compose --env-file /etc/hoister/cloud.env -f docker-compose.cloud.yaml up -d
```

Migrations run automatically on controller start.

## Day-2 checks

- Audit log: `docker compose -f docker-compose.cloud.yaml logs hoister-controller | grep "hoister.audit"` — each request line carries `user=…`.
- Token rotation works from the dashboard (`Rotate token` button on the
  "Agent token issued" row).
- Container detail page shows last-update times — anything older than 1 min
  surfaces a "Stale data" banner.
- `/datenschutz` is reachable without authentication; the Clerk app
  publishes a fresh `__session` cookie on login.

## What is _not_ covered yet

- Off-host backup rotation. Use rsync to a separate machine until we add it
  to `backup.sh`.
- Container health checks for the compose services — would be a small
  follow-up.
- Centralised logging — `docker logs` only, today.
- See `docs/next-steps.md` and the open security issues
  (`gh issue list --label security`) for the remaining backlog.
