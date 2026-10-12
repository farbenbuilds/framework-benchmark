# Framework Benchmark

> An automated, high-performance client & server framework benchmark platform hosted on bare-metal / OpenStack Linux infrastructure.

Inspired by [TechEmpower FrameworkBenchmarks](https://github.com/TechEmpower/FrameworkBenchmarks) and [HttpArena](https://github.com/MDA2AV/HttpArena), this project evaluates HTTP and WebSocket server and client frameworks across various programming languages, runtimes, and C standard libraries (**glibc** vs **musl**).

---

## Key Features

- **Multi-Category Workloads**: Plaintext, JSON serialization, Single DB query, Multiple DB queries, Fortunes (HTML escaping), Data updates, and WebSockets.
- **Dual-Libc Support**: Evaluates frameworks built against both **GNU C Library (glibc)** and **musl libc** (Alpine/Distroless).
- **Extensible Framework Registry**: Declarative framework definitions in `frameworks/` using **Docker** and **Nix**.
- **Native Rust Harness**: Ultra-low-overhead asynchronous load generator and Linux resource profiler (cgroups v2, `/proc`, peak RSS, CPU utilization).
- **Automated Results Pipeline**: The benchmark runner machine executes scheduled benchmark runs, automatically updates the results in this `README.md` and `results/`, and pushes updates directly back to GitHub.

---

## Project Roadmap

For the detailed architectural specification, component breakdown, and step-by-step developer/agent checklist, refer to:
👉 **[TODO.md](TODO.md)**

---

## Benchmark Results

<!-- BENCHMARK_RESULTS_START -->
*Benchmark runs will be automatically populated here by the benchmark runner agent.*
<!-- BENCHMARK_RESULTS_END -->

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
