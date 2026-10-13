#!/usr/bin/env sh
set -eu

state_dir=${BENCHMARK_STATE_DIR:-/var/lib/framework-benchmark}

usage() {
    echo 'usage: sudo scripts/setup_linux.sh --apply' >&2
}

install_apt() {
    apt-get update
    apt-get install -y build-essential ca-certificates curl git iproute2 iputils-ping \
        linux-tools-generic docker.io docker-compose-v2
}

install_dnf() {
    dnf install -y gcc gcc-c++ make ca-certificates curl git iproute iputils \
        perf docker docker-compose-plugin
}

install_packages() {
    if command -v apt-get >/dev/null 2>&1; then
        install_apt
        return
    fi
    if command -v dnf >/dev/null 2>&1; then
        install_dnf
        return
    fi
    echo 'Unsupported package manager; install Docker, Compose, perf, and build tools.' >&2
    exit 1
}

save_sysctls() {
    umask 077
    mkdir -p "$state_dir"
    [ -f "$state_dir/sysctl.before" ] && return
    : > "$state_dir/sysctl.before"
    for key in net.core.somaxconn net.ipv4.tcp_max_syn_backlog \
        net.ipv4.ip_local_port_range net.ipv4.tcp_tw_reuse fs.file-max \
        kernel.perf_event_paranoid; do
        value=$(sysctl -n "$key") || continue
        printf '%s=%s\n' "$key" "$value" >> "$state_dir/sysctl.before"
    done
}

apply_tuning() {
    sysctl -w net.core.somaxconn=65535
    sysctl -w net.ipv4.tcp_max_syn_backlog=65535
    sysctl -w net.ipv4.ip_local_port_range='1024 65535'
    sysctl -w net.ipv4.tcp_tw_reuse=1
    sysctl -w fs.file-max=2097152
    sysctl -w kernel.perf_event_paranoid=-1
}

set_governor() {
    for governor in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        [ -f "$governor" ] || continue
        printf '%s\n' performance > "$governor" || true
    done
}

if [ "$#" -ne 1 ] || [ "$1" != --apply ]; then
    usage
    exit 2
fi
if [ "$(id -u)" -ne 0 ]; then
    echo 'setup_linux.sh --apply must run as root.' >&2
    exit 1
fi

install_packages
save_sysctls
apply_tuning
set_governor
if command -v systemctl >/dev/null 2>&1; then
    systemctl enable --now docker
fi
echo "Host tuning applied. Restore with BENCHMARK_STATE_DIR=$state_dir scripts/cleanup_environment.sh --apply."
