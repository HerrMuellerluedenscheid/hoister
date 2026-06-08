Hoister
=======

[![Discord](https://img.shields.io/discord/1453411867224576105?color=7289da&label=Discord&logo=discord&logoColor=white)](https://discord.gg/D8kHFJXY7X)

Deploy Docker images automatically with rollback support.

> **Full documentation at [docs.hoister.io](https://docs.hoister.io)**

---

## Quick start

Mint an agent token at [hoister.io](https://hoister.io) (sign in, then open **Tokens**). Add the `hoister.enable=true` label to any service you want Hoister to manage, then add Hoister itself to the same Compose file:

```yaml
services:
  example:
    image: myorg/myapp:latest
    labels:
      - "hoister.enable=true"

  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    restart: unless-stopped
    environment:
      # Report to the hosted dashboard at api.hoister.io (the default
      # controller URL). Paste the token you minted above.
      HOISTER_CONTROLLER_TOKEN: "hst_<your-token>"
```

Push a new image under the same tag — Hoister will pull it and restart the container automatically. If the new container fails to start, it rolls back to the previous version, and the result shows up on your dashboard.

> **Prefer no hosted dashboard?** Drop the `HOISTER_CONTROLLER_TOKEN` line and Hoister runs standalone — same auto-update and rollback, with no telemetry leaving the host.

See the [Getting Started guide](https://docs.hoister.io/guides/getting-started/) for a full walkthrough including volume backups.

---

## Learn more

| Topic | Link |
|---|---|
| Notifications (Slack, Discord, Email, …) | [docs.hoister.io/guides/notifications](https://docs.hoister.io/guides/notifications/) |
| Private registries (GHCR, ECR, ACR, …) | [docs.hoister.io/guides/registries](https://docs.hoister.io/guides/registries/) |
| Dashboard (frontend + controller) | [docs.hoister.io/guides/frontend](https://docs.hoister.io/guides/frontend/) |
| Multi-host setup | [docs.hoister.io/guides/multi-host](https://docs.hoister.io/guides/multi-host/) |
| All environment variables | [docs.hoister.io/reference/environment-variables](https://docs.hoister.io/reference/environment-variables/) |
| Troubleshooting | [docs.hoister.io/guides/troubleshooting](https://docs.hoister.io/guides/troubleshooting/) |

---

## License

The agent (`agent/`), frontends (`frontend/`, `frontend-cloud/`), shared
crate (`hoister_shared/`), and documentation are MIT-licensed. See the
top-level [`LICENSE`](./LICENSE).

The controller (`controller/`) is licensed under
[AGPL-3.0-only](./controller/LICENSE) — modifications offered as a network
service must be published under the same license.
