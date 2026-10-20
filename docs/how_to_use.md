# Benchmarking on Linux and OpenStack

This guide runs the Docker benchmark matrix on a dedicated Linux host. A benchmark is publishable only when the selected glibc and musl runs both finish and the host configuration is recorded with the artifacts.

## 1. Provision an OpenStack host

Use two instances when possible: one server host and one load-generator host on a private provider network. Do not point the load generator at public or unconsenting addresses.

```sh
openstack flavor create benchmark-16c --ram 32768 --disk 80 --vcpus 16
openstack flavor set benchmark-16c --property hw:cpu_policy=dedicated
openstack flavor set benchmark-16c --property hw:numa_nodes=1
openstack security group create benchmark-ssh
openstack security group rule create --proto tcp --dst-port 22 benchmark-ssh
openstack server create --flavor benchmark-16c --image ubuntu-24.04 \
  --network benchmark-net --security-group benchmark-ssh --key-name benchmark-key bench-01
```

The image, network, and key names are cloud-specific. CPU pinning and NUMA placement require support from the OpenStack operator; confirm the resulting topology with `lscpu` after login. Do not expose port 8080 publicly unless the traffic source is explicitly restricted.

## 2. Install the host prerequisites

Connect to the instance, clone a pinned revision, and run the explicit mutating setup command. It installs Docker, Compose, build tools, network tools, and applies reversible sysctl tuning. Rust is deliberately installed as the benchmark user so its toolchain stays user-owned.

```sh
git clone git@github.com:farbenbuilds/framework-benchmark.git
cd framework-benchmark
git checkout <reviewed-commit>
sudo scripts/setup_linux.sh --apply
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
. "$HOME/.cargo/env"
sudo usermod -aG docker "$USER"
```

Log out and back in after the group change. Verify the exact toolchain required by the workspace and the host before staging a run:

```sh
rustc --version
cargo --version
docker version
scripts/validate_environment.sh
sh scripts/check_conventions.sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

`validate_environment.sh` fails when Docker, Compose, cgroups v2, Git, Cargo, or minimum available memory is missing. It does not prove CPU isolation; reserve that capacity in the cloud flavor and avoid colocating unrelated workloads.

## 3. Stage the database and a smoke target

The full runner starts the repository PostgreSQL compose service only when the selected matrix has database workloads. It waits for the service health check and leaves an already-running database untouched. Start it explicitly when inspecting it, then build and run one framework/libc image before committing to the complete matrix.

```sh
docker compose -f deploy/postgres/compose.yaml up -d
docker compose -f deploy/postgres/compose.yaml ps
scripts/build_container.sh frameworks/web-server/rust/actix-web benchmark/actix:glibc glibc
docker run --rm --network host --name benchmark-smoke benchmark/actix:glibc
```

In another shell, wait for `/healthz` and make a single request to `/plaintext`. Stop the foreground container with `Ctrl-C`. Select an implementation that is expected to build on the host; a registry manifest is validated metadata, not proof that every third-party image can build on every architecture.

## 4. Run benchmarks

### Small OpenStack instances

For a 2-vCPU, 8-GB VM, use the bounded wrapper. It validates the instance, pins the target and load generator to separate CPUs, and starts each target with one CPU, 2 GB memory, and a 128-process limit. Start with one libc and one framework smoke run; do not run the full matrix until the instance remains stable.

```sh
scripts/run_small_host.sh --check-only
scripts/run_small_host.sh cpeak
BENCHMARK_LIBC=musl scripts/run_small_host.sh cpeak
```

The wrapper sets `BENCHMARK_SMALL_HOST=1`, `BENCHMARK_CONCURRENCY=8`, `BENCHMARK_DURATION_SECS=10`, `BENCHMARK_TARGET_WORKERS=1`, and `BENCHMARK_DATABASE_MANAGED=0` unless the caller overrides them. The small profile uses a 5-second warmup and 3-second cooldown. Framework image builds can still need significant memory; keep swap enabled, stop unrelated services, and run frameworks sequentially. See [small_host.md](small_host.md) for the per-language build notes, database workflow, and troubleshooting, and [small_host_frameworks.md](small_host_frameworks.md) for the per-framework smoke matrix. Use a larger dedicated host for comparative full-matrix publication.

Each plaintext-capable adapter also runs bounded HTTP-Arena supplemental profiles: `http1_pipelining_16`, `connection_churn`, and `paced_plaintext`. The paced profile targets 100 requests per second per configured worker; it is a latency-under-load profile, not a peak-throughput ranking.

The compatibility entry point delegates to the all-framework runner. It validates the host, builds the release orchestrator, then lets the orchestrator build, start, measure, and remove each selected libc image. Start the repository database yourself before selecting frameworks with database capabilities.

```sh
BENCHMARK_DEDICATED_HOST=1 \
BENCHMARK_TARGET_CPUS=2-7 \
BENCHMARK_LOAD_CPUS=8-15 \
BENCHMARK_CGROUP_PARENT=framework-benchmark \
scripts/verify_host.sh --require-dedicated
BENCHMARK_DEDICATED_HOST=1 \
BENCHMARK_TARGET_CPUS=2-7 \
BENCHMARK_LOAD_CPUS=8-15 \
BENCHMARK_CGROUP_PARENT=framework-benchmark \
scripts/run_benchmarks.sh
```

Use this smaller matrix while developing an adapter:

```sh
BENCHMARK_LIBC=glibc scripts/run_benchmarks_all.sh
DATABASE_URL=postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world \
  scripts/run_benchmarks_all.sh
```

Set `BENCHMARK_FRAMEWORK_FILTER` to an exact manifest name to run one adapter, for example `BENCHMARK_FRAMEWORK_FILTER=actix-web scripts/run_benchmarks_all.sh`.

Set `CONTAINER_RUNTIME`, `BENCHMARK_LIBC`, `BENCHMARK_CONCURRENCY`, `BENCHMARK_DURATION_SECS`, `BENCHMARK_TARGET_WORKERS`, `BENCHMARK_TARGET_HOST`, `BENCHMARK_MANAGE_CONTAINER`, `DATABASE_URL`, and `BENCHMARK_DATABASE_MANAGED` before invoking the script when the defaults do not fit the host. `BENCHMARK_IMAGE_PULL=1` and `BENCHMARK_NO_CACHE=1` make image refresh behavior explicit.

For a two-instance run, point `BENCHMARK_TARGET_HOST` at the server instance and set `BENCHMARK_MANAGE_CONTAINER=0`; the load generator then uses the server's existing deployment. Remote targets do not expose `/proc` or cgroups to the load host, so resource columns are empty unless the collector runs on the server. When the target runs outside a managed Docker container on the same host, set `BENCHMARK_TARGET_PID` and `BENCHMARK_TARGET_CGROUP` so resource sampling attaches to the right process. Set `BENCHMARK_CONTAINER_LOGS=1` to retain Docker logs; the default `none` log driver avoids disk growth during load. Managed containers drop all Linux capabilities, enable `no-new-privileges`, and use an init process; set `BENCHMARK_DISABLE_HARDENING=1` only for a framework that provably needs a capability. Run the Autobahn conformance suite only against a running WebSocket target:

```sh
AUTOBAHN_TARGET_URL=ws://127.0.0.1:8080/ws scripts/run_autobahn_suite.sh
```

## 5. Inspect, compare, and retain results

Immutable JSON artifacts are stored under `results/data/YYYY/MM/`; the index is `results/index.json`, and generated tables/charts are in `results/`. Each artifact records the Git revision, immutable framework-input SHA-256, inspected local image ID, CPU placement, cgroup path, and per-workload traffic settings. The orchestrator replaces the README block once per selected matrix so its tables only compare results from that invocation. Review the intended result diff before staging it:

```sh
find results -type f | sort
git status --short results README.md
git diff -- results README.md
```

Never compare separate runs as if they were identical when image digests, host load, CPU placement, clock source, or load-generator placement changed. Keep the raw JSON, host details, revision, and both libc variants together.

## 6. Commit results and clean up

The result-commit helper is dry-run by default. `--apply` stages only `results/` and `README.md`; `--push` requires `--apply` and uses the configured Git remote.

```sh
scripts/commit_benchmark_results.sh
scripts/commit_benchmark_results.sh --apply
scripts/commit_benchmark_results.sh --apply --push
scripts/cleanup_environment.sh --apply
```

Cleanup removes managed containers and stops the database. With `BENCHMARK_PURGE_DATABASE=1`, it also removes database volumes; this is destructive. It restores sysctl values saved by `setup_linux.sh` when run as root. Set `BENCHMARK_PRUNE=1` only when removing unused images is intended.
