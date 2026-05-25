---
title: Operating modes
description: Choose between hosted, self-hosted, and standalone agent deployments.
---

The Hoister agent runs in one of three modes depending on which environment
variables you set. The same image (`emrius11/hoister:latest`) supports all
three; no rebuild is needed.

## Hosted (default)

Reports container state and deployments to the hosted controller at
[api.hoister.io](https://api.hoister.io). Sign in at
[hoister.io](https://hoister.io) to get a personal `hst_` agent token and
view your dashboard.

```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_TOKEN: "hst_your-personal-token"
```

The controller URL defaults to `https://api.hoister.io` — you only need to
set the token. Logs are **not** forwarded by default; set
`HOISTER_REPORT_LOGS=true` to opt in.

## Self-hosted

Run your own controller and dashboard alongside the agent. Override the
controller URL:

```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_CONTROLLER_TOKEN: "your-shared-secret"

  hoister-controller:
    image: emrius11/hoister-controller:latest
    # ... see the Dashboard guide for the full setup
```

See the [Dashboard guide](/guides/frontend/) for the full controller +
frontend configuration.

## Standalone

No controller, no dashboard, no telemetry. The agent watches its compose
stack and updates containers locally — exactly the original Hoister
behaviour.

```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
```

Omit `HOISTER_CONTROLLER_TOKEN` and `HOISTER_CONTROLLER_URL` entirely. The
agent will log `Mode: standalone` at startup and never reach out to any
controller.
