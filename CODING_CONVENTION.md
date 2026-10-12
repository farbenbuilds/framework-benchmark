# Framework Benchmark Coding Conventions

These conventions adapt the Linux Kernel Coding Style to Rust (edition 2024 / 1.85+), emphasizing pragmatism, performance, readability, and deterministic behavior.

## 1. Core General Rules
- **No Emojis**: Emojis are strictly forbidden anywhere in the repository, including code, comments, documentation, commit messages, and PRs.
- **Strict File Limit (200 Lines)**: No file of any type (Rust source, Markdown, TOML, YAML, shell scripts) may exceed 200 lines. Split larger modules into coherent submodules.
- **Minimal Comments**: Avoid explaining what the code does; clear naming and structure must make it obvious. Explain only the non-obvious "why" in 1 line (maximum 2 lines).
- **Toolchain Authority**: All Rust source code must pass `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.

## 2. Linux Style Adaptations for Rust
- **Control Flow Spacing**: Put a space after keywords (`if`, `while`, `for`, `match`). Do not cuddle parentheses around conditions.
- **Early Returns and Guard Clauses**: Always return early to avoid deep indentation nesting. Keep the happy path unindented.
  ```rust
  // Disallowed: Deep nesting
  if condition {
      if valid {
          execute_task();
      }
  }

  // Required: Guard clauses
  if !condition {
      return;
  }
  if !valid {
      return;
  }
  execute_task();
  ```
- **Nesting Limit**: Control flow must never exceed 3 levels of indentation inside any function. Extract focused helper functions when logic grows deeper.
- **Naming Conventions**:
  - Files and directories: strictly `snake_case` (e.g., `load_engine.rs`, `cgroup_sampler.rs`).
  - Functions, variables, and modules: strictly `snake_case`.
  - Types (structs, enums, traits): `PascalCase`.
  - Constants and statics: `SCREAMING_SNAKE_CASE`.

## 3. Separation of Concerns
Every component must belong to one distinct architectural layer:
1. **Data Layer**: Flat structs, memory layouts, and Plain Old Data. Contains no business logic, network calls, or side effects.
2. **Logic Layer**: Pure transformation functions (e.g., latency percentiles, HDR histogram calculations, JSON serialization models). Operates deterministically on inputs and returns outputs with zero I/O.
3. **Driver Layer**: Hardware and OS interfaces (e.g., `/proc` reading, cgroups v2 accounting, socket polling, Docker API calls). Handles raw I/O and converts external states into typed data structs.
4. **Composition Layer**: Assembles data, logic, and drivers into the top-level benchmark execution pipeline.

## 4. Pragmatic Data-Oriented Design (DoD)
- **Memory Layout**: Align structs to CPU cache line boundaries. Group fields by read/write frequency.
- **Structure of Arrays (SoA)**: Use SoA for bulk measurements and telemetry streams to enable hardware prefetching and vectorization.
- **Avoid Pointer Chasing**: Replace reference graphs, boxing (`Box<dyn Trait>`), and indirection on hot paths with contiguous vectors and index lookups.
- **Hot/Cold Splitting**: Isolate frequently accessed metric counters from cold metadata like framework names and descriptions.
- **Zero-Allocation Hot Paths**: Pre-allocate buffers, request pools, and histogram bins before beginning load measurement windows.

## 5. Pragmatic Functional Programming (FP)
- **Zero OOP**: Object-oriented patterns (classes, virtual dispatch tables, hidden mutable state) are forbidden.
- **Pure Functions**: Write standalone functions that map inputs to outputs without mutating external state or performing side effects.
- **Explicit Context**: Pass all required configuration, timestamps, and memory buffers as explicit parameters.
- **Immutability by Default**: Mark bindings `mut` only when an in-place buffer mutation is strictly required for zero-allocation performance.

## 6. Error Handling Discipline
- Return idiomatic `Result<T, E>` with explicit custom error enums.
- The `?` operator must be used for concise error propagation with guard returns.
- `unwrap()` and `expect()` are forbidden in non-test production code.
- Never swallow errors silently; handle every branch explicitly.

## 7. Anti-Slop Enforcement
- No dead code: remove unused functions, imports, constants, and dependencies immediately.
- No slop markers: never commit `TODO`, `FIXME`, `XXX`, or `HACK` comments.
- Verification: run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `sh scripts/check_conventions.sh` before opening any PR.
