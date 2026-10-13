#!/usr/bin/env sh
set -eu

strict=0
if [ "$#" -eq 1 ] && [ "$1" = --require-dedicated ]; then
    strict=1
elif [ "$#" -ne 0 ]; then
    echo 'usage: verify_host.sh [--require-dedicated]' >&2
    exit 2
fi

failed=0

report() {
    level=$1
    shift
    printf '%s: %s\n' "$level" "$*"
    [ "$level" != FAIL ] || failed=1
}

require() {
    if [ "$strict" -eq 1 ]; then
        report FAIL "$*"
    else
        report WARN "$*"
    fi
}

validate_cpu_set() {
    name=$1
    value=$2
    if printf '%s\n' "$value" | grep -Eq '^[0-9]+(-[0-9]+)?(,[0-9]+(-[0-9]+)?)*$'; then
        taskset -c "$value" true >/dev/null 2>&1 && return
    fi
    require "$name must be an allowed CPU list"
}

expand_cpu_set() {
    printf '%s\n' "$1" | tr ',' '\n' | while IFS= read -r range; do
        case $range in
            *-*) seq "${range%-*}" "${range#*-}" ;;
            *) printf '%s\n' "$range" ;;
        esac
    done
}

check_governor() {
    governor=/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
    [ -f "$governor" ] || {
        require 'CPU governor is unavailable'
        return
    }
    value=$(cat "$governor")
    [ "$value" = performance ] && report PASS 'CPU governor is performance' || \
        require "CPU governor is $value, expected performance"
}

check_cgroup_v2() {
    grep -q 'cgroup2' /proc/mounts && report PASS 'cgroups v2 is mounted' || \
        require 'cgroups v2 is not mounted'
}

check_backlog() {
    value=$(cat /proc/sys/net/core/somaxconn)
    [ "$value" -ge 65535 ] && report PASS "socket backlog is $value" || \
        require "socket backlog is $value, expected at least 65535"
}

check_dedicated() {
    target=${BENCHMARK_TARGET_CPUS:-}
    load=${BENCHMARK_LOAD_CPUS:-}
    parent=${BENCHMARK_CGROUP_PARENT:-}
    command -v taskset >/dev/null 2>&1 || require 'taskset is required'
    [ -n "$target" ] || require 'BENCHMARK_TARGET_CPUS is required'
    [ -n "$load" ] || require 'BENCHMARK_LOAD_CPUS is required'
    [ -n "$parent" ] || require 'BENCHMARK_CGROUP_PARENT is required'
    [ -z "$target" ] || validate_cpu_set BENCHMARK_TARGET_CPUS "$target"
    [ -z "$load" ] || validate_cpu_set BENCHMARK_LOAD_CPUS "$load"
    if printf '%s\n' "$target,$load" | grep -Eq '^[0-9]+(-[0-9]+)?(,[0-9]+(-[0-9]+)?)*$'; then
        overlap=$( { expand_cpu_set "$target"; expand_cpu_set "$load"; } | sort -n | uniq -d)
        [ -z "$overlap" ] || require 'target and load CPU sets must not overlap'
    fi
}

check_governor
check_cgroup_v2
check_backlog
[ "$strict" -eq 0 ] || check_dedicated
[ "$failed" -eq 0 ] || exit 1
report PASS 'host verification completed'
