# Roadmap Phase 3: Framework Registry and Baseline Implementations

## Overview
Phase 3 populates baseline framework implementations across major programming languages to establish initial performance benchmarks and calibrate the engine.

## Tasks and Checklist

### 3.1 Framework Selection and Registry Setup
- [x] Establish framework directory structure specification in `frameworks/README.md`.
- [x] Implement user-selected benchmark targets across requested languages and runtimes:
  - Zig: `uwebzockets`, `swerver`, `dusty`
  - Rust: `cpeak`, `ntex`, `actix-web`
  - Go: `fib-tuned`, `fiber`, `fasthttp`
  - C# (.NET): `genhttp`, `cadente`, `aspnet-minimal`
  - C++: `uwebsockets`, `hical`, `agensio`, `drogon`
  - JavaScript / TypeScript: `fulmine`, `bun`, `elysia`
  - Ruby: `iodine`, `agoo`, `falcon`
  - Java / Kotlin: `rapidoid`, `vertx`, `undertow`
  - PHP: `swoole`, `workerman`, `frankenphp`
  - Python: `granian`, `robyn`, `blacksheep`

### 3.2 Dual-Libc Container Verification
- [x] Verify each selected framework provides valid `framework.toml` manifest metadata and capabilities.
- [x] Verify both `Dockerfile.glibc` and `Dockerfile.musl` build paths are configured for container compilation.
