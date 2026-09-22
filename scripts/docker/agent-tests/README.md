# Container agent tests

Run the gateway and agent in separate containers while the native gateway keeps
running. Each run gets a fresh database volume. Only the selected host config,
config fragments, auth file, and auth shards are mounted, all read-only. No host
database directory is mounted. The agent shares the gateway's network namespace,
so loopback listeners work without publishing ports.

## Run

Requires Docker or Podman and Bun 1.3.13. Commands below run from the repository
root; suite paths resolve relative to the suite file, while `--output` resolves
relative to the current working directory.

```sh
docker build -t tokn-gateway-cli:agent-test .
TOKN_CONTAINER_ENGINE=docker bun --cwd scripts docker agent-test build-agent --agent opencode
TOKN_CONTAINER_ENGINE=docker bun --cwd scripts docker agent-test run \
  --suite ../examples/docker/agent-tests/opencode.json

TOKN_CONTAINER_ENGINE=docker bun --cwd scripts docker agent-test build-agent --agent pi
TOKN_CONTAINER_ENGINE=docker bun --cwd scripts docker agent-test run \
  --suite ../examples/docker/agent-tests/pi.json

TOKN_CONTAINER_ENGINE=docker bun --cwd scripts docker agent-test build-agent --agent dsh
TOKN_CONTAINER_ENGINE=docker bun --cwd scripts docker agent-test run \
  --suite ../examples/docker/agent-tests/dsh.json
```

The example reads `~/.tokn/router/config.toml` and `auth.yaml`. Set `auth_file` in
your local suite to use another credential file. Set `config_dir` or `auth_dir`
to override automatic sibling `config.d`/`auth.d` discovery. Keep credentials
out of the repository. These mounts are read-only, so credentials that need
refreshing must first be refreshed outside the agent test.

The examples expect existing `opencode-deepseek` and `opencode-codex` profiles.
Edit each case's `base_path` to match your profiles. `model` names the model in
the agent; optional `upstream_model` supplies a qualified router model identifier.
`api` explicitly selects Chat Completions or Responses. Model eligibility and
provider errors are real test failures; the harness does not bypass the router's
model catalogue. In particular, a provider that omits `gpt-5.6-luna` from its
catalogue may fail these Luna cases even if a raw upstream diagnostic succeeds.

The config must enable persistence/session recording and use default database
paths. Use a dedicated config if your native configuration has absolute paths.
Its listener must match `router_url` (default `http://127.0.0.1:4141`). The runner
clears inherited HTTP proxy environment variables; an explicit upstream proxy
in router config must still be reachable from inside the container. Legacy v1
configs may use `serve_args: ["--no-proxy"]` to disable their interception proxy.
Native v2 configs should leave `serve_args` empty.
For a proxy running on the macOS host, use an agent-test config whose proxy URL names
`host.containers.internal` instead of `localhost`.
Logging must target `stderr` or `both`; the runner sets `RUST_LOG=info` so startup
and persistence shutdown confirmations remain observable.

Select cases with repeated `--case ID` flags. `timeout_secs` bounds each agent
container (default 120). A timeout stops the actual container. SIGINT/SIGTERM
also trigger cleanup. A failed case does not prevent later cases from running.
OpenCode and Pi cases require valid structured events, exact expected text, and
a terminal completion; read-tool cases additionally require the completed read
of a random fixture token. Exit status zero alone is insufficient. The pinned
DSH 0.1.5-rc.2 headless CLI exposes only its final answer, so its adapter accepts
text probes only and verifies exact output plus successful process completion.
It rejects read-tool probes before any containers are created.

## Captures and import

Outputs default to ignored `tmp/docker-agent-tests/<run>/`. `report.json` records case
results, errors, shutdown status, and the retained private volume name. Raw agent
and gateway logs and exported databases may contain sensitive request data;
directories are mode 0700 and files mode 0600. The private client API key is
passed through process environment and is never embedded in generated files.

The runner allows 40 seconds for shutdown. Only an exit code of zero plus the
gateway's persistence-cleanup confirmation permits export. Check
`export_complete: true` before using the `export/` directory. A failed export may
leave partial files; recover from the retained volume, rather than importing that
directory. The volume is deliberately never removed automatically.

Run the import on the **host**, where SQLite locks coordinate with the native
gateway. Never open host database files from the Docker VM: shared filesystem
locking can differ across that boundary.

```sh
cargo run --locked -p tokn-gateway-cli --bin tokn-gateway -- \
  history import --source tmp/docker-agent-tests/<run>/export --json

# After reviewing the dry-run result:
cargo run --locked -p tokn-gateway-cli --bin tokn-gateway -- \
  history import --source tmp/docker-agent-tests/<run>/export --commit --json
```

By default the destination uses configured native persistence paths. Use global
`--config PATH` for a different config, or `--destination ROOT` for explicit
`ROOT/usage.db`, `ROOT/sessions.db`, and `ROOT/requests/` paths. Destination usage
and session databases must already be initialized. Import accepts closed current
schema databases, compares columns by name, inserts new rows, skips identical
rows, and refuses conflicting rows. Usage IDs are allocated locally. All writes
share one transaction; dry-run performs the same validation then rolls back.
An existing archived request day is refused. SQLite's attachment limit bounds
each capture to nine daily request databases. Split longer runs into captures.

After verifying the import, remove the retained volume explicitly:

```sh
docker volume rm <volume_name-from-report>
```

## Extend and verify

`agents/` owns preparation and output interpretation for each agent. The current
images pin OpenCode 1.18.10, Pi 0.85.1, and DSH 0.1.5-rc.2. Register a new adapter
in `agents/index.ts`, with its pinned image and offline tests.
`modes.ts` owns container connectivity; only `api` is currently implemented.
`cases.ts` validates the matrix independently of both registries, so adding an
agent does not require duplicating orchestration. Add models and probes to the
suite JSON. Unsupported agents/modes fail before creating resources.

```sh
bun install --cwd scripts --frozen-lockfile
bun --cwd scripts test
bun --cwd scripts check
cargo test --locked -p tokn-persistence --all-features --test history_import
cargo test --locked -p tokn-gateway-cli --all-features cli::history
```

Offline tests run in CI without credentials or paid model requests. Live agent tests
are explicit local runs and return nonzero if any case or cleanup step fails.
