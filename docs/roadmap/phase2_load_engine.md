# Roadmap Phase 2: Dual-Libc Infrastructure and Engine Core

## Overview
Phase 2 builds the dual-libc environment support (Docker & Nix) and the core Rust engines: load generation and cgroups v2 resource monitoring.

## Tasks and Checklist

### 2.1 Dual-Libc Container Infrastructure
- [x] Create `templates/docker/Dockerfile.glibc` using Debian Bookworm Slim with multi-stage compilation.
- [x] Create `templates/docker/Dockerfile.musl` using Alpine Linux with static linking or musl runtime.
- [ ] Provide Docker build helper scripts enforcing no caching drift.
- [x] Create `templates/nix/flake.nix` exposing `server-glibc` and `server-musl` derivations.

### 2.2 High-Throughput Load Generator (`crates/load-gen`)
- [x] Implement asynchronous HTTP/1.1 client using Tokio and Httparse.
- [ ] Implement constant-rate scheduled request issuance to eliminate coordinated omission.
- [x] Integrate HDR Histogram for sub-microsecond latency recording up to 100 seconds.
- [x] Add HTTP/1.1 pipelining driver supporting pipeline depths of 1, 16, and 64.
- [ ] Add WebSocket echo benchmark client measuring bi-directional frame round trips.
- [x] Pre-allocate all memory buffers before starting load phases (zero allocation hot path).

### 2.3 Resource Harvester (`crates/collector`)
- [x] Implement Linux `/proc/[pid]/statm` and `/proc/[pid]/status` readers.
- [x] Implement Linux cgroups v2 controller monitor (`memory.current`, `memory.peak`, `cpu.stat`).
- [x] Collect voluntary and involuntary context switches from `/proc/[pid]/status`.
- [x] Sample metrics aggregation engine for RSS and CPU utilization.

### 2.4 Orchestrator Engine (`crates/orchestrator`)
- [x] Implement test execution lifecycle tying together manifest, load engine, and telemetry.
- [x] Implement readiness poller for `/healthz` with exponential backoff.
- [x] Enforce 15-second Warmup phase and measurement intervals.
- [ ] Enforce CPU affinity pinning via `sched_setaffinity`.
