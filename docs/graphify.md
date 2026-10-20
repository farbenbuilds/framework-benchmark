# Code Knowledge Graph with Graphify

Graphify builds an on-device graph of the repository so agents can query
symbols and relationships instead of grepping raw files. It is optional for
normal builds and does not change benchmark behavior.

## Install

The Python package is distributed as `graphifyy` and installs the `graphify`
and `graphify-mcp` commands. The `mcp` extra is required: without it the MCP
server exits with `ModuleNotFoundError: No module named 'mcp'`.

```sh
uv tool install 'graphifyy[mcp]'
graphify install --platform opencode
graphify install --platform codex
```

If the server already reported that error, upgrade the tool and reconnect:

```sh
uv tool install --upgrade 'graphifyy[mcp]'
opencode mcp list
```

The repository already registers the MCP server for both agents:

- OpenCode: `.opencode/opencode.json` under `mcp.servers.graphify`
- Codex: `.codex/config.toml` under `[mcp_servers.graphify]`

Both entries run `graphify-mcp graphify-out/graph.json` from the repository
root. Restart the agent after changing the configuration.

## Build the graph

The code-only build uses tree-sitter AST extraction and needs no API key. It
indexes Rust, Go, Python, JavaScript, C, C++, C#, Java, PHP, Ruby, Zig, shell,
TOML, YAML, and other supported source files.

```sh
graphify extract . --code-only
graphify god-nodes --top 20
graphify query "Where is the sampler task stopped on error?" --budget 1500
```

Generated files are written under `graphify-out/` and ignored by Git. Rebuild
incrementally after refactors:

```sh
graphify update .
graphify cluster-only .
```

A full build that also indexes Markdown documents uses semantic extraction.
That path can use Gemini when `GEMINI_API_KEY` is set; otherwise the agent
performs the extraction. Code-only is sufficient for architecture reviews.

## Use it during review

Start with the highest-connectivity symbols, then query the areas changed by a
patch. Useful commands:

```sh
graphify god-nodes
graphify explain "BackgroundSampler"
graphify affected "record_sample"
graphify query "Which tasks can outlive their owner?" --dfs
graphify path "BackgroundSampler" "execute_single_workload"
```

Treat graph edges as navigation hints, not proof. The extraction tags each
edge as `EXTRACTED`, `INFERRED`, or `AMBIGUOUS`. Confirm every finding by
reading the cited source location before changing code.

## What to look for

- Resource lifetime: tasks, handles, files, and containers that are not stopped
  on every error path.
- Hot-path allocation: `format!`, `Vec`, `String`, or boxed values inside
  request loops.
- Central hubs: error enums and manifest types with many edges often indicate
  boundary violations.
- Isolated communities: duplicated parsing or formatting that should be one
  pure function.
- Path and command boundaries: manifest fields that reach filesystem paths,
  Docker arguments, or generated Markdown.

Graphify is a navigation aid. The verification gates remain
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test --workspace`, and `sh scripts/check_conventions.sh`.

## MCP tools

When the agent connects to the MCP server it can list, query, and traverse the
graph without shelling out. If the server is unavailable, run the CLI commands
above; the graph format and results are the same.

Do not commit `graphify-out/` to the repository. It is a local cache and may
contain machine-specific paths.
