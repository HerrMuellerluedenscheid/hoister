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
    cd frontend && npm run dev

dev-controller:
    #!/usr/bin/env bash
    set -a
    source .env.template
    set +a
    export HOISTER_CONTROLLER_DATABASE_PATH=/tmp/hoister-dev.sqlite
    export RUST_LOG=debug
    cargo run --bin controller

dev-hoister:
    #!/usr/bin/env bash
    set -a
    source .env.template
    source .env
    set +a
    export HOISTER_CONTROLLER_URL="http://localhost:3033"
    export HOISTER_SCHEDULE_INTERVAL="10"
    export HOISTER_PROJECT=hoister
    export RUST_LOG=debug,bollard=info,hyper_util=info
    cargo run --bin hoister

test-message:
    #!/usr/bin/env bash
    set -a
    source .env.template
    source .env
    set +a
    export RUST_LOG=debug,bollard=info,hyper_util=info
    cargo run --bin hoister -- --test-message