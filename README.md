# Framework Benchmark

> An automated, high-performance client and server framework benchmark platform hosted on bare-metal and OpenStack Linux infrastructure.

Inspired by [TechEmpower FrameworkBenchmarks](https://github.com/TechEmpower/FrameworkBenchmarks) and [HttpArena](https://github.com/MDA2AV/HttpArena), this project evaluates HTTP and WebSocket server and client frameworks across various programming languages, runtimes, and C standard libraries (**glibc** vs **musl**).

## Key Features

- **TechEmpower and HTTP-Arena Workloads**: `plaintext`, `json`, `single_query`, `multiple_queries`, `fortunes`, `data_updates`, HTTP/1.1 pipelining, connection churn, paced traffic, and WebSocket echo.
- **Dual-Libc Support**: Evaluates frameworks built against both **GNU C Library (glibc)** and **musl libc** (Alpine/Distroless).
- **Extensible Framework Registry**: Declarative framework definitions in `frameworks/` using **Docker** and **Nix**.
- **Native Rust Harness**: Ultra-low-overhead asynchronous load generator and Linux resource profiler (cgroups v2, `/proc`, peak RSS, CPU utilization).
- **Publishable Artifacts**: Immutable JSON results include revision, container image ID, and exact traffic settings alongside category charts and complete Markdown tables. A labelled dedicated runner publishes reviewed results to this `README.md`.
- **Bounded Small-Host Profile**: A guarded 2 vCPU / 8 GB OpenStack wrapper, per-language smoke matrix, CPU pinning, memory limits, and reduced warmup for adapter validation without pretending small-host numbers are publishable.

## Project Roadmap and Architecture

For the detailed architectural specification, component breakdown, and step-by-step developer and agent checklist, refer to:
- Roadmap Summary: [TODO.md](TODO.md)
- Architecture Overview: [docs/roadmap/overview.md](docs/roadmap/overview.md)
- Host Workflow: [docs/how_to_use.md](docs/how_to_use.md)
- Small OpenStack Hosts: [docs/small_host.md](docs/small_host.md)
- Code Knowledge Graph: [docs/graphify.md](docs/graphify.md)
- Agent Persona and Directives: [AGENTS.md](AGENTS.md)
- Coding Conventions: [CODING_CONVENTION.md](CODING_CONVENTION.md)

## Benchmark Results

<!-- BENCHMARK_RESULTS_START -->
*No verified benchmark run has been published yet. Results are only committed after both libc variants complete on a labelled dedicated host.*
<!-- BENCHMARK_RESULTS_END -->

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
