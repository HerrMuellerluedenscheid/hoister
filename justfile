# Default recipe to show available commands
default:
    @just --list

lint:
    cd frontend && npx prettier . --write

bindings:
    rm -rf frontend/src/bindings
    cargo test export_bindings
    mv -f controller/bindings frontend/src
    mv hoister_shared/bindings/* frontend/src/bindings
    rm -rf hoister_shared/bindings
    rm -rf controller/bindings

gen-certs:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p certs
    # Generate CA key and certificate
    openssl genrsa -out certs/ca-key.pem 4096
    openssl req -new -x509 -key certs/ca-key.pem -out certs/ca.pem -days 3650 \
        -subj "/CN=Hoister CA"
    # Generate server key and CSR
    openssl genrsa -out certs/server-key.pem 4096
    openssl req -new -key certs/server-key.pem -out certs/server.csr \
        -subj "/CN=hoister-controller"
    # Create extensions file for SANs
    cat > certs/server-ext.cnf <<EOF
    [v3_req]
    subjectAltName = DNS:localhost,DNS:hoister-controller
    EOF
    # Sign the server certificate with the CA
    openssl x509 -req -in certs/server.csr -CA certs/ca.pem -CAkey certs/ca-key.pem \
        -CAcreateserial -out certs/server.pem -days 365 \
        -extfile certs/server-ext.cnf -extensions v3_req
    # Clean up intermediate files
    rm -f certs/server.csr certs/server-ext.cnf certs/ca.srl
    echo "Certificates generated in certs/"

test-works:
    docker build --no-cache -f test/works.Dockerfile --push -t emrius11/example:latest .
    docker image rm emrius11/example:latest

test-fails:
    docker build --no-cache -f test/fails.Dockerfile --push -t emrius11/example:latest .
    docker image rm emrius11/example:latest

dev-frontend: bindings
    #!/usr/bin/env bash
    set -a
    source .env.template
    set +a
    export HOISTER_CONTROLLER_URL="https://localhost:3033"
    export NODE_EXTRA_CA_CERTS=../certs/ca.pem
    cd frontend && npm run dev

dev-controller:
    #!/usr/bin/env bash
    set -a
    source .env.template
    set +a
    export HOISTER_CONTROLLER_TLS_KEY_PATH=certs/server-key.pem
    export HOISTER_CONTROLLER_TLS_CERT_PATH=certs/server.pem
    export HOISTER_CONTROLLER_DATABASE_PATH=/tmp/hoister-dev.sqlite
    export RUST_LOG=debug
    cargo run --bin controller

dev-hoister:
    #!/usr/bin/env bash
    set -a
    source .env.template
    source .env
    set +a
    export HOISTER_CONTROLLER_CA_CERT_PATH=certs/ca.pem
    export HOISTER_CONTROLLER_URL="https://localhost:3033"
    export HOISTER_SCHEDULE_INTERVAL="10"
    export HOISTER_PROJECT=hoister
    export RUST_LOG=debug,bollard=info,hyper_util=info
    cargo run --bin hoister

dev-documentation:
    #!/usr/bin/env bash
    set -a
    source .env.template
    cd documentation && npm run dev

test-message:
    #!/usr/bin/env bash
    set -a
    source .env.template
    source .env
    set +a
    export RUST_LOG=debug,bollard=info,hyper_util=info
    cargo run --bin hoister -- --test-message
