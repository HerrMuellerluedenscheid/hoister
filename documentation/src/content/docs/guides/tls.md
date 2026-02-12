---
title: TLS encryption
description: Encrypt traffic between Hoister agents and the controller.
---

By default, traffic between the Hoister agent and the controller is unencrypted. You can enable TLS to secure this communication using a self-signed CA.

## Generate certificates

Use `openssl` to create a CA and a server certificate for the controller. The Subject Alternative Names (SANs) must include the hostname that agents and the frontend use to reach the controller.

```bash
mkdir -p certs

# Create a CA
openssl genrsa -out certs/ca-key.pem 4096
openssl req -new -x509 -key certs/ca-key.pem -out certs/ca.pem -days 3650 \
    -subj "/CN=Hoister CA"

# Create a server certificate
openssl genrsa -out certs/server-key.pem 4096
openssl req -new -key certs/server-key.pem -out certs/server.csr \
    -subj "/CN=hoister-controller"

cat > certs/server-ext.cnf <<EOF
[v3_req]
subjectAltName = DNS:hoister-controller
EOF

openssl x509 -req -in certs/server.csr -CA certs/ca.pem -CAkey certs/ca-key.pem \
    -CAcreateserial -out certs/server.pem -days 365 \
    -extfile certs/server-ext.cnf -extensions v3_req

# Clean up intermediate files
rm -f certs/server.csr certs/server-ext.cnf certs/ca.srl
```

Adjust the `subjectAltName` in the extensions file to match the DNS name your services use to reach the controller (e.g. `DNS:hoister-controller` in Docker Compose).

If you use a [justfile](https://github.com/casey/just), the repository includes a `just gen-certs` recipe that runs these commands.

## Configure the controller

Mount the server certificate and private key into the controller container, and set the corresponding environment variables:

```yaml title="docker-compose.yml"
services:
  hoister-controller:
    image: emrius11/hoister-controller:latest
    volumes:
      - controller-data:/data
      - ./certs:/certs:ro
    environment:
      HOISTER_CONTROLLER_TLS_CERT_PATH: /certs/server.pem
      HOISTER_CONTROLLER_TLS_KEY_PATH: /certs/server-key.pem
```

Both variables must be set together. If only one is provided, the controller will refuse to start.

## Configure the agent

Mount the CA certificate into the agent container and set the environment variables so the agent trusts the CA and connects over HTTPS:

```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./certs/ca.pem:/certs/ca.pem:ro
    environment:
      HOISTER_CONTROLLER_URL: "https://hoister-controller:3033"
      HOISTER_CONTROLLER_CA_CERT_PATH: /certs/ca.pem
```

The agent only needs the CA certificate (`ca.pem`), not the server private key.

## Configure the frontend

The frontend also connects to the controller. Mount the CA certificate and set `NODE_EXTRA_CA_CERTS` so Node.js trusts it:

```yaml title="docker-compose.yml"
services:
  hoister-frontend:
    image: emrius11/hoister-frontend:latest
    volumes:
      - ./certs/ca.pem:/certs/ca.pem:ro
    environment:
      HOISTER_CONTROLLER_URL: "https://hoister-controller:3033"
      NODE_EXTRA_CA_CERTS: /certs/ca.pem
```

## Full example

```yaml title="docker-compose.yml"
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./certs/ca.pem:/certs/ca.pem:ro
    security_opt:
      - no-new-privileges:true
    env_file: ".env"
    environment:
      HOISTER_CONTROLLER_URL: "https://hoister-controller:3033"
      HOISTER_CONTROLLER_CA_CERT_PATH: /certs/ca.pem

  hoister-controller:
    image: emrius11/hoister-controller:latest
    volumes:
      - controller-data:/data
      - ./certs:/certs:ro
    environment:
      HOISTER_CONTROLLER_TLS_CERT_PATH: /certs/server.pem
      HOISTER_CONTROLLER_TLS_KEY_PATH: /certs/server-key.pem

  hoister-frontend:
    image: emrius11/hoister-frontend:latest
    ports:
      - "3000:3000"
    volumes:
      - ./certs/ca.pem:/certs/ca.pem:ro
    environment:
      HOISTER_CONTROLLER_URL: "https://hoister-controller:3033"
      HOISTER_AUTH_USERNAME: admin
      HOISTER_AUTH_PASSWORD: $2y$05$xXHhvkw0Jl95eYvK9zMubuTj39YgyKcwj2etuEgLFeec4.S9K5AVC
      NODE_EXTRA_CA_CERTS: /certs/ca.pem

volumes:
  controller-data:
```
