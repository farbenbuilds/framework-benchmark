#!/usr/bin/env sh
set -eu

RUNTIME=${CONTAINER_RUNTIME:-docker}
IMAGE=${AUTOBAHN_IMAGE:-crossbario/autobahn-testsuite:25.10.1}
TARGET_URL=${AUTOBAHN_TARGET_URL:-ws://127.0.0.1:8080/ws}
RESULTS_DIR=${AUTOBAHN_RESULTS_DIR:-results/autobahn}
CONFIG=$(mktemp)

cleanup() {
    rm -f "$CONFIG"
}

trap cleanup EXIT HUP INT TERM

if ! command -v "$RUNTIME" >/dev/null 2>&1; then
    echo "Container runtime not found: $RUNTIME" >&2
    exit 1
fi
if ! printf '%s' "$TARGET_URL" | grep -Eq '^wss?://[^"\\[:space:]]+$'; then
    echo "AUTOBAHN_TARGET_URL must be a ws:// or wss:// URL without quotes or whitespace." >&2
    exit 2
fi

mkdir -p "$RESULTS_DIR"
printf '%s\n' '{' > "$CONFIG"
printf '  "servers": [{"agent": "framework-benchmark", "url": "%s"}],\n' "$TARGET_URL" >> "$CONFIG"
printf '%s\n' '  "outdir": "/reports",' >> "$CONFIG"
printf '%s\n' '  "cases": ["*"],' >> "$CONFIG"
printf '%s\n' '  "exclude-cases": [],' >> "$CONFIG"
printf '%s\n' '  "exclude-agent-cases": {}' >> "$CONFIG"
printf '%s\n' '}' >> "$CONFIG"

"$RUNTIME" run --rm --network host \
    --volume "$CONFIG:/config/fuzzingclient.json:ro" \
    --volume "$(cd "$RESULTS_DIR" && pwd):/reports" \
    "$IMAGE" wstest -m fuzzingclient -s /config/fuzzingclient.json
