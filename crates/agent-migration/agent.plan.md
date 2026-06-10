# `agent` subcommand redesign

## Goal

Replace the one-shot `migrate`/`rollback` pair with a **verbs family** that manages
bindings between local agents (opencode, codex-cli, …) and gateway profiles.
`[agents.<slug>]` in the gateway config is the **CLI-managed source of truth**; the
CLI reconciles it into `[profiles.*]`, account imports, and the agent's own config
file. The gateway runtime is unchanged (it still routes off `[profiles.*]` / `[defaults]`).

## Binding model (no config schema change)

`tokn_config::AgentConfig` (`[agents.<slug>]`) already carries the whole binding:

- `profile = Some("work")` -> agent routes via `/work/v1`.
- `profile = None`         -> agent routes via the non-prefixed `/v1` (gateway `[defaults]`).
- `mode`                   -> route mode for the materialised profile.
- `sync = true`            -> CLI-managed; eligible for `agent sync --all`.

## CLI surface

```
agent list                                          # read-only
agent show   <AGENT>                                # read-only
agent import <AGENT> [--yes]                        # import creds only
agent link   <AGENT> [--profile P] [--mode M] [--yes]
agent sync   [<AGENT> | --all] [--yes]
agent unlink <AGENT> [--backup-id <id>]
```

`<AGENT>` is parsed via `AgentId::from_slug` + adapter-registry check, so the
supported set lives in exactly one place.

## Target resolution (drives base URL + binding)

```
explicit --profile P         -> Named(P)        base=/P/v1     write [profiles.P], binding.profile=Some(P)
else existing binding.profile -> Named(that)
else accounts discovered      -> Named(<slug>)   base=/<slug>/v1
else (no accts, no profile)   -> Defaults        base=/v1       no profile table, binding.profile=None
```

The `/v1` fallback only triggers when nothing else is specified and no credentials
exist. `link` may override profile/mode; `sync` never does (pure source-of-truth).

## Architecture (`crates/agent-migration`, name kept)

- Unify on `tokn_core::AgentId`; delete `AgentKind` (crate) + `CliAgentKind` (CLI).
- `adapter.rs`    — `AgentAdapter` trait + `adapter_for` / `supported_agents` registry.
- `adapters/*`    — opencode + codex implementations (discovery + config rewrite).
- `reconcile.rs`  — engine: `ReconcileRequest`/`ReconcilePlan`/`apply_reconcile`,
  `import_accounts`, `unlink`, binding/profile projection, backup + manifest.
- `manifest.rs`   — backup manifest (retargeted to `AgentId`).
- `status.rs`     — `list_agents` / `show_agent`.

```rust
pub trait AgentAdapter {
  fn id(&self) -> AgentId;
  fn default_provider_id(&self) -> &'static str;
  fn auth_path(&self, home: &Path) -> PathBuf;
  fn config_path(&self, home: &Path) -> PathBuf;
  fn discover_accounts(&self, home: &Path, timestamp: &str) -> Result<Vec<Account>>; // best-effort
  fn rewrite_config(&self, home: &Path, base_url: &str) -> Result<Vec<PlannedEdit>>;
}
```

## Breaking changes

- `agent migrate` -> `agent link`; `agent rollback` -> `agent unlink`; adds
  `list`/`show`/`import`/`sync`.
- Crate API renamed (`AgentKind`, `Migrate*`, `*_migration` -> `AgentId`,
  `Reconcile*`, `apply_reconcile`/`import_accounts`/`unlink`). Only `gateway-cli`
  consumes the crate.
- No longer refuses on zero accounts — falls back to `/v1`.
- Backup manifests stay slug-compatible (`opencode`, `codex-cli`).
