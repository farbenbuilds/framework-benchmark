# Security Policy

## Supported Versions

Security fixes apply to the current development revision and the latest
released version on the `main` branch. Older revisions do not receive backports.

| Version | Supported |
| --- | --- |
| Current development (`main`) | Yes |
| Older commits / snapshots | No |

## Reporting a Vulnerability

Do not open a public issue, pull request, or discussion that contains an
undisclosed security vulnerability.

Send a private report to:
- [trananhquan1009@gmail.com](mailto:trananhquan1009@gmail.com)
- [noah1109.tran@gmail.com](mailto:noah1109.tran@gmail.com)

Please include:
- The affected revision, crate, and target platform (Linux distribution, kernel version, architecture);
- A minimal reproducer script or configuration;
- Expected and observed behavior;
- Security impact, attack vectors, and preconditions;
- Any logs, crash traces, or sanitizer outputs (with sensitive data removed); and
- Any suggested mitigation or patch.

The maintainers will acknowledge the report, evaluate and reproduce the issue,
prepare a fix, and coordinate disclosure. Response time depends on severity and
maintainer availability.

## Security Boundaries & Execution Model

The **Framework Benchmark** platform executes potentially untrusted third-party code
submitted by community authors across multiple language runtimes and frameworks.
The security surface includes:

1. **Host Isolation & Sandboxing**:
   - Framework processes under test must execute inside unprivileged Docker containers or isolated Nix derivations with restricted cgroups v2 limits (CPU quota, memory limits, process number bounds).
   - Containers must not run in `--privileged` mode or mount sensitive host directories (e.g. host `/proc`, `/sys`, root filesystems, or Docker daemon sockets).
   - Linux runner daemons must operate with least privileges necessary for scheduling benchmarks.

2. **Network & Traffic Generation Boundaries**:
   - The synthetic load generator produces high-volume network traffic directed strictly to designated loopback (`127.0.0.1`) or isolated private test VLAN ports.
   - Load generation against external, non-consenting IP addresses or public networks is strictly forbidden.

3. **Results & Artifact Integrity**:
   - Automated git push mechanisms on benchmark hosts must use scoped deployment tokens or deploy keys restricted to content updates in `results/` and `README.md`.
   - Manifests (`framework.toml`) undergo strict deserialization checks to prevent path traversal or shell injection vulnerabilities during container builds and runner dispatch.

4. **Runner Dependencies & Memory Safety**:
   - Core orchestrator and load generator crates are implemented in Rust. Memory-safety violations, unsafe code vulnerabilities, or denial-of-service defects within the runner itself fall within our security boundary and should be reported immediately.

## Disclosure

Please allow a reasonable remediation and release window before publication.
Security advisories will credit reporters who request attribution and will
describe affected versions, impact, and upgrade guidance without exposing
unnecessary exploit detail before a fix is available.
