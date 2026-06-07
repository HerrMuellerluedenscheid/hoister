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
redact_keywords = ["license", "pin"]   # extra env-var key substrings to redact (on top of the built-ins)

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

# Alternative to a bot token: post via a Discord incoming webhook. No bot
# required; the target channel is fixed when you create the webhook. The
# username/avatar_url overrides are optional.
[dispatcher.discord_webhook]
webhook="https://discord.com/api/webhooks/123456789/your-webhook-token"
username="hoister"
# avatar_url="https://example.com/icon.png"

# Microsoft Teams via an incoming webhook. No app registration; the target
# channel is fixed when you create the webhook. Works with both the Workflows
# (Power Automate) webhooks and the legacy connector webhooks.
[dispatcher.teams]
webhook="https://example.webhook.office.com/webhookb2/.../IncomingWebhook/.../..."

# Gotify push server.
[dispatcher.gotify]
server="https://gotify.example.com"
token="your-application-token"

# Email via SMTP. `from` is an optional display name (defaults to "hoister");
# the SMTP port is fixed at 587.
[dispatcher.email]
recipient="alerts@example.com"
from="hoister"
[dispatcher.email.smtp]
user="bot@example.com"
password="your-smtp-password"
server="smtp.example.com"

# ntfy. `access_token` is only needed for reserved/protected topics.
[dispatcher.ntfy]
server="https://ntfy.sh"
topic="my-deploys"
# access_token="tk_xxxxxxxx"

# Pushover. `device` optionally targets a single device.
[dispatcher.pushover]
token="your-application-api-token"
user="your-user-or-group-key"
# device="phone"

# Matrix via a homeserver access token to a room the user has joined.
[dispatcher.matrix]
homeserver="https://matrix.org"
access_token="your-access-token"
room_id="!roomid:matrix.org"

# Mattermost via an incoming webhook. `channel` and `username` are optional
# overrides; a channel override only works if the webhook allows it.
[dispatcher.mattermost]
webhook="https://mattermost.example.com/hooks/xxxxxxxxxxxxxxxxxxxxxxxxxx"
# channel="town-square"
# username="hoister"

# Rocket.Chat via an incoming webhook. `channel` and `alias` are optional
# overrides.
[dispatcher.rocketchat]
webhook="https://rocketchat.example.com/hooks/xxxxxxxx/xxxxxxxxxxxxxxxxxxxx"
# channel="#general"
# alias="hoister"

# Google Chat via an incoming webhook. The target space is fixed when you
# create the webhook; the key/token in the URL is the secret.
[dispatcher.google_chat]
webhook="https://chat.googleapis.com/v1/spaces/AAAAxxxxxxx/messages?key=XXXX&token=YYYY"

# Generic webhook — Hoister POSTs each event as JSON. Optional headers carry
# any auth your endpoint needs.
[dispatcher.webhook]
url="https://example.com/hooks/hoister"
[dispatcher.webhook.headers]
Authorization="Bearer your-token"
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

## Custom redaction keywords

Hoister redacts environment-variable values whose key looks sensitive (e.g. `*_TOKEN`, `*_PASSWORD`, `*_SECRET`) before they reach the controller, and scrubs the same values out of forwarded logs. `redact_keywords` extends that built-in list with your own project-specific terms, loaded at startup:

```toml title="hoister.toml"
redact_keywords = ["license", "pin", "seed"]
```

Keywords are matched case-insensitively as substrings of the env-var key, so `license` also redacts `ACME_LICENSE_KEY`. The equivalent `HOISTER_REDACT_KEYWORDS` environment variable is comma-separated and is *added to* this list rather than replacing it. See the [Secret redaction section](/guides/monitoring/#secret-redaction) for the full built-in keyword list.

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
