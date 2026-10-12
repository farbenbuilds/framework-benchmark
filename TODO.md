# Multi-Language Server/Client Framework Benchmark Roadmap

Welcome to the **Framework Benchmark** project roadmap. This document provides the high-level roadmap and directs developers and autonomous agents to detailed architectural specifications and phase checklists.

The project evaluates HTTP and WebSocket server and client frameworks across languages and runtimes on Linux, with parallel evaluation under **glibc** and **musl**.

## Roadmap Index & Status

1. **Architecture & Test Categories**: [docs/roadmap/overview.md](docs/roadmap/overview.md)
   - System design, runner workflow, workload matrix, and captured metrics.
2. **Phase 1: Project Foundation & Specification Standards**: [docs/roadmap/phase1_foundation.md](docs/roadmap/phase1_foundation.md) [Completed]
   - Workspace setup, `framework.toml` manifest schema, and result data models.
3. **Phase 2: Dual-Libc Infrastructure & Engine Core**: [docs/roadmap/phase2_load_engine.md](docs/roadmap/phase2_load_engine.md) [Completed]
   - Nix flakes, Docker multi-stage templates, Tokio/Httparse load generator, and cgroups v2 harvester.
4. **Phase 3: Framework Registry & Baseline Implementations**: [docs/roadmap/phase3_frameworks.md](docs/roadmap/phase3_frameworks.md) [In Progress]
   - Framework registry structure, Rust Axum baseline, and Go net/http baseline (dual-libc Dockerfiles).
5. **Phase 4: Aggregation, Metrics Processing & Markdown Engine**: [docs/roadmap/phase4_reporter.md](docs/roadmap/phase4_reporter.md) [In Progress]
   - Leaderboard aggregation, JSON serialization, and Markdown table renderer.
6. **Phase 5: Bare-Metal Deployment & GitHub CI/CD**: [docs/roadmap/phase5_deployment.md](docs/roadmap/phase5_deployment.md) [Pending]
   - Linux host tuning (`isolcpus`, sysctl), runner lifecycle script, and GitHub Actions PR checks.
7. **Verification & Acceptance Criteria**: [docs/roadmap/acceptance_criteria.md](docs/roadmap/acceptance_criteria.md)
   - Quality gates, 200 LOC ceiling, zero emojis, zero-allocation hot paths, and reproducibility.

## Core Rules for Agents and Developers
- Always apply Data-Oriented Design (DoD) and Functional Programming (FP).
- Enforce the 200-line hard limit per file.
- Follow Linux kernel coding style and ban all emojis everywhere.
- Read [AGENTS.md](AGENTS.md) and [CODING_CONVENTION.md](CODING_CONVENTION.md) before making changes.
