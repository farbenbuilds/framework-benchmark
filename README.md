# Framework Benchmark

> An automated, high-performance client and server framework benchmark platform hosted on bare-metal and OpenStack Linux infrastructure.

Inspired by [TechEmpower FrameworkBenchmarks](https://github.com/TechEmpower/FrameworkBenchmarks) and [HttpArena](https://github.com/MDA2AV/HttpArena), this project evaluates HTTP and WebSocket server and client frameworks across various programming languages, runtimes, and C standard libraries (**glibc** vs **musl**).

## Key Features

- **Multi-Category Workloads**: Plaintext, JSON serialization, single DB query, multiple DB queries, Fortunes (HTML escaping), data updates, and WebSockets.
- **Dual-Libc Support**: Evaluates frameworks built against both **GNU C Library (glibc)** and **musl libc** (Alpine/Distroless).
- **Extensible Framework Registry**: Declarative framework definitions in `frameworks/` using **Docker** and **Nix**.
- **Native Rust Harness**: Ultra-low-overhead asynchronous load generator and Linux resource profiler (cgroups v2, `/proc`, peak RSS, CPU utilization).
- **Automated Results Pipeline**: The benchmark runner machine executes scheduled benchmark runs, automatically updates results in this `README.md` and `results/`, and pushes updates directly back to GitHub.

## Project Roadmap and Architecture

For the detailed architectural specification, component breakdown, and step-by-step developer and agent checklist, refer to:
- Roadmap Summary: [TODO.md](TODO.md)
- Architecture Overview: [docs/roadmap/overview.md](docs/roadmap/overview.md)
- Agent Persona and Directives: [AGENTS.md](AGENTS.md)
- Coding Conventions: [CODING_CONVENTION.md](CODING_CONVENTION.md)

## Benchmark Results

<!-- BENCHMARK_RESULTS_START -->
*Benchmark runs will be automatically populated here by the benchmark runner agent.*
<!-- BENCHMARK_RESULTS_END -->

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
