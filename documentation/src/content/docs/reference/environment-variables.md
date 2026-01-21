---
title: Environment Variables
description: How to configure Hoister using environment variables
---

You can configure Hoister using environment variables.

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
WATCH_INTERVAL=60   # in seconds
```

If you want to define the update intervals using cron syntax, you can instead configure hoister using a [toml file](./toml.md).
