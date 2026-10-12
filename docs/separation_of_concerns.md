# Separation of Concerns Architecture

## Overview
Every file in `framework-benchmark` must have exactly one well-defined responsibility. Mixing I/O, mathematical computation, data definitions, and presentation in a single file is strictly prohibited.

## The Iron Rules
1. **Never mix data collection (I/O) with data transformation (logic)**.
2. **Never mix calculation logic with output formatting (presentation)**.
3. **Hard Limit**: No file of any type may exceed 200 lines of code.

## The Four Architectural Layers

### 1. Data Layer (`crates/common/src/models/`, `crates/*/src/data/`)
- Contains only plain structs, enums, and data definitions.
- Derives standard traits (`Clone`, `Copy`, `Debug`, `Serialize`, `Deserialize`).
- Contains zero business logic, network requests, or OS system calls.

### 2. Logic Layer (`crates/*/src/logic/`)
- Contains pure functions that transform inputs into outputs.
- Examples:
  - Latency histogram binning and percentile math.
  - Rate calculations (requests per second, transfer megabytes per second).
  - Validation of `framework.toml` configuration manifests.
- Rules:
  - Completely deterministic.
  - Zero I/O, zero network calls, zero reading from the filesystem or clock.
  - Trivially unit-testable without mocks or stubs.

### 3. Driver Layer (`crates/*/src/drivers/`)
- Handles direct interaction with the outside world:
  - Reading `/proc/[pid]/statm` and Linux cgroups v2 filesystems.
  - Docker engine API interactions via Unix domain sockets.
  - Spawning child processes with `taskset` CPU affinities.
  - Sending HTTP/WS requests over Tokio network streams.
- Rules:
  - Converts external raw bytes into strongly typed data layer structs.
  - Performs no business logic, statistical aggregation, or report formatting.

### 4. Composition Layer (`crates/orchestrator/src/runner/`, `crates/*/src/pipeline/`)
- Connects drivers, logic, and data into a cohesive workflow.
- Orchestrates step sequences: setup -> warmup -> measure -> teardown.
- Passes outputs from driver layers directly into pure logic functions.

## Refactoring Trigger
If a file does more than one of the following:
- Reads system state or network sockets
- Performs statistical transformations or calculations
- Renders Markdown, JSON, or terminal output
Then it violates separation of concerns and must be split immediately.
