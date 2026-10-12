---
name: framework-benchmark-workflow
description: Master high-performance workflow integrating specialized skills for the Framework Benchmark project.
---

# Framework Benchmark Master Workflow

This document defines the unified workflow integrating the specialized skills in `.agents/skills/*` to build an industrial-grade, deterministic benchmarking platform in Rust with Docker and Nix.

## 1. Principles and Mindset
- **Pragmatic Data-Oriented Design (`dod`)**: Structure memory around hardware access patterns. Use Structure of Arrays (SoA) for sample collection, contiguous slices, and hot/cold field splitting. Avoid pointer chasing and deep object graphs.
- **Pragmatic Functional Programming (`Pragmatic Functional Programming`)**: Separate pure data structures from transformation logic. Write pure functions for mathematical aggregations, latency percentiles, and protocol parsing. No hidden global state or mutating side effects in core logic.
- **Separation of Concerns (`separation-of-concerns`)**: Every module and file performs one isolated responsibility. Decouple data definitions, pure calculations, I/O and hardware interaction, and composition containers.
- **Strict File Limit (200 LOC)**: No file of any type may exceed 200 lines. Break large components into focused, single-purpose submodules.
- **Linux Kernel Coding Style**: Enforce `snake_case` naming for functions, variables, and filenames. Enforce early returns, guard clauses, shallow indentation (max 3 levels), and standard spacing after keywords.
- **Zero Emojis**: Emojis are strictly banned from all source code, documentation, comments, commit messages, and benchmark output reports.
- **Minimal "Why" Comments**: Never describe what the code does. If an approach is non-obvious, explain the rationale in exactly 1 line (maximum 2 lines).
- **Simplicity and Density (`ponytail`, `caveman`)**: Write the minimum code required to fulfill the contract. Deliver high signal-to-noise communication in plans and reports.
- **Live Documentation Lookup (`context7`)**: Retrieve current documentation and API specifications directly via Context7 when implementing framework adapters and low-level interfaces.

## 2. Core Architecture
- **Rust Benchmark Engine (`rust-best-practices`)**: Low-overhead asynchronous engine leveraging Tokio and Hyper with zero allocation on measurement hot paths.
- **Latency Precision**: Use HDR histograms with coordinated omission compensation to capture authentic latency percentiles from P50 to P99.99.
- **Resource Monitoring**: Sample Linux cgroups v2, `/proc/statm`, and socket allocations without perturbing the benchmarked process.
- **Dual-Libc Parity (`docker-patterns`, `nix-best-practices`)**: Provide parallel test configurations for both GNU C Library (`glibc`) and `musl` libc for every framework.

## 3. Engineering Phases
1. **Specification & Architecture**:
   - The Architect outlines ownership boundaries, acceptance criteria, and schema contracts before implementation starts.
   - Enforce the 200-line ceiling at the design phase by organizing submodules in advance.
2. **Implementation & Domain Slicing**:
   - The Developer implements changes in small, verifiable slices.
   - Pure functions are built first; I/O drivers are added at outer boundaries.
3. **Verification & Quality Gate**:
   - The Reviewer validates correctness, tests, conventions, and memory profiles.
   - Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `sh scripts/check_conventions.sh`.
4. **Integration & Release (`github-actions`)**:
   - The Integrator reconciles cross-cutting changes, resolves conflicts, and prepares the final pull request.
