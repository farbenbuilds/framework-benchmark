#!/usr/bin/env sh
set -eu

runtime=${CONTAINER_RUNTIME:-}
manage_database=${BENCHMARK_DATABASE_MANAGED:-1}
failed=0

fail() {
    echo "FAIL: $*" >&2
    failed=1
}

find_runtime() {
    [ -n "$runtime" ] && return
    if command -v docker >/dev/null 2>&1; then
        runtime=docker
        return
    fi
    if command -v podman >/dev/null 2>&1; then
        runtime=podman
        return
    fi
    fail 'Docker or Podman is required.'
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "$1 is required."
}

check_compose() {
    [ "$manage_database" = 1 ] || return
    "$runtime" compose version >/dev/null 2>&1 && return
    if [ "$runtime" = docker ] && command -v docker-compose >/dev/null 2>&1; then
        return
    fi
    fail "Compose support is required when BENCHMARK_DATABASE_MANAGED=1."
}

check_cgroup() {
    grep -q cgroup2 /proc/mounts || fail 'cgroups v2 is not mounted.'
}

find_runtime
require_command cargo
require_command git
if [ -n "$runtime" ]; then
    command -v "$runtime" >/dev/null 2>&1 || fail "Container runtime not found: $runtime."
    "$runtime" info >/dev/null 2>&1 || fail "Cannot communicate with $runtime."
    check_compose
fi
check_cgroup

memory_kb=$(awk '/MemAvailable:/ { print $2 }' /proc/meminfo 2>/dev/null || true)
[ "${memory_kb:-0}" -ge 1048576 ] || fail 'Less than 1 GiB of available memory.'
if command -v nproc >/dev/null 2>&1; then
    echo "CPUs available: $(nproc)"
fi

if [ "$failed" -ne 0 ]; then
    exit 1
fi
echo "Environment validation passed using $runtime."
