---
title: Receive Notifications
description: Stay informed of changes to your project
---

Hoister can send notifications to Slack, Discord, Email and Gotify to inform you of successful, failed and rolled back updates.


```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_SLACK_WEBHOOK_URL="https://hooks.slack.com/services/XXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXX"
      HOISTER_SLACK_CHANNEL="#my-update-channel"
      HOISTER_TELEGRAM_BOT_TOKEN="12345656789:XXXXXXXXXX-XXXXXXXXX-XXXXXXXXX"
      HOISTER_TELEGRAM_CHAT_ID="9999999999"
```

All available options can be found in the [configuration reference](/reference/environment-variables/).
