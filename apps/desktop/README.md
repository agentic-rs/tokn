# Tokn Desktop

A macOS-first local control panel built with Tauri 2, React and TypeScript.

## Run locally

Install Rust, Xcode command-line tools, Node.js 22+ and pnpm. From this directory:

```sh
pnpm install --frozen-lockfile
pnpm desktop
```

`pnpm desktop` builds the repository's gateway, prepares the Tauri sidecar, then
launches the app. `pnpm bundle` builds a release gateway and an unsigned `.app`
under `src-tauri/target/release/bundle/macos`. Signing/notarization and distribution
are not configured yet. Builds currently target the host's macOS architecture.

The desktop Rust crate has its own workspace and lockfile so headless gateway
builds do not require WebKit/GTK or other desktop libraries.

## What the app manages

- **Overview:** check the first loopback `llm_api` listener (sorted by listener
  ID), start the bundled gateway, stop a gateway owned by this app, and view
  recorded usage for the past 24 hours.
- **Providers:** local account metadata and recorded token usage. Credentials
  remain in Rust; sign-in and account changes use the existing CLI. Recorded
  usage is not remaining provider quota.
- **Routing:** edit the `defaults`, `profiles`, `routes` and `model_scores`
  sections as TOML. Save validates the complete candidate with the existing configuration
  validator (including legacy fragments) and uses a locked, atomic replacement with a revision check.
  Unrelated sections and their comments are retained. Apply separately calls
  the local admin reload endpoint; reload failure leaves the saved file in
  place and reports that the running configuration has not changed. Runtime
  linking can reject a configuration accepted by the structural compiler.
- **History:** the newest 100 requests on the most recent recorded day, with
  request metadata details. Large payloads and pagination are available in the
  existing `tokn-gateway inspect` viewer, not this first desktop version.

The app uses `~/.tokn/router/config.toml` and the normal auth store. It supports both
legacy and v2 configuration with a fixed loopback API port. It does not migrate
configuration on launch. Legacy fragments are included in validation and
revision checks; the editor lists them because their profiles/scores can override
primary settings. Fragment editing remains in the CLI. Use the CLI to initialize
a workspace first. Config errors appear in the app.

A gateway already running outside the app can be viewed and reloaded but cannot
be stopped by the app. Closing/quitting gracefully stops the child owned by the
app, allowing up to 40 seconds for shutdown before terminating it. Unexpected
app termination uses child kill-on-drop when Rust cleanup runs; force-killing
the desktop process can still leave a child, which will be treated as external
on the next launch. Gateway logs use the configuration's existing logging setup; a bounded 8 KiB
stderr tail is shown when the managed child exits.
The webview has no shell or filesystem plugin permissions.

## Isolated local testing

Set `TOKN_DESKTOP_CONFIG` to an absolute config path before launching to use
an isolated gateway configuration. Use temporary persistence paths and a free
loopback port. Account metadata still comes from the normal auth store; the app
never changes credentials. Merely launching the app does not start a gateway.

```sh
pnpm build
pnpm prepare:gateway
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Also run repository formatting and Clippy as described in `AGENTS.md`. Native
checks need permission to bind loopback sockets for process ownership tests.
Manually check start/stop, external ownership, invalid/stale routing edits,
reload failure, empty history and existing history before release.

To exercise the real bundled gateway on a temporary listener (no inference calls):

```sh
TOKN_DESKTOP_TEST_GATEWAY="$(pwd)/../../target/debug/tokn-gateway" \
  cargo test --locked --manifest-path src-tauri/Cargo.toml -- --include-ignored
```
