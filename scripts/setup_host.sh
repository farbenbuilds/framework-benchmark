#!/bin/sh
set -eu

# Verify script is run as root
if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: setup_host.sh must be executed as root." >&2
    exit 1
fi

echo "==> Configuring CPU scaling governor to performance..."
if command -v cpupower >/dev/null 2>&1; then
    cpupower frequency-set -g performance
elif [ -d /sys/devices/system/cpu/cpu0/cpufreq ]; then
    for gov in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo "performance" > "$gov"
    done
fi

echo "==> Applying sysctl network stack tuning..."
cat << 'EOF' > /etc/sysctl.d/99-benchmark.conf
# Maximum socket listen backlog
net.core.somaxconn = 65535
# Maximum socket receive queue backlog
net.core.netdev_max_backlog = 65535
# Enable fast TCP connection recycling
net.ipv4.tcp_tw_reuse = 1
# TCP memory limits
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
# Buffer ceilings
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
EOF

sysctl --system >/dev/null 2>&1 || true

echo "==> Host tuning applied successfully."
