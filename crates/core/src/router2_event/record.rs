//! Wire-truth records captured from the actual outbound HTTP call.
//!
//! Distinct from the upstream-shaped *intent* values carried on stage
//! summaries (`ConvertedRequestSummary`, `SentSummary`):
//!
//! - intent values describe what router2 *prepared*;
//! - records describe what reqwest *actually sent and received* — after
//!   `Provider::patch_headers` auth injection, `Host`/`Content-Length`
//!   stripping in [`crate::util::http::send`], and reqwest's transparent
//!   decompression on the response side.
//!
//! Persistence uses records to populate `outbound_req_*` and
//! `outbound_resp_*` columns with wire-accurate values; intent values
//! still flow through the per-stage events for diagnostics (and for
//! dry-run profiles whose Send stage is a no-op).
//!
//! Records ride a dedicated [`Router2EventPayload::Record`] variant
//! (peer of `Stage` and `Custom`) rather than nesting inside
//! `StageEvent`, so subscribers that only care about lifecycle/error
//! observation don't pay a match-arm tax for wire-truth captures, and
//! vice versa.
//!
//! [`Router2EventPayload::Record`]: super::Router2EventPayload::Record

use bytes::Bytes;
use llm_headers::HeaderMap;
use smol_str::SmolStr;

/// Wire-truth capture from the actual outbound HTTP call (via
/// [`OutboundCapture`](crate::provider::OutboundCapture)).
///
/// The three variants split a single capture so subscribers can write
/// each piece as soon as it's known: request side before the response
/// arrives, response status+headers as soon as they come back, body
/// only once it's been drained (and never for streaming responses).
#[derive(Debug, Clone)]
pub enum RecordEvent {
  /// Outbound request as it left reqwest. Headers reflect post-strip,
  /// post-patch state; `body` is the exact bytes handed to reqwest.
  UpstreamReq {
    method: SmolStr,
    url: SmolStr,
    headers: HeaderMap,
    body: Bytes,
  },
  /// Upstream response status + headers, captured as soon as they
  /// arrive. Headers reflect reqwest's post-decompression view (the
  /// `Content-Encoding` and `Content-Length` headers are stripped when
  /// reqwest decompresses).
  UpstreamResp { status: u16, headers: HeaderMap },
  /// Materialized upstream response body. Emitted only for buffered
  /// responses; streaming responses skip this variant (the live SSE
  /// byte stream is single-shot and can't be cheaply tee'd).
  UpstreamBody { body: Bytes },
}
