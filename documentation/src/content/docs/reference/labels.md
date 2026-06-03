---
title: Container Labels
description: Docker labels Hoister reads to decide which containers to manage and how.
---

Hoister reads [Docker labels](https://docs.docker.com/reference/compose-file/services/#labels)
off your containers to decide which ones to manage and how to treat each. Labels are
the **per-container** half of [Hoister's configuration](/guides/configuration/); agent-wide
behaviour lives in the [TOML file](/reference/toml/) and [environment variables](/reference/environment-variables/).

## `hoister.enable`

```yaml
labels:
  - "hoister.enable=true"
```

Opt a container in to **automatic updates**. Only containers with
`hoister.enable=true` are checked for new images and updated/rolled back. Containers
without it are left alone by the updater.

## `hoister.hide`

```yaml
labels:
  - "hoister.hide=true"
```

Exclude a container from **reporting** to the controller — its state, logs, and
metrics are not sent and it won't appear in the dashboard. Use it for sidecars or
noisy helpers you don't want to see. (By default the agent reports on every container
in its Compose project, whether or not `hoister.enable` is set.)

## `hoister.identifier`

```yaml
labels:
  - "hoister.identifier=my-service"
```

Override the **service name** Hoister uses to identify the container in deployments,
metrics, and the dashboard. When unset, Hoister falls back to the Docker Compose
service name, and finally to the container name.

## `hoister.backup-volumes`

```yaml
labels:
  - "hoister.backup-volumes=true"
```

Back up the container's **named volumes** before applying an update. If the update
fails its health check and Hoister rolls back, the volumes are restored from the
backup. Bind mounts are not affected. See the
[Getting Started guide](/guides/getting-started/#volume-backups-and-rollbacks).

## Example

```yaml title="docker-compose.yml"
services:
  app:
    image: ghcr.io/acme/app:latest
    volumes:
      - app-data:/var/lib/app
    labels:
      - "hoister.enable=true"
      - "hoister.backup-volumes=true"
      - "hoister.identifier=acme-app"

  metrics-sidecar:
    image: prom/node-exporter:latest
    labels:
      - "hoister.hide=true"   # don't report this one to the dashboard

volumes:
  app-data:
```
