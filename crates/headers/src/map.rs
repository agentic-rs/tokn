//! Vec-backed, order- and case-preserving header map.
//!
//! Designed for typical HTTP request/response sizes (< 30 entries), where
//! linear scan beats hash-table overhead. Supports duplicate names (required
//! for headers like `Set-Cookie`) by storing all entries in insertion order.
//!
//! Lookup is case-insensitive via [`HeaderName`]'s `Eq` impl.

use crate::name::HeaderName;
use crate::value::HeaderValue;

/// A list of (name, value) pairs in insertion order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderMap {
  entries: Vec<(HeaderName, HeaderValue)>,
}

impl HeaderMap {
  /// Create an empty map.
  pub fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Create a map with capacity for `n` entries (no allocations until exceeded).
  pub fn with_capacity(n: usize) -> Self {
    Self {
      entries: Vec::with_capacity(n),
    }
  }

  /// Append `(name, value)`. Allows duplicate names.
  pub fn insert(&mut self, name: impl Into<HeaderName>, value: impl Into<HeaderValue>) {
    self.entries.push((name.into(), value.into()));
  }

  /// Replace any existing entries for `name` with a single new entry. The
  /// new entry is appended at the position of the first existing match (if any),
  /// otherwise at the end. All other duplicate matches are removed.
  pub fn replace(&mut self, name: impl Into<HeaderName>, value: impl Into<HeaderValue>) {
    let name = name.into();
    let value = value.into();
    let mut first = None;
    let mut i = 0;
    while i < self.entries.len() {
      if self.entries[i].0 == name {
        if first.is_none() {
          first = Some(i);
          self.entries[i].1 = value.clone();
          i += 1;
        } else {
          self.entries.remove(i);
        }
      } else {
        i += 1;
      }
    }
    if first.is_none() {
      self.entries.push((name, value));
    }
  }

  /// Remove all entries matching `name`. Returns the count removed.
  pub fn remove(&mut self, name: &HeaderName) -> usize {
    let before = self.entries.len();
    self.entries.retain(|(n, _)| n != name);
    before - self.entries.len()
  }

  /// First value matching `name`, if any.
  pub fn get(&self, name: &HeaderName) -> Option<&HeaderValue> {
    self.entries.iter().find(|(n, _)| n == name).map(|(_, v)| v)
  }

  /// All values matching `name`, in insertion order.
  pub fn get_all<'a>(&'a self, name: &'a HeaderName) -> impl Iterator<Item = &'a HeaderValue> + 'a {
    self.entries.iter().filter(move |(n, _)| n == name).map(|(_, v)| v)
  }

  /// Whether at least one entry matches `name`.
  pub fn contains_key(&self, name: &HeaderName) -> bool {
    self.entries.iter().any(|(n, _)| n == name)
  }

  /// Iterate entries in insertion order.
  pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
    self.entries.iter().map(|(n, v)| (n, v))
  }

  /// Number of entries (counts duplicates).
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Whether the map is empty.
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Append all entries from `other` in their original order. Duplicates are
  /// preserved (this does not deduplicate against `self`).
  pub fn extend(&mut self, other: HeaderMap) {
    self.entries.extend(other.entries);
  }

  /// Append all entries from `other`, with later names winning over earlier ones
  /// in `self`. Within `other`, all entries are kept (even duplicate names).
  pub fn merge_replacing(&mut self, other: HeaderMap) {
    for (name, value) in &other.entries {
      self.entries.retain(|(n, _)| n != name);
      let _ = value;
    }
    self.entries.extend(other.entries);
  }
}

impl FromIterator<(HeaderName, HeaderValue)> for HeaderMap {
  fn from_iter<I: IntoIterator<Item = (HeaderName, HeaderValue)>>(iter: I) -> Self {
    Self {
      entries: iter.into_iter().collect(),
    }
  }
}

impl IntoIterator for HeaderMap {
  type Item = (HeaderName, HeaderValue);
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.entries.into_iter()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn name(s: &'static str) -> HeaderName {
    HeaderName::new(s)
  }

  #[test]
  fn insert_preserves_order_and_case() {
    let mut m = HeaderMap::new();
    m.insert(name("X-Aaa"), "1");
    m.insert(name("Authorization"), "Bearer x");
    m.insert(name("Content-Type"), "application/json");
    let collected: Vec<_> = m.iter().map(|(n, v)| (n.original().to_string(), v.as_str().to_string())).collect();
    assert_eq!(
      collected,
      vec![
        ("X-Aaa".into(), "1".into()),
        ("Authorization".into(), "Bearer x".into()),
        ("Content-Type".into(), "application/json".into()),
      ]
    );
  }

  #[test]
  fn get_is_case_insensitive() {
    let mut m = HeaderMap::new();
    m.insert(name("Authorization"), "Bearer x");
    assert_eq!(m.get(&name("authorization")).map(|v| v.as_str()), Some("Bearer x"));
    assert_eq!(m.get(&name("AUTHORIZATION")).map(|v| v.as_str()), Some("Bearer x"));
  }

  #[test]
  fn duplicates_preserved() {
    let mut m = HeaderMap::new();
    m.insert(name("Set-Cookie"), "a=1");
    m.insert(name("Set-Cookie"), "b=2");
    let all: Vec<_> = m.get_all(&name("set-cookie")).map(|v| v.as_str().to_string()).collect();
    assert_eq!(all, vec!["a=1", "b=2"]);
    assert_eq!(m.len(), 2);
  }

  #[test]
  fn replace_removes_duplicates_keeps_position() {
    let mut m = HeaderMap::new();
    m.insert(name("A"), "1");
    m.insert(name("Set-Cookie"), "old1");
    m.insert(name("B"), "2");
    m.insert(name("Set-Cookie"), "old2");
    m.replace(name("set-cookie"), "new");
    let collected: Vec<_> = m.iter().map(|(n, v)| (n.original().to_string(), v.as_str().to_string())).collect();
    assert_eq!(
      collected,
      vec![
        ("A".into(), "1".into()),
        ("Set-Cookie".into(), "new".into()),
        ("B".into(), "2".into()),
      ]
    );
  }

  #[test]
  fn remove_returns_count() {
    let mut m = HeaderMap::new();
    m.insert(name("X"), "1");
    m.insert(name("X"), "2");
    m.insert(name("Y"), "3");
    assert_eq!(m.remove(&name("x")), 2);
    assert_eq!(m.len(), 1);
  }

  #[test]
  fn extend_keeps_duplicates() {
    let mut a = HeaderMap::new();
    a.insert(name("X"), "1");
    let mut b = HeaderMap::new();
    b.insert(name("X"), "2");
    a.extend(b);
    assert_eq!(a.len(), 2);
  }

  #[test]
  fn merge_replacing_overrides_existing() {
    let mut a = HeaderMap::new();
    a.insert(name("X"), "old");
    a.insert(name("Y"), "keep");
    let mut b = HeaderMap::new();
    b.insert(name("x"), "new");
    a.merge_replacing(b);
    let collected: Vec<_> = a.iter().map(|(n, v)| (n.original().to_string(), v.as_str().to_string())).collect();
    assert_eq!(
      collected,
      vec![("Y".into(), "keep".into()), ("x".into(), "new".into())]
    );
  }
}
