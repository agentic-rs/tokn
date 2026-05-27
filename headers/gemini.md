# Gemini Headers

Reference source: `Wei-Shaw/sub2api` at `2fb9fb2f715903de0a7328e8e1531beef3f3b2e9`.

This document records the upstream request headers Sub2API synthesizes for Gemini AI Studio, Gemini OAuth, and Vertex Gemini service-account paths.

## Account Types

Sub2API supports Gemini API-key, OAuth, and Vertex service-account style credentials.

API-key Gemini requests authenticate with:

```http
x-goog-api-key: <api_key>
```

OAuth Gemini requests authenticate with:

```http
Authorization: Bearer <access_token>
```

Vertex service-account requests also authenticate with:

```http
Authorization: Bearer <service_account_access_token>
```

## Model Listing

For Gemini model listing, Sub2API starts with:

```http
Accept: application/json
```

Then it adds auth based on account type.

API-key model listing:

```http
Accept: application/json
x-goog-api-key: <api_key>
```

OAuth model listing:

```http
Accept: application/json
Authorization: Bearer <access_token>
```

Sub2API explicitly rejects Gemini Code Assist model listing through this generic model-sync path when a `project_id` is present. Code Assist/GCP-project accounts need their own upstream flow.

## Endpoint URL Normalization

Sub2API normalizes Gemini model-list URLs to:

```text
https://generativelanguage.googleapis.com/v1beta/models
```

Equivalent configured bases such as these resolve to the same model-list URL:

```text
https://generativelanguage.googleapis.com
https://generativelanguage.googleapis.com/v1beta
https://generativelanguage.googleapis.com/v1beta/models
```

## Vertex Service Account Token Request

When generating a Google OAuth token from a Vertex service account, Sub2API sends the token request as form data:

```http
Content-Type: application/x-www-form-urlencoded
```

The resulting access token is later used as:

```http
Authorization: Bearer <access_token>
```

## Request Header Policy

Sub2API does not use broad client-header passthrough for Gemini auth headers. Upstream auth is synthesized from account credentials.

Important safety behavior to mirror:

```text
API-key account: set x-goog-api-key from account credential.
OAuth account: set Authorization from token provider.
Vertex service account: set Authorization from generated access token.
```

Do not forward downstream `Authorization`, `x-api-key`, or `x-goog-api-key` as upstream Gemini credentials unless they are the selected account credential.

## Debug Response Headers

Sub2API has a Gemini-specific response-header debug switch:

```yaml
gateway:
  gemini_debug_response_headers: false
```

It defaults to `false` and is intended only for short troubleshooting windows because of high-frequency request overhead.

This setting affects logging/diagnostics, not upstream request header synthesis.
