# Roadmap Phase 5: Bare-Metal Deployment and GitHub CI/CD

## Overview
Phase 5 automates host tuning, bare-metal runner scheduling, and GitHub pull request / push lifecycle automation.

## Tasks and Checklist

### 5.1 Host Provisioning and OS Tuning (`scripts/`)
- [ ] Implement `scripts/setup_host.sh` for Ubuntu/Debian bare-metal machines:
  - Configure CPU scaling governor to `performance`.
  - Isolate benchmark cores using Linux boot parameter `isolcpus=2-15`.
  - Apply sysctl network tuning (`net.core.somaxconn`, `tcp_tw_reuse`, buffer ceilings).
  - Configure cgroups v2 controller hierarchy for user slices.
- [ ] Implement `scripts/verify_host.sh` to check thermal throttling, CPU governor, and memory limits before initiating benchmark runs.

### 5.2 GitHub Actions PR Smoke Pipeline (`.github/workflows/ci.yml`)
- [ ] Format check (`cargo fmt --check`).
- [ ] Lint check (`cargo clippy --all-targets -- -D warnings`).
- [ ] Convention check (`sh scripts/check_conventions.sh`).
- [ ] Test execution (`cargo test`).
- [ ] Framework matrix smoke test: build PR-modified framework in container, verify `/healthz` and `/plaintext`.

### 5.3 Automated Benchmark Runner Lifecycle
- [ ] Configure GitHub repository dispatch trigger or cron job for scheduled benchmark runs on the bare-metal machine.
- [ ] Implement runner script:
  - Pull latest `main` branch.
  - Run full benchmark orchestrator across all registered frameworks.
  - Write output JSON artifacts and render leaderboard into `README.md`.
  - Commit results with message: `chore(benchmark): update results [skip ci]`.
  - Push branch directly back to GitHub `main`.
