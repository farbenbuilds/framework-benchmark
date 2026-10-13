#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
STATE_DIR=${BENCHMARK_STATE_DIR:-/var/lib/framework-benchmark}
RUNTIME=${CONTAINER_RUNTIME:-docker}
APPLY=0

compose() {
    if "$RUNTIME" compose version >/dev/null 2>&1; then
        "$RUNTIME" compose "$@"
        return
    fi
    if [ "$RUNTIME" = docker ] && command -v docker-compose >/dev/null 2>&1; then
        docker-compose "$@"
        return
    fi
    echo "No Compose plugin found for $RUNTIME." >&2
    return 1
}

[ "${1:-}" = "--apply" ] && APPLY=1
if [ "$APPLY" -eq 0 ]; then
    echo "Dry run. Re-run with --apply to remove benchmark resources and restore saved sysctls."
    exit 0
fi

if ! command -v "$RUNTIME" >/dev/null 2>&1; then
    echo "Container runtime not found: $RUNTIME" >&2
    exit 1
fi

containers=$("$RUNTIME" ps -aq --filter label=framework-benchmark.managed=true)
if [ -n "$containers" ]; then
    "$RUNTIME" rm -f $containers
fi

if [ "${BENCHMARK_PURGE_DATABASE:-0}" = 1 ]; then
    compose -f "$REPO_ROOT/deploy/postgres/compose.yaml" down -v --remove-orphans
else
    compose -f "$REPO_ROOT/deploy/postgres/compose.yaml" down --remove-orphans
fi

if [ -f "$STATE_DIR/sysctl.before" ]; then
    if [ "$(id -u)" -ne 0 ]; then
        echo "Run as root to restore saved sysctl values." >&2
        exit 1
    fi
    while IFS='=' read -r key value; do
        sysctl -w "$key=$value"
    done < "$STATE_DIR/sysctl.before"
    rm -f "$STATE_DIR/sysctl.before"
fi

if [ "${BENCHMARK_PRUNE:-0}" = 1 ]; then
    "$RUNTIME" image prune -f
fi

echo "Benchmark environment cleanup complete."
