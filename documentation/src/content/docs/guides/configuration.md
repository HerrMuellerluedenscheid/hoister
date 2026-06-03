---
title: Configuring the agent
description: The three ways to configure Hoister — container labels, a TOML config file, and environment variables — and how they interact.
---

Hoister's agent is configured through three mechanisms, each suited to a different
kind of setting:

| Mechanism | Scope | Best for |
| --- | --- | --- |
| **Container labels** | Per container | Telling Hoister *which* containers to manage and how to treat each one (`hoister.enable`, `hoister.hide`, `hoister.identifier`, `hoister.backup-volumes`). |
| **TOML config file** (`/hoister.toml`) | The whole agent | Everything else: update schedule, registries, notifications, controller connection, and behaviour flags like `auto_update`, `report_metrics`, `report_logs`, and `redact_keywords`. |
| **Environment variables** (`HOISTER_*`) | The whole agent | The same agent-wide settings as the TOML file — handy for Docker Compose and secrets. |

## Container labels

Labels are read off each container, so they control behaviour **per service**. See
the [Container labels reference](/reference/labels/) for the full list. Quick example:

```yaml title="docker-compose.yml"
services:
  nginx:
    image: nginx:latest
    labels:
      - "hoister.enable=true"          # manage this container
      - "hoister.backup-volumes=true"  # back volumes up before an update
```

## TOML config file

Agent-wide settings live in a TOML file mounted at `/hoister.toml`. See the
[TOML reference](/reference/toml/) for every key.

```yaml title="docker-compose.yml"
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./hoister.toml:/hoister.toml
    security_opt:
      - no-new-privileges:true
```

## Environment variables

The same agent-wide settings can be supplied as `HOISTER_`-prefixed environment
variables. Nested TOML keys map to underscores — e.g. `[schedule] interval` becomes
`HOISTER_SCHEDULE_INTERVAL`. See the
[Environment variables reference](/reference/environment-variables/).

## Precedence

When the same agent-wide setting is provided in more than one place, **environment
variables win over the TOML file**, which in turn wins over the built-in default.
Container labels are independent — they configure per-container behaviour that the
file and environment variables don't cover.
