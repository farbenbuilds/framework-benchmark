# Framework Registry and Benchmark Targets

This directory contains standardized benchmark implementations across programming languages and runtime engines.

## Directory Structure Conventions

```text
frameworks/
└── <role>/                  # web-server or web-client
    └── <language>/          # rust, go, csharp, typescript, python, cplusplus, zig
        └── <framework-name>/
            ├── framework.toml
            ├── Dockerfile.glibc
            ├── Dockerfile.musl
            └── src/
```

## Implementation Requirements

1. **Manifest**: Each implementation must provide a validated `framework.toml` adhering to the common schema.
2. **Standard Endpoints**:
   - `GET /healthz`: Immediate 200 OK for orchestrator readiness polling.
   - `GET /plaintext`: Returns `Hello, World!` with `Content-Type: text/plain; charset=utf-8`.
   - `GET /json`: Returns `{"message": "Hello, World!"}` with `Content-Type: application/json`.
3. **Dual-Libc Support**: Provide `Dockerfile.glibc` (Debian) and `Dockerfile.musl` (Alpine) builds.
4. **Port Allocation**: Server listens on port `8080`.
