# Cloud rollout — resume notes

Picked up the [cloud rollout plan](./cloud-rollout.md) (on branch
`docs/cloud-rollout`) and implemented all non-destructive Phase 0 / 1 work
plus Phase 2 agent defaults. This doc is the handoff for the next session.

## Where we left off

- Branch: `main`, 16 commits ahead of `d469460` (origin tip when we started).
- Working tree: clean apart from the long-standing untracked WIP
  (`frontend-cloud/`, `controller/bindings/`, `hoister_shared/bindings/`,
  the duplicate `claude.md`).
- All commits build clean: `cargo check --workspace`, `cargo clippy`, and
  the new tests pass.

## Done this round (newest first)

| Phase | Commit | Summary |
|---|---|---|
| 0 | `40236ec` | gitleaks allowlist tightened |
| 0 | `518c7a3` | gitleaks pre-commit + `hst_` rule |
| 2 | `423c456` | agent defaults to `https://api.hoister.io`; modes doc |
| 1 | `f2c07e1` | audit log middleware |
| 1 | `3439866` | per-user rate limit + 1 MiB body cap |
| 1 | `64bd4e8` | internal listener defaults to `127.0.0.1` |
| 1 | `a176468` | SSE events scoped by user_id |
| 0 | `ff641b8` | SHA-256 token hashing |
| 1 | `5c2de2d` | `user_id` partitioning of state + pending updates |
| 0 | `1105f35` | MIT / AGPL-3.0 split |
| 0 | `1b0ccc2` | `.env.example` for both frontends |
| 0 | `ecf1154` | compose-file secret placeholders |

Tests added: rate limiter (3), token hashing (2), agent config modes (2).

## Pending tasks

### Blocked on user approval

**`git filter-repo` to scrub history** — `bf8b0d3` (`docker-compose.cloud.yaml`)
still contains the literal `hst_efec3e248eae4b168c7acfd9d1895437` and
`happyhoistering43123`. Confirmed dev-only, no production rotation needed,
but should be removed from history before the repo goes public so secret
scanners don't flag the project.

To run it (destructive — overwrites history):

```sh
# Drop the literal token & password from every revision.
git filter-repo \
  --replace-text <(cat <<'EOF'
hst_efec3e248eae4b168c7acfd9d1895437==>${HOISTER_API_SECRET}
happyhoistering43123==>${POSTGRES_PASSWORD}
EOF
)
```

After running, every existing commit hash in this doc changes. Force-push
needed; if anything else has been cloned/forked it diverges.

### Frontend-cloud follow-ups (carried over from earlier session)

The dashboard currently does `data.agentToken.token.slice(0, 16)` — with
`ff641b8` the `token` field is `null` for returning users so this crashes.
The fix is small (handle null → show "Agent connected, token issued on
{created_at}" with a Rotate button), but `frontend-cloud/` is still WIP
and untracked. Fold in when ready.

The token shape changed — frontend-cloud TS type:

```ts
// before
export type TokenResponse = { token: string; user_id: string; is_new: boolean };
// after
export type TokenResponse = { token: string | null; user_id: string; is_new: boolean };
```

A "rotate token" endpoint (`POST /token/rotate`) needs to be added on the
controller; today the dashboard has no way to issue a new token once the
first one is created. Quick to add (insert new hash, return plaintext once,
delete the old row) — postponed because the cloud frontend isn't ready to
consume it.

## Next chunks (recommended order)

1. **`git filter-repo`** when you're ready to greenlight the destructive
   rewrite. Single command, single force-push.

2. **Frontend-cloud cleanup** — handle null `token`, fold the WIP commit in,
   delete the `claude.md` duplicate (lowercase vs `CLAUDE.md`).

3. **Token rotation endpoint + UI** — `POST /token/rotate` on the internal
   router, dashboard button.

4. **Phase 3 — Cloud frontend MVP**. Doc lists the missing routes:
   container detail, container list, host list, pending-updates view,
   filled-in `/datenschutz` and `/impressum`. Decide whether to extract a
   `frontend-shared/` workspace or duplicate components for now.

5. **Phase 4 — Deploy**. Single VPS (Hetzner-style), Caddy on `:443`,
   controller + postgres + frontend on `127.0.0.1`. CI workflow to publish
   `emrius11/hoister*` images on tag. Encrypted nightly backups.

6. **Pre-publish sweep** before flipping the repo public:
   - `cargo audit` clean
   - `gitleaks detect --no-git -c .gitleaks.toml` clean (today: only
     gitignored `.env*` warnings remain)
   - Run `/security-review` and `/ultrareview`.

## Things to know when picking up

- The controller is now generic over three type params `<DS, CS, TS>` —
  any new endpoint or middleware that references `AppState` needs all three
  bounds.
- `UserId` is required on every authenticated route. The agent middleware
  inserts `UserId("local")` in self-hosted-without-secret dev mode; the
  internal middleware **rejects** with 401 if `X-User-Id` is absent (no
  fallback). The cloud frontend BFF must always send it.
- The `pending_updates` table doesn't exist — it's an in-memory store that
  resets on controller restart. Document this when writing user-facing
  copy.
- The audit log uses `log!(target: "hoister.audit", ...)`. To grep prod
  logs cleanly, route this target to its own file in deployment config.
- Pre-commit now runs gitleaks; if it blocks a legitimate commit (e.g. you
  add a new placeholder), update the allowlist in `.gitleaks.toml` rather
  than bypassing the hook.

## Useful starting points next time

- `docs/cloud-rollout.md` (on branch `docs/cloud-rollout`) — full plan with
  the decisions log.
- `controller/src/lib/inbound/server.rs` — auth middleware + routers, the
  central nervous system of the controller.
- `controller/src/lib/inbound/{rate_limit,audit_log}.rs` — the new
  middleware. Add new cross-cutting concerns here.
- `agent/src/config.rs` — three-mode default + tests.
- `documentation/src/content/docs/guides/operating-modes.md` — user-facing
  modes explainer.
