# Roadmap Phase 4: Aggregation, Metrics Processing, and Markdown Engine

## Overview
Phase 4 implements data reduction, latency percentile calculation, JSON archiving, and Markdown generation in `crates/reporter`.

## Tasks and Checklist

### 4.1 Pure Metrics Aggregation Logic (`crates/reporter/src/logic.rs`)
- [x] Implement pure transformation and sorting functions for benchmark runs across categories.
- [x] Extract and format throughput (RPS), P99 latency (ms), and peak memory (MB).
- [x] Correlate dual-libc execution results (glibc vs musl).
- [x] Strictly separate computation from formatting and I/O.

### 4.2 JSON Artifact Serializer (`crates/reporter/src/archive.rs` & `index.rs`)
- [x] Serialize full run data conforming to `docs/schemas/result_schema.json`.
- [x] Store timestamped runs under `results/data/<year>/<month>/<run_id>.json`.
- [x] Maintain an append-only index file `results/index.json` tracking historical runs.

### 4.3 Markdown Leaderboard Generator (`crates/reporter/src/markdown.rs`)
- [x] Generate GitHub-flavored Markdown tables for results presentation.
- [x] Rank frameworks by throughput (RPS) and P99 latency within workload categories.
- [x] Highlight performance deltas between glibc and musl builds for each framework.
- [x] Ensure all generated tables, headers, and logs are completely free of emojis.

### 4.4 Chart Plotting Engine (`crates/reporter/src/chart.rs`)
- [x] Generate lightweight standalone SVG bar charts for Throughput vs P99 Latency.
- [x] Plot dual-libc visual comparison (glibc vs musl color accents).
- [x] Embed SVG assets referenced directly by `README.md`.
