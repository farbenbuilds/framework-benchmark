#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
RUNTIME=${CONTAINER_RUNTIME:-docker}
DATABASE_URL=${DATABASE_URL:-postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world}
BENCHMARK_WORKERS=${BENCHMARK_WORKERS:-$(getconf _NPROCESSORS_ONLN)}
LIBCS=${BENCHMARK_LIBCS:-"glibc musl"}
FAILED=0
CURRENT_CONTAINER=

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

stop_current() {
    [ -n "$CURRENT_CONTAINER" ] || return
    "$RUNTIME" rm -f "$CURRENT_CONTAINER" >/dev/null 2>&1 || true
    CURRENT_CONTAINER=
}

wait_database() {
    attempts=0
    while [ "$attempts" -lt 30 ]; do
        database_id=$(compose -f "$REPO_ROOT/deploy/postgres/compose.yaml" ps -q postgres)
        if [ -n "$database_id" ] && "$RUNTIME" exec "$database_id" \
            pg_isready -U "${POSTGRES_USER:-benchmarkdbuser}" -d "${POSTGRES_DB:-hello_world}" >/dev/null 2>&1; then
            return
        fi
        attempts=$((attempts + 1))
        sleep 2
    done
    echo "PostgreSQL did not become ready." >&2
    return 1
}

trap stop_current EXIT INT TERM

if ! command -v "$RUNTIME" >/dev/null 2>&1; then
    echo "Container runtime not found: $RUNTIME" >&2
    exit 1
fi

"$SCRIPT_DIR/validate_environment.sh"
if [ "${BENCHMARK_DATABASE_MANAGED:-1}" = 1 ]; then
    compose -f "$REPO_ROOT/deploy/postgres/compose.yaml" up -d
    wait_database
fi

cd "$REPO_ROOT"
cargo build --release --bin orchestrator

manifests=$(find frameworks -name framework.toml -print | sort)
for manifest in $manifests; do
    context=$(dirname "$manifest")
    name=$(awk -F '"' '/^name = / { print $2; exit }' "$manifest")
    port=$(awk -F ' = ' '/^port = / { print $2; exit }' "$manifest")
    [ -n "$name" ] && [ -n "$port" ] || continue
    for libc in $LIBCS; do
        tag="framework-benchmark/${name}:${libc}"
        container="framework-benchmark-${name}-${libc}"
        output="results/${name}/${libc}"
        if ! CONTAINER_RUNTIME="$RUNTIME" "$SCRIPT_DIR/build_container.sh" "$context" "$tag" "$libc"; then
            FAILED=1
            continue
        fi
        CURRENT_CONTAINER=$container
        if ! "$RUNTIME" run -d --rm --name "$container" --network host \
            --label framework-benchmark.managed=true -e DATABASE_URL="$DATABASE_URL" \
            -e BENCHMARK_WORKERS="$BENCHMARK_WORKERS" "$tag"; then
            FAILED=1
            CURRENT_CONTAINER=
            continue
        fi
        if ! BENCHMARK_MANAGE_CONTAINER=0 BENCHMARK_LIBC="$libc" \
            "$REPO_ROOT/target/release/orchestrator" "$context" "$output"; then
            FAILED=1
        fi
        stop_current
    done
done

[ "$FAILED" -eq 0 ] || exit 1
echo "All framework benchmark runs completed."
