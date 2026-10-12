# Codex Agent Team Workflow

Codex custom agents in `.codex/agents/` represent our developer team and specialized domain roles for `framework-benchmark`. The matching OpenCode workflow agents live in `.opencode/agents/`. Root project directives remain in `AGENTS.md`.

## Procedure

1. **architect**: Reads the request, project architecture in `CODEBASE.md`, and conventions in `CODING_CONVENTION.md`. Defines explicit assumptions, ownership boundaries, a plan obeying the 200 LOC ceiling, and verifiable acceptance criteria. Does not write code.
2. **developer**: Implements the assigned slice following Data-Oriented Design, Functional Programming, Linux kernel style, and zero emojis. Consults domain specialists for low-level subsystem expertise.
3. **reviewer**: Audits the resulting changes, verifies absence of allocations on hot paths, confirms the 200 LOC limit, and inspects test coverage. Reports actionable findings with file and line references.
4. **integrator**: Reconciles cross-subsystem boundaries, runs verification suites (`cargo fmt`, `cargo clippy`, `cargo test`, `sh scripts/check_conventions.sh`), and prepares the final deliverable.

## Specialist Routing
- Subsystem orchestration and process isolation: `orchestrator`
- Async traffic generation and HDR histograms: `load_gen`
- Linux `/proc` and cgroups v2 metrics: `collector`
- Performance summaries, charts, and Markdown: `reporter`
- Framework definitions and `framework.toml`: `frameworks_harness`
- Docker dual-libc images and Nix derivations: `docker_nix`
- Quality gates, test execution, and convention audit: `verification`
