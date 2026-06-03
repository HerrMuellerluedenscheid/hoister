---
title: Environment Variables
description: How to configure Hoister using environment variables
---

You can configure the agent's behaviour with `HOISTER_`-prefixed environment variables.
These are one of [three ways to configure Hoister](/guides/configuration/) — alongside the
[TOML config file](/reference/toml/) and per-container [labels](/reference/labels/). When a
setting is given both here and in the TOML file, the environment variable wins.

Nested TOML keys map to underscores: `[schedule] interval` becomes `HOISTER_SCHEDULE_INTERVAL`.

## Agent behaviour

```dotenv
HOISTER_AUTO_UPDATE=true              # false = detect updates but don't apply them (manual rollout)
HOISTER_REPORT_METRICS=true           # collect per-container CPU/memory metrics (on by default)
HOISTER_REPORT_LOGS=false             # forward failed-container logs to the controller (off by default)
HOISTER_REDACT_KEYWORDS=license,pin   # extra env-var key substrings to redact (on top of the built-ins)
```

- `HOISTER_REPORT_METRICS` is **on by default**; set it to `false` to disable metrics collection.
- `HOISTER_REPORT_LOGS` is **off by default**; set it to `true` to forward crash/rollback logs.
- Both accept `true`/`1`/`yes`/`on` and `false`/`0`/`no`/`off`, and both require a controller.
- `HOISTER_REDACT_KEYWORDS` is a comma-separated list of extra keywords used to redact
  sensitive env-var values and log secrets. It **adds to** the built-in list (and any
  `redact_keywords` in the TOML file) rather than replacing it.

See the [Metrics & log forwarding guide](/guides/monitoring/) and the
[Manual Rollout guide](/guides/manual-rollout/) for details.

## Slack Webhook Notification

```dotenv
HOISTER_SLACK_WEBHOOK_URL="https://hooks.slack.com/services/XXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXX"
HOISTER_SLACK_CHANNEL="#my-update-channel"
```

## Telegram Notification

```dotenv
HOISTER_TELEGRAM_BOT_TOKEN="12345656789:XXXXXXXXXX-XXXXXXXXX-XXXXXXXXX"
HOISTER_TELEGRAM_CHAT_ID="9999999999"
```

## Discord Notification

```dotenv
HOISTER_DISCORD_BOT_TOKEN="soijf23JASDFOIJ@.Gj7gl8.sdfoij234sdf_sdfijoij23lijasdASDF"
HOISTER_DISCORD_CHANNEL_ID="12334556898709812334"
```

## Email Notification

Typically, you will need to configure an app password for your email provider for this to work. Your standard password
will likely not work.

```dotenv
HOISTER_DISPATCHER_EMAIL_SMTP_PASSWORD="super_secure_app_password"
HOISTER_DISPATCHER_EMAIL_SMTP_SERVER="smtp.gmail.com"
HOISTER_DISPATCHER_EMAIL_SMTP_USER="foo@bar.com"
HOISTER_DISPATCHER_EMAIL_RECIPIENT="user-to-be-informed-about-update@gmail.com"
```

## Schedule updates

```dotenv
HOISTER_SCHEDULE_INTERVAL=60   # in seconds
```

If you want to define the update intervals using cron syntax, you can instead configure hoister using a [toml file](./toml.md).
