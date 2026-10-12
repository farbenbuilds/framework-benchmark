#!/usr/bin/env sh
set -eu

STATE_DIR="${BENCHMARK_STATE_DIR:-/var/lib/framework-benchmark}"
APPLY=0

if [ "${1:-}" = "--apply" ]; then
    APPLY=1
fi

if [ "$APPLY" -eq 0 ]; then
    echo "Dry run. Re-run as root with --apply to install packages and tune the host."
    exit 0
fi

if [ "$(id -u)" -ne 0 ]; then
    echo "setup_linux.sh --apply must run as root." >&2
    exit 1
fi

install_apt() {
    apt-get update
    apt-get install -y build-essential ca-certificates curl git libssl-dev pkg-config \
        linux-tools-generic docker.io
    apt-get install -y docker-compose-plugin || apt-get install -y docker-compose-v2
}

install_dnf() {
    dnf install -y gcc gcc-c++ make ca-certificates curl git openssl-devel \
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
    echo "Unsupported package manager; install Docker, Rust, perf, and build tools manually." >&2
    exit 1
}

install_rust() {
    if command -v cargo >/dev/null 2>&1; then
        return
    fi
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
}

save_sysctls() {
    mkdir -p "$STATE_DIR"
    [ -f "$STATE_DIR/sysctl.before" ] && return
    : > "$STATE_DIR/sysctl.before"
    for key in net.core.somaxconn net.ipv4.tcp_max_syn_backlog \
        net.ipv4.ip_local_port_range net.ipv4.tcp_tw_reuse fs.file-max \
        kernel.perf_event_paranoid; do
        printf '%s=%s\n' "$key" "$(sysctl -n "$key")" >> "$STATE_DIR/sysctl.before"
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

install_packages
install_rust
save_sysctls
apply_tuning
set_governor
systemctl enable --now docker 2>/dev/null || true
echo "Host setup complete. Re-login after adding your benchmark user to the docker group."
