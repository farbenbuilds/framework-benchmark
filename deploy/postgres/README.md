# PostgreSQL benchmark database

This deployment supplies the TechEmpower-compatible `hello_world` database:

- `world`: 10,000 rows with `id` and `randomnumber`
- `fortune`: 12 rows used by `/fortunes`

Run it with Docker Compose or Podman Compose:

```sh
docker compose -f deploy/postgres/compose.yaml up -d
podman compose -f deploy/postgres/compose.yaml up -d
```

The default connection string is:

```text
postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world
```

Set `POSTGRES_BIND_ADDRESS`, `POSTGRES_PORT`, `POSTGRES_DB`, `POSTGRES_USER`,
and `POSTGRES_PASSWORD` before startup to change it. PostgreSQL applies
`seed.sql` only when it initializes a new `postgres_data` volume.

To recreate the data explicitly, use `docker compose ... down -v` or set
`BENCHMARK_PURGE_DATABASE=1` when running `scripts/cleanup_environment.sh --apply`.
