# Framework Benchmark Roadmap

Welcome to the **Framework Benchmark** project roadmap. This document provides the high-level roadmap and directs developers and autonomous agents to detailed architectural specifications and phase checklists.

The project evaluates HTTP and WebSocket server and client frameworks across languages and runtimes on Linux, with parallel evaluation under **glibc** and **musl**.

## Current status

The Docker-based dual-libc baseline is implemented, but a result is publishable only after the full selected matrix runs on an isolated Linux or OpenStack host. The repository's completed milestones describe delivered code, not proof of a reproducible performance result.

## Publication checklist

1. Use a dedicated cgroups v2 Linux host and record its CPU topology.
2. Build and run both glibc and musl for every published framework row.
3. Keep raw JSON, revision, host details, workload settings, and image provenance together.
4. Review generated results before `scripts/commit_benchmark_results.sh --apply`; pushing is explicit.

## Roadmap index

1. **Architecture & Test Categories**: [docs/roadmap/overview.md](docs/roadmap/overview.md) [Completed]
   - System design, runner workflow, workload matrix, and captured metrics.
2. **Phase 1: Project Foundation & Specification Standards**: [docs/roadmap/phase1_foundation.md](docs/roadmap/phase1_foundation.md) [Completed]
   - Workspace setup, `framework.toml` manifest schema, and result data models.
3. **Phase 2: Dual-Libc Infrastructure & Engine Core**: [docs/roadmap/phase2_load_engine.md](docs/roadmap/phase2_load_engine.md) [Completed]
   - Nix flakes, Docker multi-stage templates, Tokio/Httparse load generator, and cgroups v2 harvester.
4. **Phase 3: Framework Registry & Baseline Implementations**: [docs/roadmap/phase3_frameworks.md](docs/roadmap/phase3_frameworks.md) [Completed]
   - Framework registry populated with 32 candidate frameworks across 10 languages (Zig, Rust, Go, C#, C++, JS/TS, Ruby, Java, PHP, Python) with dual-libc Dockerfiles and valid manifests.
5. **Phase 4: Aggregation, Metrics Processing & Markdown Engine**: [docs/roadmap/phase4_reporter.md](docs/roadmap/phase4_reporter.md) [Completed]
   - Pure aggregation logic, SVG bar charts, append-only historical index, and Markdown table generator.
6. **Phase 5: Bare-Metal Deployment & GitHub CI/CD**: [docs/roadmap/phase5_deployment.md](docs/roadmap/phase5_deployment.md) [Completed]
   - Linux host tuning (`setup_host.sh`), verification (`verify_host.sh`), runner script (`run_benchmarks.sh`), and GitHub CI/CD workflows (`lint.yml`, `benchmark.yml`).
7. **End-to-End Suite Integration & Dynamic Reporting**: [Completed]
   - Asynchronous background resource sampling during live measurements (`BackgroundSampler`).
   - Capability-driven multi-workload execution across endpoints (`run_framework_suite`).
   - In-place dynamic `README.md` benchmark leaderboard updater (`update_readme_leaderboard`).
8. **Measurement Accuracy & Boundary Hardening**: [Completed]
   - HDR Histogram coordinated omission compensation (`record_micros_corrected`).
   - Worker connection resilience with zero concurrency drop on socket closure.
   - High-precision wall-clock measurement duration for exact RPS calculation.
   - Dual-source telemetry sampling (`/proc` and cgroups v2) integrated into orchestrator runner.
9. **Verification & Acceptance Criteria**: [docs/roadmap/acceptance_criteria.md](docs/roadmap/acceptance_criteria.md) [Completed]
   - Quality gates, 200 LOC ceiling, zero emojis, zero-allocation hot paths, and reproducibility.

## Next milestones

- Host-managed cgroups and CPU sets for complete collector attribution.
- Explicit paced, pipelined, connection-churn, database, and WebSocket profiles.
- Full-matrix execution on a labelled self-hosted runner; deterministic smoke validation in hosted CI.
- Immutable image and framework input pinning before historical comparison.

See [docs/how_to_use.md](docs/how_to_use.md) for the supported host workflow.

## Core Rules for Agents and Developers
- Always apply Data-Oriented Design (DoD) and Functional Programming (FP).
- Enforce the 200-line hard limit per file.
- Follow Linux kernel coding style and ban all emojis everywhere.
- Read [AGENTS.md](AGENTS.md) and [CODING_CONVENTION.md](CODING_CONVENTION.md) before making changes.
