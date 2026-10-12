#!/bin/sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> Running host verification..."
"$SCRIPT_DIR/verify_host.sh"

echo "==> Compiling orchestrator release binary..."
cd "$REPO_ROOT"
cargo build --release --bin orchestrator

echo "==> Executing benchmark orchestrator..."
"$REPO_ROOT/target/release/orchestrator" "$REPO_ROOT/frameworks" "$REPO_ROOT/results"

echo "==> Benchmark suite execution completed."
