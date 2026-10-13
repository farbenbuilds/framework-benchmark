#!/usr/bin/env sh
set -eu

PID=${1:?usage: collect_ebpf.sh PID DURATION_SECS OUTPUT_FILE}
DURATION=${2:?usage: collect_ebpf.sh PID DURATION_SECS OUTPUT_FILE}
OUTPUT=${3:?usage: collect_ebpf.sh PID DURATION_SECS OUTPUT_FILE}
SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)

case $PID in
    *[!0-9]* | '') echo "PID must be numeric." >&2; exit 2 ;;
esac
case $DURATION in
    *[!0-9]* | '') echo "DURATION_SECS must be numeric." >&2; exit 2 ;;
esac

if ! command -v bpftrace >/dev/null 2>&1; then
    echo "bpftrace is required for eBPF collection." >&2
    exit 1
fi

mkdir -p "$(dirname "$OUTPUT")"
bpftrace -q -c "sleep $DURATION" "$SCRIPT_DIR/ebpf_metrics.bt" "$PID" > "$OUTPUT"
