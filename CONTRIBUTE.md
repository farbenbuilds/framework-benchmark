# Contributing to Framework Benchmark

Framework Benchmark welcomes contributions from framework authors, performance engineers, and runtime developers across all programming languages.

Read this document, [AGENTS.md](AGENTS.md), [CODING_CONVENTION.md](CODING_CONVENTION.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing. Security vulnerabilities must be reported privately as described in [SECURITY.md](SECURITY.md).

## 1. Engineering Principles and Hard Rules
1. **Strict File Limit (200 Lines)**: No file of any type (source code, markdown, configuration, scripts) may exceed 200 lines. Break large files into modular components.
2. **Zero Emojis**: Emojis are strictly banned from all source code, documentation, comments, commit messages, and PRs.
3. **Linux Coding Style**: Adhere to `snake_case` naming for files, functions, and variables; early returns with guard clauses; maximum 3 levels of indentation; and space after control flow keywords.
4. **Pragmatic Data-Oriented Design**: Store measurement samples in contiguous Structure of Arrays (SoA). Pre-allocate memory buffers before benchmark windows to eliminate hot-path allocations.
5. **Pragmatic Functional Programming**: Keep calculations, metric aggregations, and parsers pure and free of side effects. Avoid object-oriented hierarchies and hidden state.
6. **Minimal Comments**: Never describe what the code does; explain only the non-obvious "why" in 1 line (maximum 2 lines).

## 2. Submitting a New Framework
To submit a new framework for benchmarking:
1. Create a dedicated directory under `frameworks/<category>/<language>/<framework_name>/`.
2. Provide a valid `framework.toml` manifest specifying capabilities, runtime ports, and build recipes.
3. Provide working build definitions for both libc variants:
   - `Dockerfile.glibc` (Debian or Ubuntu minimal base)
   - `Dockerfile.musl` (Alpine Linux or musl-distroless base)
   - Optional: `flake.nix` with `server-glibc` and `server-musl` packages.
4. Implement standard benchmark endpoints:
   - `/healthz` (readiness probe returning `200 OK`)
   - `/plaintext` (`Hello, World!`, `text/plain`)
   - `/json` (`{"message":"Hello, World!"}`, `application/json`)
   - Additional endpoints according to declared capabilities in `framework.toml` (`/db`, `/queries`, `/fortunes`, `/updates`).
5. Ensure the framework builds cleanly, respects graceful shutdown signals, and cleans up resources.

## 3. Pull Request and Verification Process
Before submitting a pull request:
1. Install and run pre-commit hooks: `pre-commit run --all-files`.
2. Run `cargo fmt --check` across all crates.
3. Run `cargo clippy --all-targets -- -D warnings`.
4. Run `cargo test --workspace`.
5. Run `sh scripts/check_conventions.sh` to ensure compliance with line limits, naming, and zero-emoji rules.
6. Follow the [Commit Convention](COMMIT_CONVENTION.md) for all commit messages.
