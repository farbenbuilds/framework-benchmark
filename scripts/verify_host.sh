#!/bin/sh
set -eu

echo "==> Verifying host benchmarking conditions..."

# Check CPU scaling governor
if [ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
    gov=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)
    if [ "$gov" != "performance" ]; then
        echo "WARN: CPU governor is '$gov' (expected 'performance')."
    else
        echo "PASS: CPU governor is configured to 'performance'."
    fi
fi

# Check cgroups v2 mount
if grep -q "cgroup2" /proc/mounts; then
    echo "PASS: cgroups v2 unified hierarchy mounted."
else
    echo "WARN: cgroups v2 not detected in /proc/mounts."
fi

# Check TCP socket backlog
somax=$(cat /proc/sys/net/core/somaxconn)
if [ "$somax" -lt 1024 ]; then
    echo "WARN: net.core.somaxconn is $somax (recommend >= 65535 for high concurrency)."
else
    echo "PASS: net.core.somaxconn is $somax."
fi

echo "Host verification completed."
