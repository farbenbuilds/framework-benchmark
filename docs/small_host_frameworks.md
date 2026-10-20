# Small-Host Framework Matrix

Use this matrix with [small_host.md](small_host.md). Run one row at a time on a
2 vCPU / 8 GB OpenStack instance. Every command assumes the repository root and
a prepared host.

```sh
BENCHMARK_FRAMEWORK_FILTER=<framework> scripts/run_small_host.sh
BENCHMARK_FRAMEWORK_FILTER=<framework> BENCHMARK_LIBC=musl scripts/run_small_host.sh
```

| Framework | Language | Build cost | Smoke notes |
| :-------- | :------- | :--------- | :---------- |
| `cpeak` | Rust | Low | Best first target; no database |
| `ntex` | Rust | Low | No database |
| `actix-web` | Rust | Medium | Needs PostgreSQL for `/db` and `/fortunes` |
| `fasthttp` | Go | Low | No database |
| `fiber` | Go | Low | No database |
| `fib-tuned` | Go | Low | No database |
| `aspnet-minimal` | C# | Medium | First restore downloads NuGet packages |
| `cadente` | C# | Medium | Beta engine; verify package restore |
| `genhttp` | C# | Medium | No database |
| `drogon` | C++ | High | Compiles drogon; use swap |
| `hical` | C++ | High | Boost headers; use swap |
| `uwebsockets` | C++ | Medium | Builds uSockets first |
| `agensio` | C++ | High | Asio and OpenSSL; use swap |
| `bun` | JavaScript | Low | No install step |
| `elysia` | JavaScript | Low | Bun installs one dependency |
| `fulmine` | JavaScript | Low | npm install; Node image |
| `blacksheep` | Python | Low | uvicorn worker count comes from the orchestrator |
| `granian` | Python | Low | Native wheel may need a compiler on musl |
| `robyn` | Python | Medium | Rust wheel download is large |
| `agoo` | Ruby | Medium | Native extension build |
| `falcon` | Ruby | Medium | Native extension build |
| `iodine` | Ruby | Medium | Native extension build |
| `rapidoid` | Java | Medium | Maven cache warms after the first run |
| `undertow` | Java | Medium | Maven cache warms after the first run |
| `vertx` | Java | Medium | Maven cache warms after the first run |
| `frankenphp` | PHP | Low | Worker mode; no composer step |
| `swoole` | PHP | High | PECL compilation; use swap |
| `workerman` | PHP | Medium | Composer installs workerman |
| `dusty` | Zig | Medium | Fetches pinned Zig and dusty source |
| `swerver` | Zig | Medium | Fetches pinned Zig and swerver source |
| `uwebzockets` | Zig | Medium | WebSocket target; also runs the WebSocket workload |

`robyn` 0.88 publishes neither a musllinux wheel nor an sdist. The musl image
therefore cannot install it from PyPI; use the glibc variant, or build Robyn
from its upstream source repository and update the musl Dockerfile explicitly.
Do not report a musl result until that image builds.

## Database rows

`actix-web` exposes `db_single`, `db_multi`, `fortunes`, and `updates`. Start
PostgreSQL before the run and disable managed startup:

```sh
docker compose -f deploy/postgres/compose.yaml up -d
BENCHMARK_DATABASE_MANAGED=0 \
DATABASE_URL=postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world \
BENCHMARK_FRAMEWORK_FILTER=actix-web scripts/run_small_host.sh
```

## WebSocket row

`uwebzockets` runs the HTTP suite and the WebSocket echo workload. Verify the
endpoint manually before benchmarking:

```sh
curl --fail --silent http://127.0.0.1:8080/healthz
```

For the Autobahn conformance suite, use a separate running target:

```sh
AUTOBAHN_TARGET_URL=ws://127.0.0.1:8080/ws scripts/run_autobahn_suite.sh
```

## What not to do

- Do not run the full registry: build memory and CPU contention dominate.
- Do not compare a glibc run from one boot with a musl run from another boot.
- Do not publish small-host numbers; use the dedicated-host workflow in
  [how_to_use.md](how_to_use.md).
- Do not run two frameworks at once; both use host networking and port 8080.
