# Proxy Switch Rewrite Plan

## Target Contract

- Switch mode resolves a provider/account first, independent of whether the request is a known LLM endpoint.
- Known generation requests carry an `Endpoint` and may use model/endpoint compatibility checks.
- Provider utility and opaque proxy requests carry method/path identity and must not invent a generation endpoint.
- Provider header patching receives a request kind so providers can inject credentials for both generation and non-generation traffic.

## First Breaking Step

- Replace `HeaderPatchCtx.endpoint: Endpoint` with `HeaderPatchCtx.request_kind: ProviderRequestKind`.
- `ProviderRequestKind::Operation(Endpoint)` represents chat/responses/messages traffic.
- `ProviderRequestKind::Models` represents provider model-listing traffic.
- `ProviderRequestKind::Opaque` represents arbitrary provider traffic.
- Existing generation callers must construct `Operation(endpoint)`.
- Proxy switch custom-path callers must construct `Models` for model paths and `Opaque` otherwise.

## Deferred Steps

- Split request resolution into operation routes and provider-traffic routes.
- Move `MissingResolvedEndpoint` checks into operation-only routing.
- Reuse provider-utility routing for API `/v1/models` and proxy `/models`.
