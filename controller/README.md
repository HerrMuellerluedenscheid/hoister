# Hoister Controller

> **Looking for Hoister?** The image you add to your Compose stack is [`hoister/hoister`](https://hub.docker.com/r/hoister/hoister) — the agent that updates and rolls back your containers. This image is the **optional self-hosted backend** behind the dashboard. If you use the hosted dashboard at [hoister.io](https://hoister.io) (or no dashboard at all), you don't need it.

The controller receives deployment results, container state, and metrics from Hoister agents and stores them in SQLite. The dashboard ([`hoister/hoister-frontend`](https://hub.docker.com/r/hoister/hoister-frontend)) reads from it.

📖 **Docs: [docs.hoister.io](https://docs.hoister.io)** · 🐙 [GitHub](https://github.com/HerrMuellerluedenscheid/hoister) · 💬 [Discord](https://discord.gg/D8kHFJXY7X)

## Usage

Controller plus dashboard, with an agent pointed at them:

```yaml
services:
  hoister-controller:
    image: hoister/hoister-controller:latest
    environment:
      HOISTER_CONTROLLER_API_SECRET: "your-shared-secret"
    volumes:
      - controller-data:/data   # persist deployments across restarts

  hoister-frontend:
    image: hoister/hoister-frontend:latest
    ports:
      - "3000:3000"
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_AUTH_USERNAME: admin
      HOISTER_AUTH_PASSWORD: "..."   # bcrypt hash, see the docs

  hoister:
    image: hoister/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_CONTROLLER_TOKEN: "your-shared-secret"

volumes:
  controller-data:
```

Full walkthrough: [Dashboard guide](https://docs.hoister.io/guides/frontend/) · [Multi-host setup](https://docs.hoister.io/guides/multi-host/)

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `HOISTER_CONTROLLER_API_SECRET` | — | Shared secret agents must send as their token. **Unset = unauthenticated**; only acceptable on a trusted network. |
| `HOISTER_CONTROLLER_PORT` | `3033` | Listen port for agents and the frontend. |
| `HOISTER_CONTROLLER_DATABASE_PATH` | `sqlite:///data/sqlite.db` | SQLite database location — mount a volume at `/data`. |
| `HOISTER_CONTROLLER_PENDING_UPDATE_TTL_SECS` | `172800` | How long a reported pending update stays listed after the agent last re-reported it. Must comfortably exceed the slowest agent check schedule. |
| `HOISTER_CONTROLLER_TLS_CERT_PATH` / `HOISTER_CONTROLLER_TLS_KEY_PATH` | — | Serve TLS directly; set both or neither. See the [TLS guide](https://docs.hoister.io/guides/tls/). |

## Supported tags

`latest` and semver tags (`3`, `3.3`, `3.3.1`) per release, for `linux/amd64` and `linux/arm64` — matching the agent's versioning.

## Related images

- [`hoister/hoister`](https://hub.docker.com/r/hoister/hoister) — **the agent; start here**
- [`hoister/hoister-frontend`](https://hub.docker.com/r/hoister/hoister-frontend) — dashboard UI for this controller

## License

The controller is licensed under [AGPL-3.0-only](https://github.com/HerrMuellerluedenscheid/hoister/blob/main/controller/LICENSE) (the agent and frontend are MIT).
