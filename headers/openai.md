# OpenAI Headers

Reference source: `Wei-Shaw/sub2api` at `2fb9fb2f715903de0a7328e8e1531beef3f3b2e9`.

This document records the upstream request headers Sub2API synthesizes for OpenAI Platform API-key accounts and ChatGPT/Codex OAuth accounts.

## Account Types

OpenAI API-key accounts target the OpenAI Platform API and authenticate with:

```http
Authorization: Bearer <api_key>
```

OpenAI OAuth/Codex accounts target ChatGPT internal Codex endpoints and authenticate with:

```http
Authorization: Bearer <access_token>
```

Sub2API removes inbound auth residue before injecting upstream auth:

```http
authorization
x-api-key
x-goog-api-key
```

For OAuth ChatGPT internal requests, Sub2API also sets the request host separately:

```text
Host: chatgpt.com
```

In Go this is `req.Host = "chatgpt.com"`, not `Header.Set("Host", ...)`.

## Constants

Sub2API OpenAI/Codex constants:

```text
chatgptCodexURL = https://chatgpt.com/backend-api/codex/responses
openaiPlatformAPIURL = https://api.openai.com/v1/responses
codexCLIUserAgent = codex_cli_rs/0.125.0
codexCLIVersion = 0.125.0
openAIWSBetaV1Value = responses_websockets=2026-02-04
openAIWSBetaV2Value = responses_websockets=2026-02-06
```

## Common Platform Headers

For OpenAI Platform model listing, Sub2API sends:

```http
Accept: application/json
Authorization: Bearer <api_key>
```

For normal OpenAI Platform POST requests, Sub2API injects:

```http
Authorization: Bearer <api_key>
Content-Type: application/json
Accept: application/json | text/event-stream
```

`Accept` is `text/event-stream` for streaming and `application/json` otherwise.

## OAuth ChatGPT Internal Headers

For OAuth requests to ChatGPT internal APIs, Sub2API adds or overwrites:

```http
Authorization: Bearer <access_token>
chatgpt-account-id: <account chatgpt_account_id, when configured>
OpenAI-Beta: responses=experimental
originator: <resolved originator>
Content-Type: application/json
Accept: text/event-stream
```

Originator resolution:

```text
1. Use inbound originator when non-empty.
2. Else, if the request looks like an official Codex client, use codex_cli_rs.
3. Else use opencode.
```

Sub2API special-cases some compatibility bridge paths by removing both:

```http
OpenAI-Beta
originator
```

## User-Agent Rules

For OAuth ChatGPT internal requests, Sub2API tries to avoid browser-like or non-Codex upstream identity:

```text
1. If account has custom OpenAI User-Agent, use it.
2. If ForceCodexCLI is enabled, force codex_cli_rs/0.125.0.
3. If OAuth request is not a Codex CLI request, force codex_cli_rs/0.125.0.
4. If OAuth final UA still looks browser-like, replace it with the configured Codex UA fallback.
```

API-key OpenAI Platform requests may also use a configured account User-Agent, but they do not need the ChatGPT/Codex fallback behavior.

## Session Headers

Sub2API treats OpenAI OAuth session headers as upstream-affecting and isolates them per API key to avoid cross-user collisions.

Client/header inputs:

```http
session_id: <client session>
conversation_id: <client conversation>
```

Body fallback:

```json
{
  "prompt_cache_key": "..."
}
```

Isolation rule:

```text
isolated = xxhash64("k<api_key_id>:<raw>") formatted as 16 lowercase hex chars
```

Upstream OAuth output:

```http
session_id: <isolated session_id or isolated prompt_cache_key>
conversation_id: <isolated conversation_id or isolated prompt_cache_key>
```

For compact paths, Sub2API sets:

```http
Accept: application/json
version: 0.125.0
session_id: <isolated compact session>
```

## HTTP Request Passthrough Allowlist

Sub2API has two OpenAI header allowlists.

Non-passthrough allowlist:

```text
accept-language
content-type
conversation_id
user-agent
originator
session_id
x-codex-turn-state
x-codex-turn-metadata
```

Passthrough allowlist:

```text
accept
accept-language
content-type
conversation_id
openai-beta
user-agent
originator
session_id
x-codex-turn-state
x-codex-turn-metadata
```

Optional timeout headers are passed only when `gateway.openai_passthrough_allow_timeout_headers` is enabled:

```text
x-stainless-timeout
x-stainless-read-timeout
x-stainless-connect-timeout
x-request-timeout
request-timeout
grpc-timeout
```

## Raw OpenAI-Compatible Fallback

When forwarding to third-party OpenAI-compatible Chat Completions upstreams, Sub2API intentionally does not reuse the Codex allowlist.

Only these client headers are forwarded:

```text
accept-language
user-agent
```

It explicitly avoids leaking Codex/ChatGPT headers such as:

```text
originator
session_id
conversation_id
x-codex-turn-state
x-codex-turn-metadata
```

Reason: those headers are meaningful for ChatGPT OAuth/Codex but can pollute metrics or trigger strict 400s on third-party OpenAI-compatible providers.

## WebSocket Headers

OpenAI WebSocket mode builds a fresh header map rather than copying the HTTP request wholesale.

Always set:

```http
authorization: Bearer <access_token>
OpenAI-Beta: responses_websockets=2026-02-06
```

For the older Responses WebSocket transport, Sub2API uses:

```http
OpenAI-Beta: responses_websockets=2026-02-04
```

Optional forwarded/synthesized headers:

```http
accept-language: <inbound accept-language>
session_id: <isolated or raw session>
conversation_id: <isolated or raw conversation>
x-codex-turn-state: <turn state>
x-codex-turn-metadata: <turn metadata>
chatgpt-account-id: <account chatgpt_account_id>
originator: <resolved originator>
user-agent: <custom/inbound/forced Codex UA>
```

For OAuth WebSocket accounts, `session_id` and `conversation_id` are isolated with the same API-key-id hash scheme. If the client omits both, `prompt_cache_key` can become the session fallback.
