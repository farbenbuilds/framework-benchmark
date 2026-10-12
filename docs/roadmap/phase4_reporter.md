# Roadmap Phase 4: Aggregation, Metrics Processing, and Markdown Engine

## Overview
Phase 4 implements data reduction, latency percentile calculation, JSON archiving, and Markdown generation in `crates/reporter`.

## Tasks and Checklist

### 4.1 Pure Metrics Aggregation Logic (`crates/reporter/src/logic/`)
- [ ] Implement pure transformation functions converting raw HDR histogram samples into percentiles (P50, P75, P90, P99, P99.9, P99.99).
- [ ] Calculate transfer rates, throughput averages, and request loss percentages.
- [ ] Correlate cgroups v2 resource samples with load phases to calculate peak RSS and average CPU utilization.
- [ ] Strictly separate computation from formatting and I/O.

### 4.2 JSON Artifact Serializer (`crates/reporter/src/serializers/`)
- [ ] Serialize full run data conforming to `docs/schemas/result_schema.json`.
- [ ] Store timestamped runs under `results/data/<year>/<month>/<run_id>.json`.
- [ ] Maintain an append-only index file `results/index.json` tracking historical runs.

### 4.3 Markdown Leaderboard Generator (`crates/reporter/src/formatters/`)
- [ ] Generate GitHub-flavored Markdown tables for `README.md`.
- [ ] Rank frameworks by throughput (RPS) and P99 latency within workload categories (Plaintext, JSON, DB queries, Fortunes, WebSocket).
- [ ] Highlight performance deltas between glibc and musl builds for each framework.
- [ ] Ensure all generated tables, headers, and logs are completely free of emojis.

### 4.4 Chart Plotting Engine (`crates/reporter/src/charts/`)
- [ ] Generate lightweight standalone SVG bar charts for Throughput vs P99 Latency.
- [ ] Plot memory footprint comparison graphs (Peak RSS vs Concurrency tier).
- [ ] Embed SVG assets into `results/charts/` referenced directly by `README.md`.
