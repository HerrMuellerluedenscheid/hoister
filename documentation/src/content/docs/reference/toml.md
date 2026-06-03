---
title: Toml Configuration
description: Configure hoister with a toml file
---

As an alternative to environment variables, you can configure hoister with a toml file.
Find an example below:

```toml title="hoister.toml"
auto_update = true      # set to false to only detect updates without applying them automatically
report_metrics = true   # collect per-container CPU/memory metrics (on by default; set false to disable)
report_logs = false     # forward failed-container logs to the controller (off by default)

[schedule]
cron="0 * * * * * *"

[registry.ghcr]
username="foo"
token="ghc_asdfasdf"

[dispatcher.telegram]
token="123456789:qwertyuiopasdfghjkl"
chat=123456789

[dispatcher.slack]
webhook="https://hooks.slack.com/xxx/xx"
channel="channel-name"

[dispatcher.discord]
token="foo"
channel="getsoverriddenbyenvvar"
```

## Disable automatic rollout

Set `auto_update = false` to switch Hoister into **detection-only** mode. In this mode Hoister checks for new image versions on the configured schedule but does not pull or restart any containers. Instead, detected updates are reported to the controller and appear in the dashboard as **Pending Updates**, where you can review and apply them manually.

```toml title="hoister.toml"
auto_update = false

[schedule]
cron="0 * * * * * *"
```

This is useful in production environments where you want to control exactly when a service is updated. See the [Manual Rollout guide](/guides/manual-rollout/) for a full walkthrough.

## Metrics and log forwarding

`report_metrics` controls whether the agent samples per-container CPU/memory usage and sends it to the controller for the dashboard graphs. It is **enabled by default**; set it to `false` to disable.

`report_logs` controls whether the agent forwards the logs of failed/crashed containers to the controller (so a rolled-back deployment shows *why* it failed). It is **disabled by default** because logs can contain secrets; set it to `true` to enable.

```toml title="hoister.toml"
report_metrics = false   # turn metrics collection off
report_logs = true       # turn failure-log forwarding on
```

Both require a controller to be configured. See the [Metrics & log forwarding guide](/guides/monitoring/) for details and the security note on logs.

## Container labels

Which containers Hoister manages, hides, or backs up is configured with **per-container Docker labels**, not this file. See the [Container labels reference](/reference/labels/).

Save the file as `hoister.toml` and mount it into the container:

```yaml title="docker-compose.yml"
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./hoister.toml:/hoister.toml
    security_opt:
      - no-new-privileges:true
```

If both, environment variables and a toml file are present, the environment variables will be used.
