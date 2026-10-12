# Framework Benchmark CI/CD Pipeline

This document defines the automated verification and benchmarking pipeline for `framework-benchmark`.

## 1. Overview
The platform uses a two-tier pipeline:
1. **GitHub Actions CI**: Fast, deterministic smoke and conformance testing for PRs and commits.
2. **Dedicated Benchmark Runner**: Heavyweight, isolated benchmark execution on bare-metal Linux hardware with automatic result commits back to GitHub.

## 2. GitHub Actions CI (PR & Push Validation)
Every pull request and merge to `main` executes the following automated checks:

### Step 1: Code Quality and Lint Gate
- Format check: `cargo fmt --check`
- Clippy analysis: `cargo clippy --all-targets -- -D warnings`
- Convention and hygiene check: `sh scripts/check_conventions.sh`
  - Enforces the 200 LOC per file hard ceiling.
  - Verifies zero emojis across all tracked files.
  - Verifies `snake_case` naming for files and directories.
  - Rejects `TODO`, `FIXME`, and marker slop.

### Step 2: Test Suite
- Unit tests: `cargo test --lib`
- Integration tests: `cargo test --test '*'`
- Manifest schema validation: validates syntax of all `framework.toml` files in `frameworks/`.

### Step 3: Container and Packaging Conformance
- Dual-libc container build verification:
  - Validates `Dockerfile.glibc` builds cleanly.
  - Validates `Dockerfile.musl` builds cleanly.
- Healthcheck smoke test: runs container, polls `/healthz`, and executes a single curl request to `/plaintext`.

## 3. Dedicated Benchmark Host Lifecycle
When triggered on a schedule or via repository dispatch:

```text
1. git checkout main && git pull origin main
2. Apply kernel sysctl configurations and set CPU governor to performance
3. Configure cgroups v2 controller hierarchy
4. Execute orchestrator:
   cargo run --release -p orchestrator -- --all-frameworks
5. For each framework:
   a. Build glibc and musl images
   b. Launch container on pinned cores (Cores 2-7)
   c. Poll /healthz until ready
   d. Warmup run (15s JIT stabilization)
   e. Measurement run (Load gen on Cores 8-15 + Collector on cgroups v2)
   f. Cooldown period (10s TCP socket drain)
   g. Container teardown and volume wipe
6. Write raw JSON snapshots to results/data/<run_id>.json
7. Update leaderboard tables and graphs in README.md
8. git commit -m "chore(benchmark): update results [skip ci]" && git push origin main
```

## 4. Quality Rules for CI Workflows
- Workflows must never disable lints or weaken thresholds to pass.
- Any change modifying performance metrics must include rationale in the commit message.
- Emojis are strictly banned from all workflow logs and commit templates.
