---
title: Manual Rollout
description: Detect updates automatically but apply them on your own schedule.
---

By default Hoister applies a new image as soon as it is detected. If you prefer to stay in control of when a container is actually restarted — for example in a production environment — you can enable **detection-only** mode by setting `auto_update = false`.

## How it works

When `auto_update` is disabled Hoister still polls the registry on the configured schedule. If a newer image digest is found it:

1. Logs the pending update.
2. Reports it to the controller (if one is configured).
3. **Does not** pull the image or restart the container.

The pending update is then visible in the dashboard, where you can apply it with a single click.

## Configuration

Add `auto_update = false` to your `hoister.toml`:

```toml title="hoister.toml"
auto_update = false

[schedule]
cron="0 * * * * * *"
```

Or use an interval instead of cron:

```toml title="hoister.toml"
auto_update = false

[schedule]
interval = 300  # check every 5 minutes
```

See the [Toml Configuration reference](/reference/toml/) for the full list of options.

## Applying updates from the dashboard

When a pending update is detected it appears in the **Pending Updates** section at the top of the Containers page:

- **Host** – the hostname of the agent that detected the update.
- **Service** – the Docker Compose service name.
- **Image** – the image name currently in use.
- **New Digest** – the digest of the newer image.
- **Detected** – when the update was first detected.

Click **Apply** next to a service to trigger the rollout. The agent on the corresponding host receives the command, pulls the new image, and restarts the container. If the restart fails, Hoister rolls back to the previous image automatically.

:::note
The controller and frontend must be configured for pending updates to be visible in the dashboard. Without a controller the agent still skips the automatic rollout, but there is no UI to trigger it remotely. See the [Dashboard guide](/guides/frontend/) for setup instructions.
:::
