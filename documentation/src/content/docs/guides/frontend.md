---
title: Dashboard
description: Keep track of all monitored services
---

The Hoister container is stateless and does not store any info about updates. But it can forward the update events to the
hoister controller service. The dashboard is a frontend for this service to keep track of all monitored services and updates.

```yaml title="docker-compose.yml"
service:
  
  hoister-controller:
    image: emrius11/hoister-controller:latest
    labels:
      - "hoister.enable=true"  # enable deployments
    volumes:
      - controller-data:/data  # to persist deployments across restarts

  hoister-frontend:
    image: emrius11/hoister-frontend:latest
    labels:
      - "hoister.enable=true"  # enable deployments
    ports:
      - "3000:3000"
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_AUTH_USERNAME: admin
      HOISTER_AUTH_PASSWORD: $2y$05$xXHhvkw0Jl95eYvK9zMubuTj39YgyKcwj2etuEgLFeec4.S9K5AVC  # password
```

You can set a clear text password in the environment variable `HOISTER_AUTH_PASSWORD` but a better way is to use a
hashed password.
