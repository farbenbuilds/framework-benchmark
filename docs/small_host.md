# Testing Frameworks on a 2 vCPU, 8 GB OpenStack Instance

This guide is for adapter development and smoke validation on a small OpenStack
instance. See [small_host_frameworks.md](small_host_frameworks.md) for the
per-framework smoke matrix. A run from this profile is not a publishable comparative benchmark:
two vCPUs cannot isolate the load generator from the framework, and the host
usually has noisy neighbours. Publish only results from a labelled dedicated
host with explicit CPU placement.

## 1. Provision the instance

Use an image with cgroups v2, Docker support, and a current Linux kernel. A
2 vCPU / 8 GB / 40 GB volume is the minimum for one framework and one libc at a
time. Two vCPUs are required because the target and the load generator must not
share a CPU.

```sh
openstack flavor create benchmark-2c8g --ram 8192 --disk 40 --vcpus 2
openstack security group create benchmark-small
openstack security group rule create --proto tcp --dst-port 22 benchmark-small
openstack server create --flavor benchmark-2c8g --image ubuntu-24.04 \
  --network benchmark-net --security-group benchmark-small \
  --key-name benchmark-key bench-small-01
```

Do not expose port 8080 to the public network. Use SSH port forwarding when a
developer workstation needs to reach a running target.

## 2. Prepare the host

```sh
sudo apt-get update
sudo apt-get install -y git curl
git clone git@github.com:farbenbuilds/framework-benchmark.git
cd framework-benchmark
sudo scripts/setup_linux.sh --apply
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
. "$HOME/.cargo/env"
sudo usermod -aG docker "$USER"
```

Log out and back in after the group change. Add 2 GB of swap before building
large images; the smallest instances otherwise kill the compiler or linker.

```sh
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
free -h
```

Check the baseline before building anything:

```sh
scripts/validate_environment.sh
scripts/verify_host.sh
sh scripts/check_conventions.sh
cargo test --workspace
```

## 3. Use the small-host profile

`scripts/run_small_host.sh` sets the bounded profile and refuses hosts below
2 vCPUs, 6 GiB total RAM, or without cgroups v2. It defaults to one target CPU, one
load CPU, eight load workers, one framework worker, a ten-second measurement,
and the glibc variant.

```sh
scripts/run_small_host.sh --check-only
scripts/run_small_host.sh cpeak
```

The equivalent environment is:

| Variable | Small-host value | Meaning |
| :--- | :--- | :--- |
| `BENCHMARK_SMALL_HOST` | `1` | Enables bounded timing and matrix rules |
| `BENCHMARK_TARGET_CPUS` | `0` | CPU reserved for the framework |
| `BENCHMARK_LOAD_CPUS` | `1` | CPU reserved for the load generator |
| `BENCHMARK_TARGET_WORKERS` | `1` | Framework worker processes |
| `BENCHMARK_CONCURRENCY` | `8` | Load-generator connections |
| `BENCHMARK_DURATION_SECS` | `10` | Measurement window |
| `BENCHMARK_TARGET_MEMORY` | `2g` | Container memory limit |
| `BENCHMARK_PIDS_LIMIT` | `128` | Container process limit |
| `BENCHMARK_CONTAINER_LOGS` | `0` | Keep Docker logs instead of the no-log driver |

The small profile uses a 5-second warmup and 3-second cooldown. A plaintext
framework runs the standard endpoint plus pipelining, connection churn, and
paced profiles, so expect roughly 70 to 90 seconds of measurement after the
image build. Database workloads add more time.

## 4. Test one framework at a time

Do not run the full registry on this instance. Build and run one adapter, verify
the standard endpoints manually, then switch libc or framework.

```sh
BENCHMARK_FRAMEWORK_FILTER=cpeak scripts/run_small_host.sh
BENCHMARK_FRAMEWORK_FILTER=fasthttp BENCHMARK_LIBC=musl scripts/run_small_host.sh
```

For a manual endpoint check, start the container in one shell:

```sh
scripts/build_container.sh frameworks/web-server/rust/cpeak benchmark/cpeak:glibc glibc
docker run --rm --network host --name cpeak-smoke benchmark/cpeak:glibc
```

In another shell:

```sh
curl --fail --silent http://127.0.0.1:8080/healthz
curl --fail --silent http://127.0.0.1:8080/plaintext
curl --fail --silent http://127.0.0.1:8080/json
```

The expected bodies are `Hello, World!` and `{"message": "Hello, World!"}`.
Check the content types as well. Stop the foreground container with Ctrl-C.

## 5. Language-specific notes

- Rust (`cpeak`, `ntex`, `actix-web`): these are the fastest path to a working
  smoke test. Use the cpeak adapter first. `actix-web` needs PostgreSQL because
  its manifest exposes database capabilities.
- Go (`fasthttp`, `fiber`, `fib-tuned`): static binaries build quickly; the
  final image is small. Build the glibc and musl variants separately.
- C# (`aspnet-minimal`, `cadente`, `genhttp`): `dotnet restore` is memory
  hungry. Run one project at a time and keep swap enabled.
- Java (`rapidoid`, `undertow`, `vertx`): Maven downloads dominate first-run
  time. Use one Maven module per invocation and keep the local Maven cache.
- C++ (`drogon`, `hical`, `uwebsockets`, `agensio`): compilation is the most
  likely OOM point on 8 GB. Build with `-j2`, not the default parallel width.
- JavaScript (`bun`, `elysia`, `fulmine`): Bun images are small. For npm-based
  fulmine, the install step needs access to the npm registry.
- Python (`blacksheep`, `granian`, `robyn`): pin the requirements, build one
  image at a time, and keep the wheel cache.
- Ruby (`agoo`, `falcon`, `iodine`): native extensions need `build-base`.
  Gem installation is the slow step.
- PHP (`frankenphp`, `swoole`, `workerman`): Swoole compiles from PECL and is
  the heaviest build. Composer and PECL need network access.
- Zig (`dusty`, `swerver`, `uwebzockets`): the Dockerfiles fetch Zig and the
  framework source at pinned revisions. Verify the download still resolves
  before blaming the adapter.

## 6. Database workloads

Plaintext and JSON adapters need no database. For database-capable adapters,
start PostgreSQL and disable managed database startup so the runner does not
race an already-running service:

```sh
docker compose -f deploy/postgres/compose.yaml up -d
DATABASE_URL=postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world \
  BENCHMARK_DATABASE_MANAGED=0 BENCHMARK_FRAMEWORK_FILTER=actix-web \
  scripts/run_small_host.sh
```

PostgreSQL shares the two vCPUs with the target and load generator. Treat the
result as a correctness check, not a throughput ranking.

## 7. Troubleshooting

- Build killed by the kernel: enable swap, stop other containers, prune unused
  images, and rebuild with fewer parallel jobs.
- Readiness timeout: run the container manually and check `/healthz`; look at
  `docker logs` when `BENCHMARK_CONTAINER_LOGS=1`.
- Port already in use: stop the previous container with
  `scripts/cleanup_environment.sh --apply`.
- Unstable results: pin the target to CPU 0 and the load generator to CPU 1,
  close SSH sessions that run compilers, and never run two frameworks at once.
- Full matrix refused: the orchestrator requires a dedicated host for the full
  matrix. Use `BENCHMARK_FRAMEWORK_FILTER` or the small-host profile.
- Small-host run still too slow: reduce `BENCHMARK_DURATION_SECS` to 5 and keep
  the cooldown at 3 seconds. Do not compare shortened runs with full runs.

## 8. Clean up

```sh
scripts/cleanup_environment.sh --apply
sudo swapoff /swapfile
sudo rm /swapfile
```

Remove the instance when the smoke test is finished. Keep raw JSON, the git
revision, and the host details together if the run is used for debugging.
