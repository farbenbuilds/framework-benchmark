#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd "$script_dir/.." && pwd)

"$script_dir/validate_environment.sh"
cd "$repo_root"
cargo build --release --bin orchestrator
exec ./target/release/orchestrator frameworks results
