# Framework Benchmark Architecture

## System Overview
`framework-benchmark` is an end-to-end benchmark harness designed to measure HTTP and WebSocket server and client frameworks with minimal measurement jitter and zero coordinated omission.

## Process and Isolation Model
```text
+-----------------------------------------------------------------+
| Dedicated Linux Benchmark Host                                  |
|                                                                 |
| [ Cores 0-1 ]   Kernel interrupts, OS daemons, and collector    |
| [ Cores 2-7 ]   Framework Under Test (cgroups v2 + taskset)     |
| [ Cores 8-15]   Asynchronous Load Generator (Tokio / Hyper)     |
|                                                                 |
| Communication: Loopback (lo) or 10/40 GbE dedicated interface   |
+-----------------------------------------------------------------+
```

### CPU isolation
- Reserve server and load-generator capacity in the host or OpenStack flavor before a run.
- The orchestrator requires dedicated runs to declare server and load CPU sets, pins the load-generator process, and passes the server set to Docker.
- Record topology and traffic placement with every publishable result.

### Cgroups v2 Resource Accounting
Resource consumption is monitored at the kernel level:
- `memory.current`: Total instantaneous memory usage.
- `memory.peak`: Peak memory footprint (RSS + page cache).
- `cpu.stat`: Microsecond-accurate user and system CPU times.
- `/proc/[pid]/statm`: Break down shared, data, and resident memory pages.

## Test Execution State Machine
Every benchmark trial follows a deterministic state machine:

```text
[Init] -> [Build Container] -> [Spawn Process] -> [Poll Healthz]
                                                        |
[Teardown] <- [Cooldown (10s)] <- [Sample (30s)] <- [Warmup (15s)]
     |
[Process Metrics] -> [Emit JSON]
```

1. **Build**: Build `Dockerfile.glibc` or `Dockerfile.musl`.
2. **Spawn**: Start container with cgroups v2 limits and CPU affinity.
3. **Poll Healthz**: Exponential backoff polling `/healthz` on port 8080 until HTTP 200 is received or 30-second timeout expires.
4. **Warmup**: Drive traffic for 15 seconds to warm up JIT runtimes, TCP connection pools, and page tables. Data is discarded.
5. **Sample**: Drive synthetic load for 30 seconds while the collector records latency percentiles and system metrics.
6. **Cooldown**: Terminate load and wait 10 seconds for TIME_WAIT sockets to drain.
7. **Teardown**: Send SIGTERM to the container, wait for graceful exit, or issue SIGKILL after 5 seconds.
8. **Process Metrics**: Read collected HDR histogram and cgroups stats, serialize to JSON, and clean up.

## Load Generation and Latency Tracking
- Built on Tokio TCP streams and httparse response framing.
- Latency samples are recorded in microsecond-resolution **HDR Histograms**.
- **Coordinated Omission**: Uses a scheduled issuance model so request delays under queue saturation are recorded in the histogram rather than silently dropped.
