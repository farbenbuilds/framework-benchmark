# Framework Benchmark Codebase

## Scope
`framework-benchmark` is an industrial-grade client and server benchmark suite written in Rust. It evaluates HTTP and WebSocket server and client frameworks across programming languages and C standard libraries (**glibc** vs **musl**).

The engine provides deterministic execution on dedicated Linux systems, utilizing CPU core pinning, cgroups v2 resource quotas, HDR histogram latency tracking with coordinated omission compensation, and automated result publishing to GitHub.

## Core Architectural Rules
1. **Strict File Limit (200 Lines)**: Every file across Rust source, Markdown, TOML, YAML, and scripts must stay at or below 200 lines.
2. **Data-Oriented Design (DoD)**: Telemetry and latency histograms use contiguous Structure of Arrays (SoA) layout. Pre-allocate measurement slabs prior to test runs.
3. **Pragmatic Functional Programming (FP)**: Pure functions handle latency percentiles, histogram aggregation, and manifest validation without hidden state or mutating side effects.
4. **Separation of Concerns**: Data definitions, pure algorithms, Linux/cgroups drivers, and orchestration stay in strictly separate modules.
5. **Linux Coding Style**: Enforce `snake_case` naming, guard clauses, shallow indentation (<= 3 levels), and space-after-keyword formatting.
6. **Zero Emojis**: Emojis are strictly banned from all source code, documentation, comments, commit messages, and reports.
7. **Minimal Comments**: Keep code self-explanatory. Document only the non-obvious "why" in 1 line (maximum 2 lines).

## Workspace Layout
```text
framework-benchmark/
├── Cargo.toml                 # Root workspace manifest
├── AGENTS.md                  # Unified sub-agent directives and rules
├── SKILL.md                   # Master engineering workflow integrating skills
├── CODING_CONVENTION.md       # Linux-style Rust coding conventions
├── CODEBASE.md                # System architecture and codebase overview
├── CONTRIBUTE.md              # Framework and code contribution contract
├── CI_CD_PIPELINE.md          # Continuous integration and benchmark pipeline
├── crates/
│   ├── orchestrator/          # Test execution harness, process and container lifecycle
│   ├── load-gen/              # Asynchronous HTTP/WS load generator and HDR histograms
│   ├── collector/             # Linux cgroups v2, /proc, and socket resource monitor
│   ├── reporter/              # JSON artifact generator, Markdown table builder, and SVG charts
│   └── common/                # Manifest schema, metric types, and configuration models
├── frameworks/                # Contributed framework definitions and source code
│   └── <category>/<lang>/<fw> # Isolated directory containing framework.toml and Docker/Nix files
├── docs/                      # Architectural specifications and subsystem guides
│   ├── architecture.md        # Detailed engine architecture and lifecycle state machine
│   ├── data_oriented_design.md# Data-oriented memory layout and SoA guidelines
│   ├── separation_of_concerns.md# Architectural layer boundaries and decomposition
│   ├── small_host.md          # Bounded 2 vCPU / 8 GB OpenStack test workflow
│   ├── graphify.md            # Optional code knowledge graph for agent reviews
│   ├── sub_agents.md          # Antigravity, OpenCode, and Codex sub-agent workflows
│   └── roadmap/               # Modularized implementation phases
├── scripts/                   # Quality gate and verification scripts
│   ├── check_conventions.sh   # POSIX convention, emoji, and line-limit lint script
│   └── check_conventions.ps1  # PowerShell convention lint script
├── .codex/                    # Codex agent team definitions and configuration
└── .opencode/                 # OpenCode sub-agent definitions and configuration
```

## Runtime Data Flow
```text
CLI Runner / Scheduled Event
          |
          v
Orchestrator Engine
  +-- Read framework manifest (framework.toml)
  +-- Launch isolated container/process (Docker or Nix, glibc or musl)
  +-- Pin CPU affinities (sched_setaffinity) & configure cgroups v2
  +-- Poll readiness probe (/healthz)
          |
          v
Warmup Phase (JIT stabilization and TCP warmup)
          |
          v
Measurement Window
  +-- Load Generator (Tokio / Hyper / HDR Histogram) <---> Target Framework
  +-- Resource Collector (cgroups v2, /proc/statm, socket counters)
          |
          v
Cooldown & Graceful Teardown (Socket drain & container removal)
          |
          v
Result Processing Engine (crates/reporter)
  +-- Write JSON snapshot to results/data/
  +-- Generate Markdown leaderboards in README.md
  +-- Commit and push results to GitHub
```

## Subsystem Details
For detailed subsystem specifications, consult:
- Engine Lifecycle: [docs/architecture.md](docs/architecture.md)
- Memory and Layout: [docs/data_oriented_design.md](docs/data_oriented_design.md)
- Layered Boundaries: [docs/separation_of_concerns.md](docs/separation_of_concerns.md)
- Agent Collaboration: [docs/sub_agents.md](docs/sub_agents.md)
- Phased Roadmap: [TODO.md](TODO.md) and [docs/roadmap/overview.md](docs/roadmap/overview.md)
