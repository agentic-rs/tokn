//! The `HeaderSchema` trait + helper functions used by per-(persona, overlay)
//! schema structs to round-trip between their typed Rust form and the
//! generic [`HeaderMap`].
//!
//! # Tiers and build modes
//!
//! Every field on a schema struct belongs to exactly one [`Tier`]:
//!
//! * [`Tier::Required`] — must be present after `build`. Missing without a
//!   default yields [`Error::MissingHeader`]. Filled in both loose (`build`)
//!   and strict (`build_strict`) modes.
//! * [`Tier::Standard`] — optional. In loose mode skipped when not provided by
//!   inbound; in strict mode filled from persona defaults when absent.
//! * [`Tier::Extra`] — optional. In loose mode best-effort (use inbound or
//!   persona default if available); in strict mode skipped entirely when not
//!   provided by inbound.
//!
//! Two named constructors live on each schema: `build` (loose) and
//! `build_strict`. A separate proxy-layer "passthrough" path lives outside
//! this crate and is not modelled here.
//!
//! Each schema struct hand-implements [`HeaderSchema`] in Phase 1; a
//! proc-macro derive may follow in a later phase.

use crate::error::Error;
use crate::map::HeaderMap;
use crate::name::HeaderName;
use crate::value::HeaderValue;
use smol_str::SmolStr;

/// Per-field semantic classification used by tier-aware builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
  /// Must be present after `build` / `build_strict`. Missing without a
  /// default yields [`Error::MissingHeader`].
  Required,
  /// Optional. Loose: skipped if absent in inbound. Strict: synthesized from
  /// persona defaults when absent.
  Standard,
  /// Optional. Loose: best-effort (inbound → default → omit). Strict:
  /// skipped if absent in inbound.
  Extra,
}

/// A typed view over a subset of HTTP headers belonging to a particular
/// persona or provider overlay.
pub trait HeaderSchema: Sized {
  /// Build the typed struct from a [`HeaderMap`]. Returns [`Error::MissingHeader`]
  /// when a required header is absent and [`Error::InvalidValue`] when a value
  /// fails domain-specific validation.
  fn parse(map: &HeaderMap) -> Result<Self, Error>;

  /// Render the typed struct back into a [`HeaderMap`]. Inverse of `parse`;
  /// optional fields that are `None` are omitted from the output.
  ///
  /// `dump` only round-trips an already-populated schema; use `build` or
  /// `build_strict` (on the concrete impl) to construct a populated schema
  /// from [`crate::TemplateVars`] and inbound headers.
  fn dump(&self) -> HeaderMap;

  /// Tier classification for every header this schema may emit. Replaces the
  /// older `known_names` API: callers can derive the name list via
  /// [`HeaderSchema::known_names`].
  fn field_tiers() -> &'static [(&'static HeaderName, Tier)];

  /// All header names this schema may emit. Default implementation derives
  /// from [`HeaderSchema::field_tiers`] and is suitable for golden-test
  /// allowlists.
  fn known_names() -> Vec<&'static HeaderName> {
    Self::field_tiers().iter().map(|(n, _)| *n).collect()
  }
}

/// Read a required header value as a [`SmolStr`].
pub fn required(map: &HeaderMap, name: &HeaderName) -> Result<SmolStr, Error> {
  map
    .get(name)
    .map(|v| SmolStr::new(v.as_str()))
    .ok_or_else(|| Error::MissingHeader {
      name: SmolStr::new(name.as_str()),
    })
}

/// Read an optional header value as `Option<SmolStr>`.
pub fn optional(map: &HeaderMap, name: &HeaderName) -> Option<SmolStr> {
  map.get(name).map(|v| SmolStr::new(v.as_str()))
}

/// Insert a `SmolStr` value into the map under `name`.
pub fn put(map: &mut HeaderMap, name: &HeaderName, value: &SmolStr) {
  map.insert(name.clone(), HeaderValue::from_string(value.to_string()));
}

/// Insert a `SmolStr` value into the map under `name` only if `Some`.
pub fn put_opt(map: &mut HeaderMap, name: &HeaderName, value: &Option<SmolStr>) {
  if let Some(v) = value {
    put(map, name, v);
  }
}

// ---------- Tier-aware build helpers ----------

/// Required field, infallible default. Inbound wins; otherwise compute the
/// supplied default. Always returns `Some` to satisfy a required slot.
pub fn req_inbound_or<F: FnOnce() -> SmolStr>(inbound: &HeaderMap, key: &HeaderName, default: F) -> SmolStr {
  inbound
    .get(key)
    .map(|v| SmolStr::from(v.as_str()))
    .unwrap_or_else(default)
}

/// Required field, fallible default. Inbound wins; otherwise consult `default`
/// for an optional fallback. Returns `Err(MissingHeader)` when both yield
/// nothing.
pub fn req_inbound_or_try<F: FnOnce() -> Option<SmolStr>>(
  inbound: &HeaderMap,
  key: &HeaderName,
  default: F,
) -> Result<SmolStr, Error> {
  if let Some(v) = inbound.get(key) {
    return Ok(SmolStr::from(v.as_str()));
  }
  default().ok_or_else(|| Error::MissingHeader {
    name: SmolStr::new(key.as_str()),
  })
}

/// Standard field in loose mode: inbound only, otherwise `None`.
pub fn std_loose(inbound: &HeaderMap, key: &HeaderName) -> Option<SmolStr> {
  inbound.get(key).map(|v| SmolStr::from(v.as_str()))
}

/// Standard field in strict mode: inbound, otherwise synthesised from
/// `default`. Always returns `Some`.
pub fn std_strict<F: FnOnce() -> SmolStr>(inbound: &HeaderMap, key: &HeaderName, default: F) -> Option<SmolStr> {
  Some(req_inbound_or(inbound, key, default))
}

/// Extra field in loose mode: best-effort (inbound, otherwise consult
/// `default`).
pub fn extra_loose<F: FnOnce() -> Option<SmolStr>>(
  inbound: &HeaderMap,
  key: &HeaderName,
  default: F,
) -> Option<SmolStr> {
  inbound.get(key).map(|v| SmolStr::from(v.as_str())).or_else(default)
}

/// Extra field in strict mode: inbound only, otherwise `None` (skipped on
/// dump).
pub fn extra_strict(inbound: &HeaderMap, key: &HeaderName) -> Option<SmolStr> {
  inbound.get(key).map(|v| SmolStr::from(v.as_str()))
}

// ---------- Legacy build helpers (retained for transitional call sites) ----------

/// Read a header value from `inbound` if present, otherwise compute the
/// supplied default.
///
/// Prefer [`req_inbound_or`] / [`std_strict`] / [`extra_loose`] in tier-aware
/// builders; this remains for legacy `build` constructors that haven't been
/// migrated to the tier helpers yet.
pub fn from_inbound_or<F: FnOnce() -> SmolStr>(inbound: &HeaderMap, key: &HeaderName, default: F) -> SmolStr {
  req_inbound_or(inbound, key, default)
}

/// Read an optional header value from `inbound`. Returns `None` if absent.
///
/// Sister to [`from_inbound_or`] for legacy call sites; tier-aware builders
/// should prefer [`std_loose`] / [`extra_strict`].
pub fn opt_from_inbound(inbound: &HeaderMap, key: &HeaderName) -> Option<SmolStr> {
  inbound.get(key).map(|v| SmolStr::from(v.as_str()))
}
