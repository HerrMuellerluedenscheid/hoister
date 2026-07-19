# Hoister

[![Discord](https://img.shields.io/discord/1453411867224576105?color=7289da&label=Discord&logo=discord&logoColor=white)](https://discord.gg/D8kHFJXY7X)
[![GitHub](https://img.shields.io/badge/GitHub-hoister-181717?logo=github)](https://github.com/HerrMuellerluedenscheid/hoister)

**Deploy Docker images automatically — with rollback when an update fails.**

Hoister runs as a small container inside your Docker Compose stack. It periodically checks whether a newer image was pushed under the same tag, pulls it, and recreates the container. If the updated container fails to come up, Hoister rolls back to the previous image — optionally restoring volume backups taken right before the update.

Written in Rust, shipped as a `scratch`-based image for `linux/amd64` and `linux/arm64`.

📖 **Full documentation: [docs.hoister.io](https://docs.hoister.io)** · 🐙 [Source on GitHub](https://github.com/HerrMuellerluedenscheid/hoister) · 💬 [Discord](https://discord.gg/D8kHFJXY7X)

---

## Quick start

Add the `hoister.enable=true` label to any service you want managed, then add Hoister to the same Compose file:

```yaml
services:
  app:
    image: myorg/myapp:latest
    labels:
      - "hoister.enable=true"

  hoister:
    image: hoister/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
```

That's it. Every 120 seconds (configurable) Hoister checks whether a newer image was pushed for `myorg/myapp:latest`. When one appears, it pulls the image and recreates the container — and rolls back if the new container fails to start.

Run like this, Hoister is fully **standalone**: no account, no external service, no telemetry leaving the host.

## Optional: dashboard

To see rollouts, container state, and metrics across hosts, connect the agent to a controller. Sign in at [hoister.io](https://hoister.io), mint a token, and add it:

```yaml
    environment:
      HOISTER_CONTROLLER_TOKEN: "hst_<your-token>"
```

Or run your own controller (`hoister/hoister-controller`) and point the agent at it:

```yaml
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_CONTROLLER_TOKEN: "your-shared-secret"
```

See [Operating modes](https://docs.hoister.io/guides/operating-modes/) for the details of hosted, self-hosted, and standalone operation.

## Container labels

Labels control per-container behaviour:

| Label | Effect |
|---|---|
| `hoister.enable=true` | Opt this container in to automatic updates. |
| `hoister.backup-volumes=true` | Back up named volumes before an update; restore them on rollback. |
| `hoister.identifier=my-service` | Override the service name shown in deployments and the dashboard. |
| `hoister.hide=true` | Don't report this container to the controller (sidecars, helpers). |

Full reference: [docs.hoister.io/reference/labels](https://docs.hoister.io/reference/labels/)

## Configuration

Everything can be set through `HOISTER_`-prefixed environment variables — no config file required:

| Variable | Default | Meaning |
|---|---|---|
| `HOISTER_SCHEDULE_INTERVAL` | `120` | Seconds between update checks. |
| `HOISTER_AUTO_UPDATE` | `true` | `false` detects new images but doesn't apply them (manual rollout). |
| `HOISTER_CONTROLLER_TOKEN` | — | Agent token; enables reporting to a controller. |
| `HOISTER_CONTROLLER_URL` | `https://api.hoister.io` | Controller to report to; override for self-hosted. |
| `HOISTER_REPORT_METRICS` | `true` | Per-container CPU/memory metrics for the dashboard. |
| `HOISTER_REPORT_LOGS` | `false` | Forward log tails of failed containers to the controller (opt-in). |
| `HOISTER_REDACT_KEYWORDS` | — | Extra comma-separated keywords marking env vars as secrets to redact. |

For cron-style schedules and private registry credentials, mount a TOML file at `/hoister.toml`:

```toml
[schedule]
cron = "0 0 4 * * * *"   # every day at 04:00 instead of a fixed interval

[registry.ghcr]
username = "you"
token = "ghp_..."
```

Docker Hub, GHCR, AWS ECR, Azure ACR, and Google Artifact Registry are supported. See the [TOML reference](https://docs.hoister.io/reference/toml/), [environment variables](https://docs.hoister.io/reference/environment-variables/), and the [registries guide](https://docs.hoister.io/guides/registries/).

## Notifications

Get a message when an update succeeds, fails, or is rolled back — via Slack, Discord, Telegram, Microsoft Teams, Email, Matrix, Mattermost, Rocket.Chat, Google Chat, ntfy, Gotify, Pushover, or a generic webhook:

```yaml
    environment:
      HOISTER_TELEGRAM_BOT_TOKEN: "123456789:XXXX"
      HOISTER_TELEGRAM_CHAT_ID: "9999999999"
```

See the [notifications guide](https://docs.hoister.io/guides/notifications/) for all channels.

## Security

- The agent needs the Docker socket to pull images and recreate containers; run it with `security_opt: [no-new-privileges:true]` as shown above.
- The image is built `FROM scratch` and contains only the static binary plus busybox `sh`/`cp` for volume backups — no package manager, minimal attack surface.
- When reporting to a controller, env-var values matching secret-like keywords are redacted before anything leaves the host; `HOISTER_REDACT_KEYWORDS` extends the built-in list.
- Container logs are **never** forwarded unless you opt in with `HOISTER_REPORT_LOGS=true`.
- In standalone mode (no controller token) the agent talks to nothing but your Docker daemon and your image registries.

## Supported tags

- `latest` — most recent stable release
- `3`, `3.3`, `3.3.1` — semver tags per release

Architectures: `linux/amd64`, `linux/arm64`.

## Related images

- [`hoister/hoister-controller`](https://hub.docker.com/r/hoister/hoister-controller) — self-hosted backend aggregating deployments and metrics
- [`hoister/hoister-frontend`](https://hub.docker.com/r/hoister/hoister-frontend) — self-hosted dashboard UI

## License

The agent is MIT-licensed. Source: [github.com/HerrMuellerluedenscheid/hoister](https://github.com/HerrMuellerluedenscheid/hoister)
