# tokn

Local, account-aware LLM gateway for OpenAI-compatible clients.

`tokn` runs a local HTTP API and optional MITM forward proxy, routes requests
across configured provider accounts, and records local usage/session/request
history. GitHub Copilot is still the default provider, but the gateway now also
supports OpenAI, ChatGPT Codex, DeepSeek, llama.cpp, Z.ai, and Zhipu BigModel.

The shipped Cargo package is `tokn-gateway-cli` and the binary is
`tokn-gateway`.

## Active Development

**This project is moving quickly.** Config shape, database schemas, API
behavior, provider routing, and proxy behavior are all expected to change as the
gateway settles.

## Features

- OpenAI-compatible local API on `127.0.0.1:4141`.
- Endpoints for `GET /v1/models`, `POST /v1/chat/completions`,
  `POST /v1/responses`, and `POST /v1/messages`.
- Profile-prefixed routes like `/{profile}/v1/chat/completions`.
- Multiple accounts per provider with active/fallback/disabled tiers.
- Route modes for passthrough, provider switching, exact routing, catalogue
  routing, and fuzzy model-family routing.
- Streaming support through the shared request pipeline.
- Local SQLite-backed usage, session, and request-body persistence.
- Optional HTTP CONNECT proxy with local CA generation for agent workflows.

Docker PR trial helpers live under [`scripts/`](/Users/clouds/.codex/worktrees/59e1/llm-router/scripts/README.md).
They load the CI image artifact, run a persistent gateway container, and launch
disposable Codex/opencode/pi agent containers through Bun.

## Install

From this workspace:

```sh
cargo install --path crates/gateway-cli
```

Or run directly during development:

```sh
cargo run -p tokn-gateway-cli -- --help
```

## Quick Start

Add an account, start the API server, then point any OpenAI-compatible client at
the local base URL.

```sh
# Interactive provider/account setup.
tokn-gateway account add

# GitHub Copilot device-flow login.
tokn-gateway account login --provider github-copilot

# Or import a static API key from the environment.
OPENAI_API_KEY=sk-... tokn-gateway account import --provider openai --from env --id openai

# Start the local API server.
tokn-gateway serve

# Send a chat-completions request.
curl http://127.0.0.1:4141/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "hi"}],
    "stream": true
  }'
```

For clients that expect `OPENAI_BASE_URL`, use:

```sh
export OPENAI_BASE_URL=http://127.0.0.1:4141/v1
```

## Config And Data

Default files live under `~/.tokn/router/`:

- `config.toml`: runtime config.
- `config.d/`: non-secret, agent-owned binding and profile overlays.
- `auth.yaml`: user-managed and shared account credentials.
- `auth.d/`: credential-only fragments owned by linked agents.
- `usage.db`: usage summaries.
- `sessions.db`: session affinity data.
- `requests/`: archived request bodies.
- `ca/`: proxy CA material.
- `logs/`: file logs when enabled.

Print the config path with:

```sh
tokn-gateway config path
```

Minimal config:

```toml
[server]
host = "127.0.0.1"
port = 4141

[defaults]
mode = "route"
# Required when mode is "passthrough" or "switch"; optional otherwise.
# default_provider_id = "github-copilot"
# Omit providers/accounts to allow every configured active account.
# providers = ["github-copilot", "openai"]
# accounts = ["personal", "openai"]

[pool]
strategy = "round_robin"
failure_cooldown_secs = 60
session_ttl_secs = 18000

[db]
enabled = true
record_sessions = true
record_request_bodies = true
body_max_bytes = 10485760

[proxy]
# url = "http://user:pass@proxy.example.com:8080"
# url = "socks5h://127.0.0.1:1080"
# system = false
# no_proxy = ["localhost", "127.0.0.1", ".internal"]
```

Profiles merge with `[defaults]` and are selected by prefixing the route:

```toml
[profiles.coding]
mode = "fuzzy"
agent_id = "codex-cli"
# Overrides [defaults].default_provider_id when present.
# default_provider_id = "github-copilot"
providers = ["github-copilot"]
accounts = ["personal"]

[[profiles.coding.model_families]]
name = "glm"
members = ["glm-4.6", "glm-4.7"]
```

Requests to `/v1/...` use `[defaults]`. Requests to `/coding/v1/...` use
`[defaults]` plus `[profiles.coding]`. Profile `providers` entries must be
canonical provider ids; if omitted, the profile inherits the default provider
set. Profile `accounts` entries must be configured account ids; if omitted, the
profile inherits the default account set. Profile `model_families`, when
present, replaces default model families for that profile. API `passthrough`
and `switch` policies require `default_provider_id` so the router can target a
deterministic provider while preserving request bytes.

## Database

When `[db].enabled` is true, the gateway writes local SQLite state under
`~/.tokn/router/` unless paths are overridden:

- `usage.db` stores aggregate request usage for `tokn-gateway usage`.
- `sessions.db` stores session affinity and routing state.
- `requests/` stores day-rotated request databases named like
  `2026-06-09.db`.

The request DBs are not a single `requests.db` file. They record request and
response metadata, and can also persist request bodies when
`record_request_bodies = true`. Use `body_max_bytes` to cap stored body size.

Schema migrations are applied when the databases are opened. To inspect or
apply them explicitly:

```sh
tokn-gateway migration
tokn-gateway migration --commit
```

## Accounts

Accounts are managed separately from `config.toml`.

```sh
tokn-gateway account add
tokn-gateway account list
tokn-gateway account status
tokn-gateway account show personal
tokn-gateway account refresh personal
tokn-gateway account switch --only personal
tokn-gateway account remove personal
```

Non-interactive imports support `env`, `string`, `file`, `stdin`, and
provider-specific sources:

```sh
tokn-gateway account import --provider openai --from env --id openai
tokn-gateway account import --provider deepseek --from env --id deepseek
tokn-gateway account import --provider github-copilot --from gh --id personal
tokn-gateway account import --provider github-copilot --from copilot-plugin --id personal
```

Default environment variable names are derived from the provider id and
credential kind, for example `OPENAI_API_KEY`, `DEEPSEEK_API_KEY`,
`ZAI_API_KEY`, and `GITHUB_COPILOT_REFRESH_TOKEN`.

## Providers

| id | auth | primary endpoints |
| --- | --- | --- |
| `github-copilot` | GitHub OAuth refresh token | chat completions |
| `openai` | API key | chat completions, responses |
| `codex` | OpenAI refresh token or API key | responses |
| `deepseek` | API key | chat completions, messages |
| `llama-cpp` | API key | chat completions |
| `zai`, `zai-coding-plan` | API key | chat completions |
| `zhipuai`, `zhipuai-coding-plan` | API key | chat completions |

Provider ids are canonical config values. Z.ai and Zhipu coding-plan ids use
coding-plan upstream paths; the non-coding ids use the regular PAAS paths.

Per-account `base_url` can override the provider default. Manual account
commands write account records to `auth.yaml`; linked agents keep transferred
credentials in their own `auth.d/<agent>.yaml` fragment. The gateway loads both
locations as one account pool, while preserving the file that owns each account
when credentials are refreshed or removed.

```yaml
version: 1
accounts:
  - id: local
    provider: llama-cpp
    enabled: true
    tier: active
    auth_type: bearer
    api_key: unused
    base_url: http://127.0.0.1:8080/v1
```

## Commands

```text
tokn-gateway account add [--provider PROVIDER] [--id ID]
tokn-gateway account login [--provider PROVIDER] [--id ID] [--no-proxy]
tokn-gateway account import --provider PROVIDER --from env|string|file|stdin|gh|copilot-plugin [--id ID]
tokn-gateway account list [--no-quota]
tokn-gateway account status [ID]
tokn-gateway account switch --only ID
tokn-gateway headers [--account ID]
tokn-gateway serve [--host HOST] [--port PORT] [--with-proxy] [--no-proxy]
tokn-gateway proxy start [--host HOST] [--port PORT] [--route-mode MODE] [--passthrough]
tokn-gateway proxy env [--shell sh|bash|zsh|fish|pwsh]
tokn-gateway proxy shell [--shell /path/to/shell]
tokn-gateway proxy codex|opencode|pi [--npx] [ARGS...]
tokn-gateway proxy run [--npx] codex|opencode|pi [ARGS...]
tokn-gateway proxy exec COMMAND [ARGS...]
tokn-gateway proxy ca path|show|regenerate
tokn-gateway usage [--since 24h] [--account ID] [--provider PROVIDER]
tokn-gateway config get|set|unset KEY [--account ID] [--add]
tokn-gateway config list|edit|path|init
tokn-gateway agent list
tokn-gateway agent show codex-cli|opencode
tokn-gateway agent import codex-cli|opencode [--yes]
tokn-gateway agent link codex-cli|opencode [--profile NAME] [--mode MODE] [--yes]
tokn-gateway agent link opencode --use-main-accounts [--mode passthrough|switch] [--provider ID] [--source-provider ID]... [--yes]
tokn-gateway agent sync codex-cli|opencode|--all [--yes]
tokn-gateway agent unlink codex-cli|opencode [--backup-id ID]
tokn-gateway migration [--commit|--rollback]
tokn-gateway update
tokn-gateway smoke provider|model|send ...
```

Route modes are `passthrough`, `switch`, `exact`, `route`, and `fuzzy`.

`agent link` writes its binding and generated profile to
`config.d/<agent>.toml`, so the primary config remains untouched. When a normal
agent-owned link transfers credentials, its matching `auth.d/<agent>.yaml`
fragment forms a separately backed up and restored credential bundle; the shared
root `auth.yaml` stays unchanged. `--use-main-accounts` creates no auth fragment:
OpenCode keeps its local credentials unchanged and routes selected provider
namespaces through the gateway's existing account pool. `--source-provider` is
repeatable and defaults to `openai`; raw `passthrough` and `switch` links require
a target `--provider` (or a configured default provider) that supports
OpenCode's Chat Completions endpoint. Codex CLI does not yet support
main-account links because its credential bootstrap would need to be changed.
To move a pre-`auth.d` imported link, unlink it first so its local credentials
are restored, then link it again.

## Proxy Mode

The proxy runs a local HTTP CONNECT forward proxy. Requests for recognized LLM
API hosts are intercepted and routed through the same account pool; unrelated
hosts are tunneled through untouched.

```sh
tokn-gateway proxy start
tokn-gateway proxy ca show
eval "$(tokn-gateway proxy env)"
```

The generated environment includes:

- `HTTPS_PROXY` and `HTTP_PROXY`.
- `SSL_CERT_FILE`, `REQUESTS_CA_BUNDLE`, `CURL_CA_BUNDLE`, and
  `GIT_SSL_CAINFO` pointing at a merged system-root plus tokn CA bundle.
- `NODE_EXTRA_CA_CERTS` pointing at the tokn CA certificate.
- `NO_PROXY` for local loopback addresses.

Useful wrappers:

```sh
tokn-gateway proxy shell
tokn-gateway proxy codex --help
tokn-gateway proxy exec curl https://api.openai.com/v1/models
```

Proxy config:

```toml
[proxy_mode]
host = "127.0.0.1"
port = 4142
route_mode = "route"

[proxy_mode.provider_modes]
# openai = "switch"
# github-copilot = "passthrough"

# Optional; defaults to ~/.tokn/router/ca
# ca_dir = "/some/path"

# Extend or trim the interception set.
# intercept_hosts = ["my-gateway.example.com"]
# passthrough_hosts = ["api.githubcopilot.com"]
```

`tokn-gateway serve --with-proxy` runs both the API listener and proxy in one
process. API routes use `[defaults]` or a named profile; proxy interception uses
`[proxy_mode].route_mode` unless overridden with `--proxy-route-mode`.

## LAN Bootstrap

By default, listeners must bind to loopback. To expose a trusted LAN gateway,
bind explicitly and opt into the risk:

```sh
tokn-gateway serve --host 0.0.0.0 --with-proxy --insecure-allow-remote
```

This exposes helper routes on the API listener:

- `/-/lan/bootstrap.json`
- `/-/lan/ca.crt`
- `/-/lan/env?shell=sh|bash|zsh|fish|pwsh`

The server prints the CA SHA-256 fingerprint at startup. Verify that fingerprint
before trusting a CA fetched over the LAN. The private CA key is never served.

## Development

This is a Rust workspace. The runtime entrypoint lives in
`crates/gateway-cli`; `crates/router` owns the HTTP API/router/proxy wiring.

```sh
cargo fmt --all
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Schema snapshots track the active release line in `VERSION`. If `VERSION` is on
`v0.2.x`, keep snapshot updates on the existing `v0.2.0.sql` files.

## License

MIT.

## Inspiration

Inspired by [`sub2api`](https://github.com/Wei-Shaw/sub2api).
