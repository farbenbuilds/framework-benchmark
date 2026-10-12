# Contributing to Framework Benchmark

Framework Benchmark welcomes contributions from framework authors, performance engineers, and runtime contributors across all programming languages.

Please read this document, the [Code of Conduct](CODE_OF_CONDUCT.md), and the [Commit Convention](COMMIT_CONVENTION.md) before submitting code. Report security vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

---

## 1. Development Environment

The project is structured around a central **Rust** harness and polyglot framework suites orchestrated via **Docker** and **Nix**.

### Prerequisites
- **Rust Toolchain**: 1.85+ (`cargo`, `rustc`, `clippy`)
- **Docker**: Container runtime with multi-stage build support
- **Nix** (optional but recommended): Flakes-enabled Nix for reproducible builds and glibc/musl cross-compilation

---

## 2. Engineering Requirements & Principles

1. **Deterministic & Jitter-Free Runs**:
   - Benchmarks must run with pinned core affinities and memory quotas.
   - Load generation must correct for coordinated omission using HDR histograms.
2. **Dual-Libc Parity**:
   - Whenever feasible, framework contributions should provide both `glibc` and `musl` build recipes (`Dockerfile.glibc`, `Dockerfile.musl`, or Nix flake derivations).
3. **Strict Resource Cleanup**:
   - Processes, spawned child containers, temporary network sockets, and shared memory segments must be cleanly torn down after each run.
4. **Code Quality & Formatting**:
   - Rust code must be formatted with `cargo fmt --check` and pass `cargo clippy --all-targets -- -D warnings`.
   - Manifest files (`framework.toml`) must validate against the common schema.

---

## 3. Submitting a New Framework

To submit your framework for benchmarking:

1. Create a dedicated directory under `frameworks/<category>/<language>/<framework-name>/`.
2. Provide a valid `framework.toml` manifest specifying capabilities, build engine, and runtime ports.
3. Provide working build definitions (`Dockerfile.glibc`, `Dockerfile.musl`, or `flake.nix`).
4. Implement the standard benchmark endpoints:
   - `/healthz` (readiness probe)
   - `/plaintext` (`Hello, World!`)
   - `/json` (`{"message":"Hello, World!"}`)
   - Additional endpoints according to declared capabilities in `framework.toml` (e.g. `/db`, `/fortunes`, `/queries`).
5. Test locally to verify the container builds, passes `/healthz`, and completes a smoke test without errors.

---

## 4. Pull Requests & Conventional Commits

- Commit messages must follow the [Commit Convention](COMMIT_CONVENTION.md).
- Open PRs against the `main` branch.
- Complete the checklist in the [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md).
- Ensure all CI validation workflows pass.
