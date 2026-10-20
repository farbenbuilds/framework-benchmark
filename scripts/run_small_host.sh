#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd "$script_dir/.." && pwd)
check_only=0
framework=cpeak

usage() {
    echo "usage: run_small_host.sh [--check-only] [framework-name]" >&2
}

for arg in "$@"; do
    case $arg in
        --check-only) check_only=1 ;;
        -h|--help) usage; exit 0 ;;
        -*) usage; exit 2 ;;
        *) framework=$arg ;;
    esac
done

cpus=$(nproc 2>/dev/null || echo 1)
memory_kb=$(awk '/MemTotal:/ { print $2 }' /proc/meminfo 2>/dev/null || echo 0)
memory_gb=$((memory_kb / 1024 / 1024))

if [ "$cpus" -lt 2 ]; then
    echo "Small-host profile needs at least 2 vCPUs; detected $cpus." >&2
    exit 1
fi
if [ "$memory_gb" -lt 6 ]; then
    echo "Small-host profile needs at least 6 GiB total RAM; detected ${memory_gb} GiB." >&2
    exit 1
fi
if ! grep -q cgroup2 /proc/mounts; then
    echo "Small-host profile needs cgroups v2 mounted." >&2
    exit 1
fi
if [ "${BENCHMARK_DATABASE_MANAGED:-0}" = 1 ]; then
    echo "Database workloads require a reachable PostgreSQL and DATABASE_URL." >&2
fi

export BENCHMARK_SMALL_HOST=1
export BENCHMARK_TARGET_CPUS=${BENCHMARK_TARGET_CPUS:-0}
export BENCHMARK_LOAD_CPUS=${BENCHMARK_LOAD_CPUS:-1}
export BENCHMARK_TARGET_WORKERS=${BENCHMARK_TARGET_WORKERS:-1}
export BENCHMARK_CONCURRENCY=${BENCHMARK_CONCURRENCY:-8}
export BENCHMARK_DURATION_SECS=${BENCHMARK_DURATION_SECS:-10}
export BENCHMARK_LIBC=${BENCHMARK_LIBC:-glibc}
export BENCHMARK_FRAMEWORK_FILTER=${BENCHMARK_FRAMEWORK_FILTER:-$framework}
export BENCHMARK_TARGET_MEMORY=${BENCHMARK_TARGET_MEMORY:-2g}
export BENCHMARK_PIDS_LIMIT=${BENCHMARK_PIDS_LIMIT:-128}
export BENCHMARK_CONTAINER_LOGS=${BENCHMARK_CONTAINER_LOGS:-0}
export BENCHMARK_DATABASE_MANAGED=${BENCHMARK_DATABASE_MANAGED:-0}

expand_cpu_set() {
    printf '%s\n' "$1" | tr ',' '\n' | while IFS= read -r range; do
        case $range in
            *-*) seq "${range%-*}" "${range#*-}" ;;
            *) printf '%s\n' "$range" ;;
        esac
    done
}

overlap=$( { expand_cpu_set "$BENCHMARK_TARGET_CPUS"; expand_cpu_set "$BENCHMARK_LOAD_CPUS"; } | sort -n | uniq -d)
if [ -n "$overlap" ]; then
    echo "BENCHMARK_TARGET_CPUS and BENCHMARK_LOAD_CPUS must not overlap." >&2
    exit 1
fi

echo "Small-host profile: ${cpus} vCPU, ${memory_gb} GiB RAM"
echo "Target: ${BENCHMARK_FRAMEWORK_FILTER} [${BENCHMARK_LIBC}]"
echo "CPUs: target=${BENCHMARK_TARGET_CPUS} load=${BENCHMARK_LOAD_CPUS}"
echo "Traffic: concurrency=${BENCHMARK_CONCURRENCY} duration=${BENCHMARK_DURATION_SECS}s"

if [ "$check_only" -eq 1 ]; then
    exit 0
fi

cd "$repo_root"
exec "$script_dir/run_benchmarks_all.sh"
