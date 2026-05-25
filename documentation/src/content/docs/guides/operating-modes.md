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
set the token.

### Log forwarding (opt-in)

By default the agent ships container inspect payloads to the controller but
**not** the container logs. Container logs can contain secrets the agent's
keyword-based env-var redaction does not catch (e.g. tokens passed on the
command line, JSON-encoded credentials, third-party library output). When
a container is in a non-running state (restarting / exited / dead) you can
opt in to forwarding a tail of its logs to make debugging easier:

```yaml
environment:
  HOISTER_CONTROLLER_TOKEN: "hst_your-personal-token"
  HOISTER_REPORT_LOGS: "true"
```

The forwarded tail is capped at 16 KB and runs through the same env-var
value redactor as the inspect payload, but you should still treat opting
in as sharing log content with the hoister.io controller.

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
