# Roadmap Phase 3: Framework Registry and Baseline Implementations

## Overview
Phase 3 populates baseline framework implementations across major programming languages to establish initial performance benchmarks and calibrate the engine.

## Tasks and Checklist

### 3.1 Rust Framework Implementations
- [ ] Implement `frameworks/web-server/rust/axum/` (glibc + musl).
- [ ] Implement `frameworks/web-server/rust/actix-web/` (glibc + musl).
- [ ] Implement `frameworks/web-server/rust/hyper-raw/` (glibc + musl baseline).

### 3.2 Go Framework Implementations
- [ ] Implement `frameworks/web-server/go/fasthttp/` (glibc + musl / CGO-free).
- [ ] Implement `frameworks/web-server/go/gin/` (glibc + musl).
- [ ] Implement `frameworks/web-server/go/nethttp/` (standard library baseline).

### 3.3 C# / .NET Implementations
- [ ] Implement `frameworks/web-server/csharp/aspnetcore-minimal/` (glibc + musl Alpine chiseled).

### 3.4 JavaScript / TypeScript Implementations
- [ ] Implement `frameworks/web-server/typescript/fastify/` (Node.js runtime).
- [ ] Implement `frameworks/web-server/javascript/uWebSockets.js/` (native C++ bindings).
- [ ] Implement `frameworks/web-server/typescript/bun-http/` (Bun native server).

### 3.5 Python Implementations
- [ ] Implement `frameworks/web-server/python/robyn/` (Rust-based runtime).
- [ ] Implement `frameworks/web-server/python/fastapi/` (Uvicorn + uvloop).
- [ ] Implement `frameworks/web-server/python/granian/` (Rust-based HTTP runner).

### 3.6 C and C++ Implementations
- [ ] Implement `frameworks/web-server/cplusplus/drogon/` (glibc + musl).
- [ ] Implement `frameworks/web-server/zig/zap/` (glibc + musl).

### 3.7 Conformance and Verification
- [ ] Verify each framework passes `/healthz` and `/plaintext` in local smoke containers.
- [ ] Verify both `Dockerfile.glibc` and `Dockerfile.musl` build successfully.
