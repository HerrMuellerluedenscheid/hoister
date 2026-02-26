---
title: Toml Configuration
description: Configure hoister with a toml file
---

As an alternative to environment variables, you can configure hoister with a toml file.
Find an example below:

```toml title="hoister.toml"
auto_update = true  # set to false to only detect updates without applying them automatically

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
