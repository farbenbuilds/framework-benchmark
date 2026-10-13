#!/usr/bin/env sh
set -eu

TARGET_HOST="${BENCHMARK_TARGET_HOST:-127.0.0.1}"
RUNTIME="${CONTAINER_RUNTIME:-}"
FAILED=0

warn() {
    echo "WARN: $*" >&2
    FAILED=1
}

find_runtime() {
    if [ -n "$RUNTIME" ]; then
        return
    fi
    if command -v docker >/dev/null 2>&1; then
        RUNTIME=docker
        return
    fi
    if command -v podman >/dev/null 2>&1; then
        RUNTIME=podman
        return
    fi
    warn "Docker or Podman is required."
}

find_runtime
if [ -n "$RUNTIME" ] && ! "$RUNTIME" info >/dev/null 2>&1; then
    warn "Cannot communicate with $RUNTIME."
fi

if command -v ip >/dev/null 2>&1; then
    route=$(ip route get "$TARGET_HOST" 2>/dev/null || true)
    interface=$(printf '%s\n' "$route" | awk '/dev/ { for (i = 1; i <= NF; i++) if ($i == "dev") print $(i + 1) }')
    [ -n "$interface" ] && ip link show "$interface" | grep -q 'mtu ' || warn "No route or MTU for $TARGET_HOST."
else
    warn "iproute2 is required to inspect network settings."
fi

if command -v ping >/dev/null 2>&1; then
    ping -c 2 -W 1 "$TARGET_HOST" >/dev/null 2>&1 || warn "Cannot ping $TARGET_HOST."
fi

memory_kb=$(awk '/MemAvailable:/ { print $2 }' /proc/meminfo 2>/dev/null || true)
[ "${memory_kb:-0}" -ge 1048576 ] || warn "Less than 1 GiB of available memory."
command -v nproc >/dev/null 2>&1 && echo "CPUs available: $(nproc)"
grep -q cgroup2 /proc/mounts || warn "cgroups v2 is not mounted."

if command -v perf >/dev/null 2>&1; then
    perf stat -e cycles true >/dev/null 2>&1 || warn "Hardware perf counters are unavailable."
else
    warn "perf is not installed."
fi

if [ "$FAILED" -ne 0 ]; then
    echo "Environment validation found issues." >&2
    exit 1
fi

echo "Environment validation passed for $TARGET_HOST using $RUNTIME."
