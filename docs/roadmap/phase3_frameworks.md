# Roadmap Phase 3: Framework Registry and Baseline Implementations

## Overview
Phase 3 populates baseline framework implementations across major programming languages to establish initial performance benchmarks and calibrate the engine.

## Tasks and Checklist

### 3.1 Framework Selection and Registry Setup
- [x] Establish framework directory structure specification in `frameworks/README.md`.
- [ ] Implement user-selected benchmark targets across requested languages and runtimes.

### 3.2 Dual-Libc Container Verification
- [ ] Verify each selected framework passes `/healthz`, `/plaintext`, and `/json`.
- [ ] Verify both `Dockerfile.glibc` and `Dockerfile.musl` build successfully.
