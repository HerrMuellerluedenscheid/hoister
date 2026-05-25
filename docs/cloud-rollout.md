# Cloud rollout plan

Plan for turning hoister into a hosted SaaS at **hoister.io** while keeping the
self-hosted path working. Internal planning doc — not user-facing
documentation.

---

## Critical — must fix before publishing to GitHub

### Committed secrets in `docker-compose.cloud.yaml` (commit `bf8b0d3`)

```yaml
HOISTER_API_SECRET: "hst_efec3e248eae4b168c7acfd9d1895437"
POSTGRES_PASSWORD: happyhoistering43123
```

Confirmed dev-only throwaways — no production rotation needed. Still scrub
from history before publish: the values look real, will trip secret scanners,
and set a bad example for contributors copying the compose file. Replace
with `${HOISTER_API_SECRET}` / `${POSTGRES_PASSWORD}` placeholders and use
`git filter-repo` to rewrite history.

### Other secret-hygiene gaps

- `frontend-cloud/.env` holds live `sk_test_…` Clerk keys and a Clerk webhook
  secret. It's gitignored, so safe in the repo — but no `.env.example` exists
  yet for contributors.
- Root `.env` (gitignored) contains a Slack webhook URL and a GitHub PAT. Not
  a publish blocker, but should be sanitised before any backup or
  screen-share.
- Add `.env.example` for root, `frontend/`, and `frontend-cloud/`.
- Add a `gitleaks` (or `git-secrets`) pre-commit hook.
- Audit migrations and Dockerfiles for stray credentials before publish.

---

## Architectural picture

### Three operating modes, one agent binary

| Mode | Controller URL | Token | Controller DB | Use case |
|---|---|---|---|---|
| **Hosted** (new default) | `https://api.hoister.io` | `hst_<user>` | Postgres | User only runs the agent |
| **Self-hosted** | user's URL | static secret OR `hst_` | SQLite | Current `docker-compose.yaml` setup |
| **Standalone** | none | — | — | Just auto-update, no dashboard |

All three modes stay supported — standalone is the agent's original use case
and we keep it. One image (`emrius11/hoister:latest` — see below) decides
mode from environment. The existing controller `self-hosted` cargo feature
already gates the static API-secret auth path.

### Multi-tenant isolation — biggest gap today

The agent → controller path correctly resolves `hst_*` → `user_id` inside
`agent_auth_middleware`. **After** that, no scoping exists:

- `StateMemory` is keyed by `(HostName, ProjectName, ServiceName)` only — two
  users with `project=educk-rs, service=educk-api` would collide.
- `PendingUpdatesMemory` has the same shape.
- `GET /container/state` returns the entire global map — currently exposes
  everyone's data to anyone with internal-network access.
- `Deployment` rows carry `user_id` via `CreateDeploymentRequest.user_id`,
  but every read path needs an audit to confirm it filters on it.

Mandatory before letting external users point an agent at hoister.io:

1. Make `UserId` the top-level key in `StateMemory` and `PendingUpdatesMemory`
   (`HashMap<UserId, HashMap<HostName, …>>`).
2. Every `get_*` handler extracts `UserId` from the request extension and
   filters on it.
3. Hash agent tokens in the DB. Today `api_token.token` is plain text — a DB
   dump leaks every token. Standard fix: store `SHA-256(token)`, show the
   full token once at creation (the dashboard already only displays the
   compose snippet once on first login).

### Data-collection consent

The agent ships full container inspect payloads + log tails to hoister.io.
Even with the existing env-var key/value redaction, the controller still
sees:

- container image names (may reveal private repos)
- env-var **keys** (e.g. `INTERNAL_BILLING_API_HOST`)
- mount source paths (host filesystem layout)
- network IPs (may be internal ranges)
- command-line args and labels

For a hosted service this is a privacy contract. Plan:

- Document exactly what's collected in `/datenschutz` (route exists, empty).
- Log forwarding is **opt-in**: `HOISTER_REPORT_LOGS=true` to enable; off by
  default in all modes.
- Env reporting stays on but values are already redacted by
  `redact_credentials`. Optional `HOISTER_REPORT_ENV=none` for users who
  don't want env keys to leave their host either.
- Cap payload sizes (log tail already 16 KB; cap inspect too).

---

## Phased rollout

### Phase 0 — Pre-publish hygiene (blocks GitHub publish)

1. Replace the dev-throwaway `hst_` and postgres values in
   `docker-compose.cloud.yaml` with `${VAR}` placeholders; `git filter-repo`
   to scrub the literal values from history.
2. Add `.env.example` for root, `frontend/`, `frontend-cloud/`.
3. Add `gitleaks` pre-commit hook; audit existing tree for stray secrets.
4. Hash agent tokens in DB; migration drops existing rows (no production
   data to preserve yet).
5. License split:
   - `agent/`, `frontend/`, `frontend-cloud/`, `hoister_shared/`,
     `documentation/` → MIT (current).
   - `controller/` → AGPL-3.0 (deters rehosted clones of the SaaS).
   - Add per-crate `LICENSE` files and update `Cargo.toml` `license` fields;
     update root `README.md` to note the split.

### Phase 1 — Multi-tenant safety

1. `user_id` scoping in `StateMemory` and `PendingUpdatesMemory`.
2. All read endpoints filter by `user_id`.
3. Per-token rate limit on `POST /container/state` (today: every 5s per
   agent, no limit).
4. Payload size limits.
5. Bind internal port (`3034`) to a private interface only, not `0.0.0.0`.
6. Audit log (which `user_id` did what) for incident response.

### Phase 2 — Agent defaults to hoister.io

1. `Controller::url` defaults to `https://api.hoister.io` when neither
   config file nor env sets it.
2. Three modes documented; dashboard snippet matches.
3. Standalone mode opt-in via `HOISTER_CONTROLLER=none` (or by omitting the
   token).

### Phase 3 — Cloud frontend MVP

Today: landing page + dashboard (deployments list, token snippet). Missing
for parity with self-hosted:

- Container detail view (`/containers/[hostname]/[project]/[service]`).
- Container list view.
- Host list.
- Pending-updates view (the new feature from upstream).
- Token rotation UI ("revoke and regenerate").
- Filled-in `/datenschutz` and `/impressum`.

Shared UI: extract container/deployment components into a `frontend-shared/`
workspace, or duplicate now and dedupe later. Both apps are SvelteKit so a
shared package is straightforward.

### Phase 4 — Deploy & open-source

- Hosting: **self-host everything on the VPS** for now. Frontend, controller,
  and Postgres all run on the same host (Hetzner or similar) behind Caddy
  for TLS. Move frontend to Vercel / Cloudflare Pages later if needed —
  current scale doesn't justify a managed platform.
- Bind layout on the VPS:
  - Caddy (or whatever reverse proxy) terminates TLS on `:443`, routes
    `api.hoister.io` to the controller's agent port (3033) and
    `hoister.io` to the frontend.
  - Controller internal port (`3034`) binds to `127.0.0.1` only; the
    frontend (also on `127.0.0.1`) reaches it via loopback. Never exposed.
  - Postgres binds to `127.0.0.1` only.
- Image names stay `emrius11/hoister*` for the first public release —
  rebrand to `hoister/*` can come later once the namespace is registered.
- CI builds & publishes images on tag. Extend `.github/workflows/rust.yml`.
- Encrypted DB backups, daily, 30-day retention.
- Run `/security-review` and `/ultrareview` against the final state.
- Flip repo public.

---

## Decisions log

1. **Standalone mode** — keep. Agent-only is the original use case.
2. **Log forwarding to cloud** — opt-in (`HOISTER_REPORT_LOGS=true`).
3. **Frontend hosting** — self-host on the VPS for now; revisit later.
4. **License split** — MIT for agent / frontend / shared / docs;
   AGPL-3.0 for the controller.
5. **Leaked `hst_` token** — dev-only throwaway. No production rotation
   needed; still scrub from history before publish.
6. **Image rebrand** — defer. Ship under `emrius11/*` for the first public
   release.

---

## Recommended next steps

1. Phase 0 step 1–2 (rotate + scrub) — non-negotiable before publish.
2. Phase 1 step 1 (`user_id` scoping) — non-negotiable before letting anyone
   else point an agent at hoister.io.

Everything else is sequencing.
