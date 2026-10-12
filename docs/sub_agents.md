# Sub-Agent Collaboration and Workflow System

## Overview
This document defines the agent coordination protocol for Antigravity, OpenCode, and Codex across `framework-benchmark`. The architecture coordinates a 4-role development team with specialized domain agents.

## Core Team Roles

```text
[User Request]
       |
       v
  [Architect]       (Read-only: scoped plan, boundaries, <= 200 LOC verification)
       |
       v
  [Developer]       (Implementation: DoD, FP, Linux style, unit tests)
       |
       v
   [Reviewer]       (Read-only: latency validity, allocations, conventions)
       |
       v
  [Integrator]      (Verification: full test suite, lint gates, PR creation)
```

### 1. Architect (Read-Only)
- Evaluates the request against [AGENTS.md](../AGENTS.md), [CODING_CONVENTION.md](../CODING_CONVENTION.md), and [CODEBASE.md](../CODEBASE.md).
- Deconstructs tasks into independent slices, planning file structures to stay under 200 lines.
- States explicit assumptions, ownership boundaries, and acceptance criteria.
- Never writes or edits code.

### 2. Developer
- Implements the assigned slice following Data-Oriented Design, Functional Programming, and Linux coding conventions.
- Consults specialized domain agents for low-level guidance.
- Writes focused unit tests alongside code.
- Reports all modified files, executed commands, and actual results to the team.

### 3. Reviewer (Read-Only)
- Audits code against:
  - Hard limit of 200 lines per file.
  - Strict zero-emoji enforcement across all touched files.
  - Zero allocation on measurement hot paths.
  - Flat control flow with max 3 levels of indentation.
  - Clean separation of concerns (no mixed I/O and calculation).
- Returns concrete findings with file and line references.

### 4. Integrator
- Merges completed slices and resolves boundary conflicts.
- Executes full validation: `cargo fmt --check`, `cargo clippy`, `cargo test`, and `sh scripts/check_conventions.sh`.
- Prepares pull requests and updates documentation.

## Domain Specialists
- **orchestrator**: Owns the runner CLI, container/process lifecycle, and CPU pinning.
- **load_gen**: Owns async HTTP/WS load generation, pipelining, and HDR histogram latency recording.
- **collector**: Owns Linux cgroups v2 metrics, `/proc/statm` polling, and socket health tracking.
- **reporter**: Owns results serialization, Markdown table rendering, and SVG performance graphs.
- **frameworks_harness**: Owns `framework.toml` validation and framework source implementations under `frameworks/`.
- **docker_nix**: Owns dual-libc container definitions (`Dockerfile.glibc`, `Dockerfile.musl`) and Nix flakes.
- **verification**: Owns automated regression runs, convention auditing, and CI workflow scripts.
