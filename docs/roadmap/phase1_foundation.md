# Roadmap Phase 1: Project Foundation and Specification Standards

## Overview
Phase 1 establishes the Rust workspace, file layout, manifest schema, and data models for all subsequent benchmark operations.

## Tasks and Checklist

### 1.1 Workspace Setup and Directory Skeleton
- [ ] Initialize standard root directory structure: `crates/`, `frameworks/`, `scripts/`, `results/`, `docs/`.
- [ ] Create root `Cargo.toml` defining the workspace members:
  - `crates/orchestrator`
  - `crates/load-gen`
  - `crates/collector`
  - `crates/reporter`
  - `crates/common`
- [ ] Configure workspace-level lints and release profile optimizations:
  - `opt-level = 3`
  - `lto = "fat"`
  - `codegen-units = 1`
  - `panic = "abort"`
- [ ] Enforce the 200-line per file ceiling on all newly created crates.

### 1.2 Framework Manifest Specification (`crates/common`)
- [ ] Implement `framework.toml` parser using `serde` and `toml`.
- [ ] Define data models:
  - `FrameworkManifest`: metadata, version, category, language.
  - `Capabilities`: boolean flags for supported routes (`plaintext`, `json`, `db_single`, `db_multi`, `fortunes`, `updates`, `websocket`).
  - `BuildEngine`: Docker and Nix build configurations.
  - `RuntimeConfig`: port, healthcheck path, startup timeout, shutdown signal.
- [ ] Write unit tests verifying parsing of compliant and invalid manifests.

### 1.3 Result Schema Specification (`crates/common`)
- [ ] Define JSON result models:
  - `RunResult`: run ID, machine metadata, framework metadata, timestamp.
  - `TestMetrics`: category name, duration, concurrency, requests total.
  - `LatencyDistribution`: P50, P75, P90, P99, P99.9, P99.99, min, max, std_dev.
  - `ResourceUsage`: peak RSS, average memory, CPU user %, CPU kernel %.
- [ ] Export JSON Schema definition to `docs/schemas/result_schema.json`.
