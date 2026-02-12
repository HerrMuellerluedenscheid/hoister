#!/usr/bin/env bash
#
# Launches multiple compose projects to test multi-host + multi-project
# reporting to a single shared controller.
#
# Each project gets its own agent (with a distinct hostname and project name)
# and its own workload containers. All agents connect to the shared controller
# started first.
#
# Prerequisites:
#   just gen-certs
#
# Usage:
#   test/run-multi-project.sh          # start everything
#   test/run-multi-project.sh down     # tear everything down

set -euo pipefail
cd "$(dirname "$0")/.."

ACTION="${1:-up}"
COMPOSE_SHARED=(-f test/compose/shared.yaml)
COMPOSE_WEB=(-f test/compose/project-web.yaml)
COMPOSE_API=(-f test/compose/project-api.yaml)

if [ "$ACTION" = "down" ]; then
    docker compose -p project-api  "${COMPOSE_API[@]}"    down -v
    docker compose -p project-web  "${COMPOSE_WEB[@]}"    down -v
    docker compose -p shared       "${COMPOSE_SHARED[@]}" down -v
    docker network rm hoister-test-net 2>/dev/null || true
    exit 0
fi

# Create a shared network so all projects can reach the controller
docker network create hoister-test-net 2>/dev/null || true

docker compose -p shared      "${COMPOSE_SHARED[@]}" up --build -d
docker compose -p project-web "${COMPOSE_WEB[@]}"    up --build -d
docker compose -p project-api "${COMPOSE_API[@]}"    up --build -d

echo ""
echo "Dashboard: http://localhost:3000  (admin / password)"
echo ""
echo "Running projects:"
echo "  shared       → controller + frontend"
echo "  project-web  → host-alpha agent + nginx, redis"
echo "  project-api  → host-beta agent  + httpbin"
echo ""
echo "Tear down with:  test/run-multi-project.sh down"
