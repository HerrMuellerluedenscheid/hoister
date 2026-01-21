---
title: Toml Configuration
description: Configure hoister with a toml file
---

As an alternative to environment variables, you can configure hoister with a toml file.
Find an example below:

```toml title="hoister.toml"
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
