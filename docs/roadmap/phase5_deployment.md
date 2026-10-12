# Roadmap Phase 5: Bare-Metal Deployment and GitHub CI/CD

## Overview
Phase 5 automates host tuning, bare-metal runner scheduling, and GitHub pull request / push lifecycle automation.

## Tasks and Checklist

### 5.1 Host Provisioning and OS Tuning (`scripts/`)
- [x] Implement `scripts/setup_host.sh` for Ubuntu/Debian bare-metal machines:
  - Configure CPU scaling governor to `performance`.
  - Apply sysctl network tuning (`net.core.somaxconn`, `tcp_tw_reuse`, buffer ceilings).
- [x] Implement `scripts/verify_host.sh` to check CPU governor, cgroups v2, and socket backlog before initiating benchmark runs.

### 5.2 GitHub Actions PR Smoke Pipeline (`.github/workflows/lint.yml`)
- [x] Format check (`cargo fmt --check`).
- [x] Lint check (`cargo clippy --all-targets -- -D warnings`).
- [x] Convention check (`scripts/check_conventions.sh`).
- [x] Test execution (`cargo test --workspace`).

### 5.3 Automated Benchmark Runner Lifecycle
- [ ] Configure GitHub repository dispatch trigger or cron job for scheduled benchmark runs on the bare-metal machine.
- [ ] Implement runner orchestrator execution script updating `results/` artifacts.
