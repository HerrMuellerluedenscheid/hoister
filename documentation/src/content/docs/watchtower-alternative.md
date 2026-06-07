---
title: A Watchtower alternative with automatic rollback
description: Hoister is a self-hosted Watchtower alternative that auto-updates Docker containers, then rolls back and restores volumes if an update fails its health check.
---

[Watchtower](https://github.com/containrrr/watchtower) is the tool most people
reach for to auto-update Docker containers: it watches your running containers,
pulls newer images, and restarts them. It is battle-tested and simple — but it
**updates and walks away**. If the new image is broken, your service stays down
until you notice and intervene.

Hoister is a self-hosted Watchtower alternative built around that exact gap. It
does the same automatic image updates, but treats every update as a deployment:
it runs the new container, watches its health check, and **rolls back to the
previous image — restoring named volumes — if the update fails**. An optional
dashboard then aggregates what happened across all your hosts.

## Why look for a Watchtower alternative?

- **No rollback.** Watchtower has no notion of a failed update. A bad image, a
  missing migration, or a crash-looping container just becomes downtime.
- **No data safety.** If an update corrupts a volume, there is nothing to
  restore from.
- **One host at a time.** Watchtower runs per host and reports to logs and
  notifications only — there is no shared view of what updated where.
- **Slowing development.** Watchtower's release cadence and issue triage have
  become infrequent, which pushes teams to look for an actively maintained
  option.

If none of those bother you, Watchtower is a perfectly good choice. If they do,
read on.

## Hoister vs Watchtower

| Capability | Hoister | Watchtower |
| --- | --- | --- |
| Automatic image updates | Yes | Yes |
| Scheduled checks (cron or interval) | Yes | Yes |
| Opt in/out via container labels | Yes | Yes |
| Private registries (GHCR, ECR, ACR, GCR) | Yes | Yes |
| Notifications | A dozen-plus channels | Yes (via shoutrrr) |
| Removes the old image after updating | Yes (automatic) | Optional (`--cleanup`) |
| **Automatic rollback on failed health check** | **Yes** | No |
| **Volume backup & restore on rollback** | **Yes** | No |
| **Approve-before-apply (detection-only) mode** | **Yes** | Monitor-only (notify, no apply) |
| **Web dashboard** | **Yes** (optional controller) | No |
| **Multi-host aggregation** | **Yes** | No (per host) |
| Deployment history & container metrics | Yes | No |
| Secret redaction before data leaves the host | Yes | — |
| Runs standalone without a backend | Yes | Yes |
| Pre/post-update lifecycle hooks | No | Yes |
| Implementation | Single Rust binary | Single Go binary |

## The differences that matter

### Rollback instead of downtime

This is the headline. With `hoister.enable=true` on a container, Hoister pulls a
new image, starts it, and watches its Docker health check. If the container
becomes healthy, the update sticks and the old image is cleaned up. If it fails,
Hoister puts the previous image back — automatically, within the same update
cycle. Watchtower has no equivalent: once it restarts the container, you own
whatever happens next.

### Your data survives a bad update

Add `hoister.backup-volumes=true` and Hoister snapshots the container's named
volumes before the update. If the update is rolled back, the volumes are
restored from that snapshot, so a botched migration doesn't leave your data in a
half-written state. Watchtower does not touch volumes at all.

### A dashboard, not just logs

Point the agent at the optional [controller](/guides/frontend/) and every host
reports deployments, rollbacks, pending updates, and CPU/memory metrics to one
web dashboard. Watchtower is per-host and surfaces results only through logs and
notifications.

### Approve updates before they roll out

Set `auto_update = false` and Hoister switches to **detection-only** mode: it
finds new images on schedule but doesn't apply them. They show up as *Pending
Updates* in the dashboard, where you click to roll out. Watchtower's
monitor-only mode can notify you, but it can't apply an update on demand from a
UI. See [Operating modes](/guides/operating-modes/).

:::note[When Watchtower is still the right call]
If you only want the simplest possible "pull and restart" with no controller,
don't need rollback or volume safety, and rely on Watchtower's pre/post-update
lifecycle hooks, Watchtower remains a solid, ubiquitous choice. Hoister earns
its keep when an update going wrong has real consequences.
:::

## Migrating from Watchtower

Hoister runs as a single service in your Compose stack, the same way Watchtower
does. Drop the Watchtower service and add Hoister:

```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true

  app:
    image: ghcr.io/acme/app:latest
    labels:
      - "hoister.enable=true"          # was: com.centurylinklabs.watchtower.enable=true
      - "hoister.backup-volumes=true"  # no Watchtower equivalent
```

The concepts map across directly:

| Watchtower | Hoister |
| --- | --- |
| `com.centurylinklabs.watchtower.enable=true` | `hoister.enable=true` label |
| `WATCHTOWER_SCHEDULE` / `--interval` | `[schedule]` `cron` or `interval` in [`hoister.toml`](/reference/toml/) |
| `WATCHTOWER_MONITOR_ONLY=true` | `auto_update = false` ([detection-only mode](/guides/operating-modes/)) |
| Shoutrrr notification URLs | `[dispatcher.*]` tables ([Notifications](/guides/notifications/)) |
| Registry auth via Docker config | [Registry credentials](/guides/registries/) (GHCR, ECR, ACR, GCR) |

That's the whole migration: swap the service, relabel your containers, and
optionally turn on volume backups for the containers whose data you care about.

## Get started

- [Getting started guide](/guides/getting-started/) — set up Hoister with Docker
  Compose, including volume backups and rollbacks.
- [Container labels](/reference/labels/) — the per-container settings Hoister
  reads.
- [Operating modes](/guides/operating-modes/) — automatic vs. approve-before-apply.
