//! Session → account affinity map used by [`super::AccountPool`] to keep
//! multi-turn conversations (tool-call follow-ups, OpenAI Responses
//! `previous_response_id`, Anthropic extended-thinking continuations) on the
//! same upstream credential.
//!
//! Lookup semantics are tri-state via [`Lookup`]:
//! - `Hit(account_id)` — session known and still within TTL.
//! - `Expired` — session was known but eviction has elapsed; surfaces to the
//!   client as HTTP 410 so they replay rather than silently switching account.
//! - `Unknown` — first-use; the dispatcher allocates an account and records.
//!
//! Tombstones distinguish `Expired` from `Unknown`. They live for
//! `tombstone_ttl` (≥ `session_ttl`); after that the entry is fully forgotten
//! and a future request with the same id is treated as a brand new session.
//!
//! In-memory only — by design. Cross-restart affinity would need durable
//! state, which we explicitly chose not to keep here.
//!
//! Concurrency: a single `RwLock<HashMap<…>>` covers both lookup and write
//! paths. Write rate is one per request (record on success / on retry); read
//! rate is the same. No contention concern at expected throughput.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum Lookup {
  Hit(String),
  Expired,
  Unknown,
}

#[derive(Debug)]
struct Entry<I> {
  /// Empty string ⇒ tombstone (the session was evicted).
  account_id: String,
  /// For live entries: when this binding was last touched.
  /// For tombstones: when the entry was tombstoned.
  stamped_at: I,
}

#[doc(hidden)]
pub trait Clock {
  type Instant: Copy;
  type Duration: Copy + Ord;

  fn now(&self) -> Self::Instant;
  fn elapsed(&self, stamped_at: Self::Instant) -> Self::Duration;
}

#[doc(hidden)]
pub struct SystemClock;

impl Clock for SystemClock {
  type Instant = Instant;
  type Duration = Duration;

  fn now(&self) -> Instant {
    Instant::now()
  }

  fn elapsed(&self, stamped_at: Instant) -> Duration {
    stamped_at.elapsed()
  }
}

pub struct Affinity<C: Clock = SystemClock> {
  map: RwLock<HashMap<String, Entry<C::Instant>>>,
  ttl: C::Duration,
  tombstone_ttl: C::Duration,
  clock: C,
}

impl Affinity<SystemClock> {
  pub fn new(ttl: Duration, tombstone_ttl: Duration) -> Self {
    Affinity::<SystemClock>::with_clock(ttl, tombstone_ttl, SystemClock)
  }

  #[cfg(test)]
  pub(crate) fn rewind_live_entry(&self, key: &str, delta: Duration) -> bool {
    let mut g = self.map.write();
    let Some(entry) = g.get_mut(key) else {
      return false;
    };
    if entry.account_id.is_empty() {
      return false;
    }
    let Some(stamped_at) = entry.stamped_at.checked_sub(delta) else {
      return false;
    };
    entry.stamped_at = stamped_at;
    true
  }
}

impl<C: Clock> Affinity<C> {
  fn with_clock(ttl: C::Duration, tombstone_ttl: C::Duration, clock: C) -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
      ttl,
      tombstone_ttl,
      clock,
    }
  }

  /// Look up `key`. Side-effect: stale live entries are converted to
  /// tombstones; expired tombstones are removed.
  pub fn lookup(&self, key: &str) -> Lookup {
    // Fast path: read-only check.
    {
      let g = self.map.read();
      if let Some(e) = g.get(key) {
        let age = self.clock.elapsed(e.stamped_at);
        if e.account_id.is_empty() {
          // Tombstone.
          if age < self.tombstone_ttl {
            return Lookup::Expired;
          }
        } else if age < self.ttl {
          return Lookup::Hit(e.account_id.clone());
        }
      } else {
        return Lookup::Unknown;
      }
    }
    // Slow path: state transition needed.
    let mut g = self.map.write();
    let now = self.clock.now();
    match g.get(key) {
      Some(e) if e.account_id.is_empty() => {
        if self.clock.elapsed(e.stamped_at) < self.tombstone_ttl {
          Lookup::Expired
        } else {
          g.remove(key);
          Lookup::Unknown
        }
      }
      Some(e) => {
        if self.clock.elapsed(e.stamped_at) < self.ttl {
          Lookup::Hit(e.account_id.clone())
        } else {
          // Convert to tombstone.
          g.insert(
            key.to_string(),
            Entry {
              account_id: String::new(),
              stamped_at: now,
            },
          );
          Lookup::Expired
        }
      }
      None => Lookup::Unknown,
    }
  }

  /// Bind `key` to `account_id` (sliding-window refresh on repeat calls).
  /// Clears any tombstone for `key`.
  pub fn record(&self, key: &str, account_id: &str) {
    let mut g = self.map.write();
    g.insert(
      key.to_string(),
      Entry {
        account_id: account_id.to_string(),
        stamped_at: self.clock.now(),
      },
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cell::Cell;

  fn ms(value: u64) -> u64 {
    value
  }

  struct FakeClock {
    now: Cell<u64>,
  }

  impl FakeClock {
    fn new() -> Self {
      Self { now: Cell::new(0) }
    }

    fn advance(&self, delta: u64) {
      self.now.set(self.now.get() + delta);
    }
  }

  impl Clock for FakeClock {
    type Instant = u64;
    type Duration = u64;

    fn now(&self) -> u64 {
      self.now.get()
    }

    fn elapsed(&self, stamped_at: u64) -> u64 {
      self.now() - stamped_at
    }
  }

  fn test_affinity(ttl: u64, tombstone_ttl: u64) -> Affinity<FakeClock> {
    Affinity::<FakeClock>::with_clock(ttl, tombstone_ttl, FakeClock::new())
  }

  #[test]
  fn unknown_then_hit_then_expired() {
    let a = test_affinity(ms(100), ms(300));
    assert_eq!(a.lookup("k1"), Lookup::Unknown);
    a.record("k1", "acct-a");
    assert_eq!(a.lookup("k1"), Lookup::Hit("acct-a".into()));
    a.clock.advance(ms(140));
    assert_eq!(a.lookup("k1"), Lookup::Expired);
  }

  #[test]
  fn record_clears_tombstone() {
    let a = test_affinity(ms(80), ms(400));
    a.record("k", "old");
    a.clock.advance(ms(120));
    assert_eq!(a.lookup("k"), Lookup::Expired);
    a.record("k", "new");
    assert_eq!(a.lookup("k"), Lookup::Hit("new".into()));
  }

  #[test]
  fn tombstone_eventually_forgotten() {
    let a = test_affinity(ms(40), ms(120));
    a.record("k", "x");
    a.clock.advance(ms(70)); // > ttl
    assert_eq!(a.lookup("k"), Lookup::Expired); // tombstoned
    a.clock.advance(ms(150)); // > tombstone_ttl
    assert_eq!(a.lookup("k"), Lookup::Unknown);
  }

  #[test]
  fn sliding_window_keeps_session_alive() {
    let a = test_affinity(ms(150), ms(400));
    a.record("k", "acct");
    for _ in 0..4 {
      a.clock.advance(ms(140));
      assert_eq!(a.lookup("k"), Lookup::Hit("acct".into()));
      a.record("k", "acct"); // refresh
    }
    a.clock.advance(ms(149));
    assert_eq!(a.lookup("k"), Lookup::Hit("acct".into()));
    a.clock.advance(ms(2));
    assert_eq!(a.lookup("k"), Lookup::Expired);
  }
}
