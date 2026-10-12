# Roadmap Phase 2: Dual-Libc Infrastructure and Engine Core

## Overview
Phase 2 builds the dual-libc environment support (Docker & Nix) and the core Rust engines: load generation and cgroups v2 resource monitoring.

## Tasks and Checklist

### 2.1 Dual-Libc Container Infrastructure
- [ ] Create `templates/docker/Dockerfile.glibc` using Debian Bookworm Slim with multi-stage compilation.
- [ ] Create `templates/docker/Dockerfile.musl` using Alpine Linux with static linking or musl runtime.
- [ ] Provide Docker build helper scripts enforcing no caching drift.
- [ ] Create `templates/nix/flake.nix` exposing `server-glibc` and `server-musl` derivations.

### 2.2 High-Throughput Load Generator (`crates/load-gen`)
- [ ] Implement asynchronous HTTP/1.1 and HTTP/2 client using Tokio and Hyper.
- [ ] Implement constant-rate scheduled request issuance to eliminate coordinated omission.
- [ ] Integrate HDR Histogram for sub-microsecond latency recording up to 100 seconds.
- [ ] Add HTTP/1.1 pipelining driver supporting pipeline depths of 1, 16, and 64.
- [ ] Add WebSocket echo benchmark client measuring bi-directional frame round trips.
- [ ] Pre-allocate all memory buffers before starting load phases (zero allocation hot path).

### 2.3 Resource Harvester (`crates/collector`)
- [ ] Implement Linux `/proc/[pid]/statm` and `/proc/[pid]/status` readers.
- [ ] Implement Linux cgroups v2 controller monitor (`memory.current`, `memory.peak`, `cpu.stat`).
- [ ] Collect voluntary and involuntary context switches from `/proc/[pid]/status`.
- [ ] Sample metrics at 100ms intervals using a dedicated housekeeping core.

### 2.4 Orchestrator Engine (`crates/orchestrator`)
- [ ] Implement process lifecycle manager using Docker Unix domain socket (`bollard`) and direct Nix execution.
- [ ] Implement readiness poller for `/healthz` with exponential backoff.
- [ ] Enforce 15-second Warmup phase and 10-second Cooldown phase.
- [ ] Enforce CPU affinity pinning via `sched_setaffinity`.
