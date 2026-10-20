# Verification and Acceptance Criteria

## Quality Gates and Standards

Every benchmark phase and pull request must satisfy the following acceptance criteria prior to merging:

### 1. Code Quality Gates
- **Formatting**: `cargo fmt --check` succeeds with zero warnings or edits.
- **Clippy**: `cargo clippy --all-targets -- -D warnings` reports zero lints.
- **Convention Audit**: `sh scripts/check_conventions.sh` passes:
  - Every file in the repository contains at most 200 lines.
  - Zero emojis exist in code, comments, documentation, commit messages, or outputs.
  - File and folder names adhere strictly to `snake_case`.
  - Zero `TODO`, `FIXME`, `XXX`, or `HACK` slop markers remain.
- **Unit and Integration Tests**: `cargo test` passes cleanly.

### 2. Performance and Measurement Integrity
- **Zero Allocations on Hot Paths**: Load generation and telemetry loops must not allocate heap memory during active measurement sampling.
- **Coordinated Omission**: Paced workloads must use HDR histogram correction with constant-rate scheduling to capture queue wait time. Closed-loop profiles record service latency and must be labeled as closed-loop; do not present them as open-loop client latency.
- **Dual-Libc Validation**: Frameworks must provide verified build paths for both `glibc` and `musl`.
- **Reproducibility**: Compare repeated runs only after recording identical revision, image inputs, CPU topology, traffic placement, and workload settings.

### 3. Stability and Cleanliness
- **Resource Leaks**: Benchmark child containers and processes must be completely cleaned up; no orphan processes, dangling Docker containers, or unmounted cgroups remain.
- **Graceful Shutdown**: Framework containers must terminate within 5 seconds upon receiving `SIGTERM`.
