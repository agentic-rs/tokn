# Anthropic Headers

Reference source: `Wei-Shaw/sub2api` at `2fb9fb2f715903de0a7328e8e1531beef3f3b2e9`.

This document records the upstream request headers Sub2API synthesizes for Anthropic/Claude accounts. It is provider-specific header behavior, not client ingress auth behavior.

## Account Types

Sub2API separates Anthropic OAuth accounts from Anthropic API-key accounts.

OAuth accounts use:

```http
Authorization: Bearer <access_token>
```

API-key accounts use:

```http
x-api-key: <api_key>
```

Before sending an upstream request, Sub2API removes inbound auth residue when constructing forwarded Anthropic requests:

```http
authorization
x-api-key
x-goog-api-key
cookie
```

## Claude Code Mimic Headers

Sub2API forces Claude Code-like request identity headers for Claude Code-scoped OAuth credentials instead of trusting downstream client headers.

Default header constants:

```http
User-Agent: claude-cli/2.1.92 (external, cli)
X-Stainless-Lang: js
X-Stainless-Package-Version: 0.70.0
X-Stainless-OS: Linux
X-Stainless-Arch: arm64
X-Stainless-Runtime: node
X-Stainless-Runtime-Version: v24.13.0
X-Stainless-Retry-Count: 0
X-Stainless-Timeout: 600
X-App: cli
Anthropic-Dangerous-Direct-Browser-Access: true
```

Additional forced values:

```http
Accept: application/json
x-client-request-id: <new uuid when missing>
```

For streaming Anthropic requests, Sub2API also sets:

```http
x-stainless-helper-method: stream
```

## Version Header

Sub2API defaults Anthropic API version to:

```http
anthropic-version: 2023-06-01
```

For API-key Anthropic request construction, this is inserted if missing. For Vertex/Bedrock-like Anthropic conversions, Sub2API may remove or suppress `anthropic-version` because those upstreams do not accept the same Anthropic API header set.

## Beta Constants

Sub2API defines these Anthropic beta tokens:

```text
oauth-2025-04-20
claude-code-20250219
interleaved-thinking-2025-05-14
fine-grained-tool-streaming-2025-05-14
token-counting-2024-11-01
context-1m-2025-08-07
fast-mode-2026-02-01
prompt-caching-scope-2026-01-05
effort-2025-11-24
redact-thinking-2026-02-12
context-management-2025-06-27
extended-cache-ttl-2025-04-11
```

Named header compositions:

```http
# OAuth default / model sync
anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14

# API-key default / model sync
anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14

# /v1/messages without tools, OAuth
anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14

# /v1/messages with tools, OAuth
anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14

# count_tokens, OAuth
anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14,token-counting-2024-11-01

# Haiku, OAuth
anthropic-beta: oauth-2025-04-20,interleaved-thinking-2025-05-14

# Haiku, API key
anthropic-beta: interleaved-thinking-2025-05-14
```

Full Claude Code mimicry beta order for OAuth non-Haiku requests:

```text
claude-code-20250219
oauth-2025-04-20
interleaved-thinking-2025-05-14
prompt-caching-scope-2026-01-05
effort-2025-11-24
context-management-2025-06-27
extended-cache-ttl-2025-04-11
```

Sub2API intentionally does not default `redact-thinking-2026-02-12`; it is preserved only when the client explicitly asks for it.

## API-Key Model Listing

For Anthropic API-key model listing, Sub2API sends:

```http
Accept: application/json
anthropic-version: 2023-06-01
anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14
x-api-key: <api_key>
```

It also applies the Claude default identity headers listed above.

## OAuth Model Listing

For Anthropic OAuth model listing, Sub2API sends:

```http
Accept: application/json
anthropic-version: 2023-06-01
anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14
Authorization: Bearer <access_token>
```

It also applies the Claude default identity headers listed above.

## Request Header Passthrough Allowlist

Sub2API only forwards selected downstream headers into Anthropic upstream requests. The core allowlist is:

```text
accept
x-stainless-retry-count
x-stainless-timeout
x-stainless-lang
x-stainless-package-version
x-stainless-os
x-stainless-arch
x-stainless-runtime
x-stainless-runtime-version
x-stainless-helper-method
anthropic-dangerous-direct-browser-access
anthropic-version
x-app
anthropic-beta
accept-language
sec-fetch-mode
user-agent
content-type
accept-encoding
x-claude-code-session-id
x-client-request-id
```

For Claude Code mimicry, several of these are overwritten after allowlist copying.

## Cache-Control Default

When Sub2API generates its own Anthropic `cache_control` blocks, the default TTL constant is:

```text
5m
```

The source notes that real Claude Code currently uses `1h`, but Sub2API chooses `5m` unless the client supplied a TTL.
