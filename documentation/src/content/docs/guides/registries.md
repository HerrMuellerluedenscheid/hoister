---
title: Private Registries
description: Pull images from private container registries
---

Hoister supports pulling images from private registries. Configure credentials either via environment variables or a `hoister.toml` file.

## GitHub Container Registry (ghcr.io)

```toml title="hoister.toml"
[registry.ghcr]
username = "YourGithubUsername"
token = "ghp_aaijaweoijro123123"
```

```dotenv title=".env"
HOISTER_REGISTRY_GHCR_USERNAME="YourGithubUsername"
HOISTER_REGISTRY_GHCR_TOKEN="ghp_aaijaweoijro123123"
```

Matches images whose name starts with `ghcr.io/`.

---

## Docker Hub

```toml title="hoister.toml"
[registry.dockerhub]
username = "yourdockerhubuser"
password = "yourpassword"
```

```dotenv title=".env"
HOISTER_REGISTRY_DOCKERHUB_USERNAME="yourdockerhubuser"
HOISTER_REGISTRY_DOCKERHUB_PASSWORD="yourpassword"
```

Matches images with a `docker.io/` prefix and bare image names that have no registry host (e.g. `nginx:latest` or `myorg/myapp:1.0`).

---

## AWS Elastic Container Registry (ECR)

Hoister fetches a short-lived auth token from the ECR API automatically using your AWS credentials — no manual token rotation needed.

```toml title="hoister.toml"
[registry.ecr]
access_key_id = "AKIAIOSFODNN7EXAMPLE"
secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
region = "us-east-1"
```

```dotenv title=".env"
HOISTER_REGISTRY_ECR_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
HOISTER_REGISTRY_ECR_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
HOISTER_REGISTRY_ECR_REGION="us-east-1"
```

Matches images whose name contains `.dkr.ecr.` and `.amazonaws.com/`, for example:

```
123456789012.dkr.ecr.us-east-1.amazonaws.com/myapp:latest
```

---

## Azure Container Registry (ACR)

```toml title="hoister.toml"
[registry.acr]
username = "myregistry"
password = "mypassword"
```

```dotenv title=".env"
HOISTER_REGISTRY_ACR_USERNAME="myregistry"
HOISTER_REGISTRY_ACR_PASSWORD="mypassword"
```

Matches images whose name contains `.azurecr.io/`, for example:

```
myregistry.azurecr.io/myapp:latest
```

---

## Google Container Registry / Artifact Registry (GCR / GAR)

Use a [service account](https://cloud.google.com/iam/docs/service-account-overview) key for authentication. Set `username` to `_json_key` and `password` to the full JSON key content.

```toml title="hoister.toml"
[registry.gcr]
username = "_json_key"
password = """
{
  "type": "service_account",
  "project_id": "my-project",
  ...
}
"""
```

```dotenv title=".env"
HOISTER_REGISTRY_GCR_USERNAME="_json_key"
HOISTER_REGISTRY_GCR_PASSWORD='{"type":"service_account",...}'
```

Matches images starting with `gcr.io/`, `us.gcr.io/`, `eu.gcr.io/`, `asia.gcr.io/`, or containing `.pkg.dev/` (Google Artifact Registry), for example:

```
gcr.io/my-project/myapp:latest
us-central1-docker.pkg.dev/my-project/my-repo/myapp:latest
```
