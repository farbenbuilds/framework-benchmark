# Framework Benchmark Roadmap: Overview

## Scope and Objectives
`framework-benchmark` is an open, reproducible, multi-language benchmark suite for HTTP and WebSocket servers and clients on Linux. It evaluates frameworks built against both GNU C Library (`glibc`) and `musl` libc.

## System Architecture
```text
+-----------------------------------------------------------------+
| GitHub Repository (farbenbuilds/framework-benchmark)            |
| - frameworks/ (framework sources, Dockerfiles, Nix flakes)      |
| - results/ (JSON performance artifacts)                         |
| - README.md (auto-updated leaderboard tables and graphs)        |
+--------------------------------+--------------------------------+
                                 | git clone / dispatch
                                 v
+-----------------------------------------------------------------+
| Dedicated Linux Benchmark Host                                  |
| +-------------------------------------------------------------+ |
| | Rust Orchestrator Engine                                    | |
| | - Isolation: CPU pinning (sched_setaffinity) & cgroups v2   | |
| | - Lifecycle: Build -> Spawn -> Healthz -> Warmup -> Sample  | |
| | - Measurement: Tokio/Hyper load gen + cgroups v2 collector  | |
| +-------------------------------------------------------------+ |
|                                |                                |
|                                v                                |
|             Results Engine (crates/reporter)                    |
|             - JSON artifact generation                          |
|             - Markdown summary table generation                 |
|             - Git commit and push back to GitHub                |
+-----------------------------------------------------------------+
```

## Benchmark Categories
1. **Plaintext (`/plaintext`)**: Tests raw protocol framing, parser overhead, and socket throughput with `Hello, World!`.
2. **JSON Serialization (`/json`)**: Tests JSON serialization and memory allocation with `{"message":"Hello, World!"}`.
3. **Single Database Query (`/db`)**: Tests async DB driver and connection pool overhead fetching a single row.
4. **Multiple Database Queries (`/queries?queries=N`)**: Stresses connection multiplexing fetching $N$ rows.
5. **Fortunes (`/fortunes`)**: Reads rows from DB, sorts them, and renders HTML with entity escaping.
6. **Data Updates (`/updates?queries=N`)**: Updates $N$ database records in a transaction and persists them.
7. **WebSocket Echo (`/ws-echo`)**: Measures round-trip frame latency and bi-directional message throughput.

## Captured Metrics
- **Throughput**: Requests per second (RPS), network transfer bandwidth (MB/s).
- **Latency Distribution**: Min, P50, P75, P90, P99, P99.9, P99.99, Max, and Standard Deviation (recorded via HDR Histogram with coordinated omission compensation).
- **System Resources**: Peak RSS, working set RAM, user vs system CPU %, page faults, and context switches.
- **Reliability**: Status code distribution (2xx, 4xx, 5xx), timeouts, resets, and dropped packets.
