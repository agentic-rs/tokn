# Scripts

This directory contains repo-local helper tooling that should not make the Rust
workspace root a JavaScript package.

## Docker PR Trial

Download and load the `tokn-gateway-cli-image` artifact from CI, build the agent
runner image once, and start the gateway:

```sh
bun --cwd scripts docker load --pr 67
```

If you already downloaded the artifact tar manually:

```sh
bun --cwd scripts docker load ./tokn-gateway-cli-image.tar
```

Then:

```sh
bun --cwd scripts docker build-agent
bun --cwd scripts docker up
```

Run disposable agent containers against that gateway:

```sh
bun --cwd scripts docker agent --agent opencode --mode api-route
bun --cwd scripts docker agent --agent codex --mode proxy-switch
```

Forward arguments to the selected agent after `--`:

```sh
bun --cwd scripts docker agent --agent codex --mode api-route -- --help
```

Modes:

- `api-route`: point the agent at `http://gateway:4141/v1`; gateway owns
  credentials and routing.
- `proxy-switch`: run through `http://gateway:4142`; gateway injects upstream
  credentials for recognized providers.
- `api-passthrough`: diagnostic API passthrough endpoint.
- `proxy-passthrough`: diagnostic transparent proxy passthrough; the concrete
  agent may need its own upstream login.

Lifecycle:

```sh
bun --cwd scripts docker status
bun --cwd scripts docker logs
bun --cwd scripts docker down
```

`down` keeps the persistent gateway state volume. To remove containers and
volumes, use the explicit reset guard:

```sh
bun --cwd scripts docker reset --yes
```

The CLI uses Podman by default. Set `TOKN_CONTAINER_ENGINE=docker` only if you
want the same lifecycle managed through Docker-compatible commands.
