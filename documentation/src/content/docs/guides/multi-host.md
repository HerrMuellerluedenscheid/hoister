---
title: Multi-host setup
description: Monitor containers across multiple hosts and projects from a single dashboard.
---

Hoister supports running multiple agents on different hosts, each monitoring its own Docker daemon. All agents report to a single controller, and the dashboard groups containers by hostname and project.

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ host-alpha  │     │  host-beta  │     │ host-gamma  │
│(project-web)│     │(project-web)│     │(project-api)│
│             │     │             │     │             │
│  hoister    │     │  hoister    │     │  hoister    │
│  agent      │     │  agent      │     │  agent      │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────┴───────┐
                    │  controller  │
                    │  + frontend  │
                    └──────────────┘
```

Each agent identifies itself with a **hostname** and a **project name**. The controller uses these to distinguish which containers belong to which host and project.

## Controller and frontend

Deploy the controller and frontend on a central host. See the [Dashboard](/guides/frontend) guide for the basics.

```yaml title="docker-compose.yml (central host)"
services:
  hoister-controller:
    image: emrius11/hoister-controller:latest
    volumes:
      - controller-data:/data

  hoister-frontend:
    image: emrius11/hoister-frontend:latest
    ports:
      - "3000:3000"
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_AUTH_USERNAME: admin
      HOISTER_AUTH_PASSWORD: $2y$05$xXHhvkw0Jl95eYvK9zMubuTj39YgyKcwj2etuEgLFeec4.S9K5AVC

volumes:
  controller-data:
```

## Agent configuration

On each host, deploy a hoister agent and point it at the controller. Set `HOISTER_HOSTNAME` and `HOISTER_PROJECT` to identify the agent.

### Host Alpha (project-web)

```yaml title="docker-compose.yml (host-alpha)"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_HOSTNAME: host-alpha
      HOISTER_PROJECT: project-web
      HOISTER_CONTROLLER_URL: "http://controller.example.com:3033"
      HOISTER_SCHEDULE_INTERVAL: "60"

  nginx:
    image: nginx:latest
    labels:
      - "hoister.enable=true"
```

### Host Beta (project-web)

A second host in the same project. The controller will show both hosts side by side.

```yaml title="docker-compose.yml (host-beta)"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_HOSTNAME: host-beta
      HOISTER_PROJECT: project-web
      HOISTER_CONTROLLER_URL: "http://controller.example.com:3033"
      HOISTER_SCHEDULE_INTERVAL: "60"

  nginx:
    image: nginx:latest
    labels:
      - "hoister.enable=true"
```

### Host Gamma (project-api)

A different project on a third host.

```yaml title="docker-compose.yml (host-gamma)"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_HOSTNAME: host-gamma
      HOISTER_PROJECT: project-api
      HOISTER_CONTROLLER_URL: "http://controller.example.com:3033"
      HOISTER_SCHEDULE_INTERVAL: "60"

  httpbin:
    image: kennethreitz/httpbin:latest
    labels:
      - "hoister.enable=true"
```

## Environment variables

| Variable | Description |
|---|---|
| `HOISTER_HOSTNAME` | Identifies the host in the dashboard. Defaults to the container ID if not set. |
| `HOISTER_PROJECT` | The project name. Must match the Docker Compose project name (`com.docker.compose.project` label) on that host. |
| `HOISTER_CONTROLLER_URL` | URL of the controller (e.g. `http://controller.example.com:3033`). |
| `HOISTER_SCHEDULE_INTERVAL` | Seconds between container state reports to the controller. |

## TLS

When exposing the controller over a network, you should encrypt traffic between agents and the controller. See the [TLS encryption](/guides/tls) guide for setup instructions. In short, add these to each agent:

```yaml
volumes:
  - ./certs/ca.pem:/certs/ca.pem:ro
environment:
  HOISTER_CONTROLLER_URL: "https://controller.example.com:3033"
  HOISTER_CONTROLLER_CA_CERT_PATH: /certs/ca.pem
```

## Hiding infrastructure containers

Agents report all containers in their Docker Compose project. To hide Hoister's own containers from the dashboard, add the `hoister.hide` label:

```yaml
services:
  hoister:
    image: emrius11/hoister:latest
    labels:
      - "hoister.hide=true"
```
