---
title: Troubleshooting
description: Solutions to common issues with Hoister.
---

## Permission denied on the Docker socket

```
dial unix /var/run/docker.sock: connect: permission denied
```

This error means the user running Hoister is not a member of the `docker` group. Add the user (replace `foo` with the actual username) and then re-login or start a new shell session:

```bash
sudo usermod -aG docker foo
```

If you are running Hoister as a container and still see this error, make sure the socket is mounted correctly:

```yaml
volumes:
  - /var/run/docker.sock:/var/run/docker.sock
```

## Container does not get updated

- Confirm the label `hoister.enable=true` is set on the target service.
- Check that Hoister has access to the registry. For private registries see the [Registries guide](/guides/registries/).
- Increase log verbosity by setting `RUST_LOG=debug` on the Hoister container to see what is happening.

## Rollback was triggered unexpectedly

Hoister marks an update as failed when the container exits with a non-zero code or does not become healthy within the configured timeout. Check the container logs of the updated service to find the root cause:

```bash
docker logs <container-name>
```
