---
title: Getting started guide
description: Set up Hoister using Docker Compose.
---

Hoister uses [docker's labels](https://docs.docker.com/reference/compose-file/services/#labels) to identify which containers to monitor and manage. In this example, we create a 
simple Docker Compose file that includes both the nginx and the Hoister service.

```yaml title="docker-compose.yml"
services:
  nginx:
    image: nginx:latest
    labels:
      - "hoister.enable=true"   # <-- This label tells Hoister to manage this container

  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
```

This will start Hoister and the nginx service and checks by default every 120 seconds if a new `latest` image was release.
If a new image is found, Hoister will automatically update the nginx container.

## Volume backups and rollbacks

To extend the example above, we can add a label to the nginx container and enable automatic volume backups:

```yaml title="docker-compose.yml"
services:
  nginx:
    image: nginx:latest
    volumes:
      - my-named-volume:/usr/share/nginx/html
    labels:
      - "hoister.enable=true"
      - "hoister.backup-volumes=true"   # <-- This label tells Hoister to back up attached volumes

volumes:
  my-named-volume:
```

If an update fails, Hoister will roll back the container to the previous version.
