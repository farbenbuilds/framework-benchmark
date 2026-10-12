# Data-Oriented Design Guidelines

## Purpose
In high-throughput benchmarking, data-oriented design (DoD) guarantees that the measurement harness does not introduce latency spikes, memory fragmentation, or cache invalidation into the target framework's run.

## 1. Data is Memory Layout
- Hardware processes cache lines (64 bytes on x86_64).
- Structs must be organized to avoid false sharing and excessive padding.
- Arrange struct fields in descending order of size or explicitly mark them with `#[repr(C, align(64))]` when shared across worker threads.

## 2. Structure of Arrays (SoA)
Avoid storing arrays of fat structs during continuous metric sampling:

```rust
// Disallowed: Array of Structures (AoS) - poor cache line efficiency
struct MetricSample {
    timestamp_ns: u64,
    cpu_user_pct: f32,
    cpu_sys_pct: f32,
    memory_rss_bytes: u64,
    active_connections: u32,
}
struct CollectorAos {
    samples: Vec<MetricSample>,
}

// Required: Structure of Arrays (SoA) - vectorized cache line streaming
struct MetricSeriesSoa {
    timestamps: Vec<u64>,
    cpu_user_pct: Vec<f32>,
    cpu_sys_pct: Vec<f32>,
    memory_rss_bytes: Vec<u64>,
    active_connections: Vec<u32>,
}
```

Benefits:
- Prefetchers stream single-field arrays with optimal throughput.
- Vectorized SIMD operations can calculate min, max, average, and percentiles in bulk.

## 3. Hot and Cold Data Splitting
- **Hot Data**: Variables modified or read on every request or sample tick (e.g., latency counter, response status code, byte counter).
- **Cold Data**: Metadata read once during setup or reporting (e.g., framework name, author, git commit hash, compiler version).
- Keep hot data in compact, contiguous arrays. Keep cold metadata in separate heap allocations outside the measurement loop.

## 4. Zero-Allocation Hot Paths
- Allocate all network buffers, request templates, and HDR histogram bins prior to initiating the measurement phase.
- Use reusable ring buffers or contiguous slices.
- Do not call `Vec::push` with reallocation, `String::from`, or `Box::new` while the load generator or resource collector is active.

## 5. Functional Processing of DoD Data
- Keep data structures as plain data with public fields.
- Write pure transformation functions that take slices of arrays and compute summaries:
  ```rust
  pub fn compute_percentile(sorted_samples: &[u64], percentile: f64) -> u64 {
      // Pure calculation over contiguous slice
      let idx = ((sorted_samples.len() as f64) * percentile / 100.0) as usize;
      sorted_samples[idx.min(sorted_samples.len() - 1)]
  }
  ```
