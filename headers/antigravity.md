# Antigravity Headers

Reference source: `Wei-Shaw/sub2api` at `2fb9fb2f715903de0a7328e8e1531beef3f3b2e9`.

This document records the upstream request headers Sub2API synthesizes for Antigravity API-key and OAuth paths.

## Account Types

Sub2API has two Antigravity paths:

```text
API-key Antigravity-compatible gateway
OAuth Antigravity / Cloud Code upstream
```

API-key Antigravity-compatible requests use Anthropic-style headers:

```http
x-api-key: <api_key>
anthropic-version: 2023-06-01
anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14
Accept: application/json
```

They also apply the Claude default identity headers:

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

OAuth Antigravity obtains an access token through its token provider and sends it to the Antigravity client/upstream as a bearer token. In the model-listing path, this token is passed to `FetchAvailableModels` with the configured `project_id`.

## API-Key Model Listing

Sub2API only supports API-key Antigravity model sync when the configured base URL is an Antigravity-compatible gateway ending in `/antigravity`.

Required base URL rule:

```text
<base_url> must end with /antigravity
```

If this is not true, Sub2API rejects model sync and tells the user to use Antigravity OAuth for official Cloud Code upstreams.

API-key model-listing headers:

```http
Accept: application/json
anthropic-version: 2023-06-01
anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14
x-api-key: <api_key>
```

Plus the Claude default identity headers listed above.

## OAuth Model Listing

For OAuth Antigravity model listing, Sub2API requires:

```text
antigravityGatewayService != nil
antigravityGatewayService.GetTokenProvider() != nil
access_token from token provider is non-empty
```

It then calls the Antigravity client with:

```text
access_token
project_id
proxy_url, when configured
```

The exact HTTP header construction is encapsulated by the Antigravity client, but the credential semantics are bearer-token OAuth, not `x-api-key`.

## User-Agent Setting

Sub2API exposes an Antigravity upstream User-Agent version setting in its settings view:

```text
AntigravityUserAgentVersion
```

An empty value means use the configured or built-in default. This is separate from the API-key compatible gateway path, which uses Claude default identity headers.

## Credential Separation

Mirror this separation:

```text
API-key compatible gateway: x-api-key + Anthropic-style version/beta/Claude identity headers.
OAuth official upstream: bearer access token via Antigravity client, with project_id when available.
```

Do not send an API key to the OAuth Antigravity path, and do not use OAuth bearer semantics for the `/antigravity` API-key gateway compatibility path.
