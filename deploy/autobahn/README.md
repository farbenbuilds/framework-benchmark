# Autobahn protocol conformance

Run a candidate WebSocket server on the benchmark host, then execute:

```sh
scripts/run_autobahn_suite.sh
```

The runner uses the official pinned Autobahn image in `fuzzingclient` mode, runs every enabled upstream case, and writes its HTML and JSON report to `results/autobahn`.

Set `AUTOBAHN_TARGET_URL` for a nondefault server endpoint, `AUTOBAHN_RESULTS_DIR` for another report directory, or `CONTAINER_RUNTIME=podman` to use Podman.
