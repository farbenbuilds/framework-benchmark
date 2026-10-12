## Description

Please describe the changes this PR makes and why it should be merged.

## Type of Change

- [ ] New framework integration (added framework under `frameworks/...`)
- [ ] Benchmark engine feature (orchestrator, load-generator, collector, reporter)
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] Infrastructure / CI / Build tooling (Docker, Nix, GitHub Actions)
- [ ] Documentation update
- [ ] Breaking change (change that modifies existing configuration or manifest format)

## Framework Submission Checklist (if adding/modifying a framework)

- [ ] Framework is located in `frameworks/<category>/<language>/<name>/`
- [ ] Manifest `framework.toml` contains valid metadata, capabilities, and runtime config
- [ ] Provides `Dockerfile.glibc` and/or `Dockerfile.musl` (or Nix flake derivations)
- [ ] Implements mandatory endpoints (`/healthz`, `/plaintext`, `/json`, etc.) with correct HTTP status and MIME types
- [ ] Verified locally with warmup and load tests

## General Checklist

- [ ] I have read the [Code of Conduct](CODE_OF_CONDUCT.md) and [Commit Convention](COMMIT_CONVENTION.md)
- [ ] My commit messages follow the conventional commit format
- [ ] Code compiles and passes all linter/formatting checks (`cargo fmt`, `cargo clippy`, etc.)
- [ ] I documented any new parameters, environment variables, or architectural changes
