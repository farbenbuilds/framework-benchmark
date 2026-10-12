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
- **Coordinated Omission**: Latency reporting must utilize HDR histogram with constant-rate scheduling to capture actual queue wait times.
- **Dual-Libc Validation**: Frameworks must provide verified build paths for both `glibc` and `musl`.
- **Reproducibility**: Repeated runs of the same framework on identical hardware must achieve throughput numbers within a 3% variance window.

### 3. Stability and Cleanliness
- **Resource Leaks**: Benchmark child containers and processes must be completely cleaned up; no orphan processes, dangling Docker containers, or unmounted cgroups remain.
- **Graceful Shutdown**: Framework containers must terminate within 5 seconds upon receiving `SIGTERM`.
