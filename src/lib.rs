//! Token bucket, sliding window, and fixed window rate limiting.
//!
//! This crate provides three rate limiting strategies behind a common
//! [`RateLimiter`] trait. All implementations are thread-safe (`Send + Sync`)
//! with zero external dependencies.
//!
//! # Examples
//!
//! ```
//! use philiprehberger_rate_limiter::{RateLimiter, TokenBucket, Decision};
//!
//! let limiter = TokenBucket::new(5, 1.0);
//!
//! match limiter.check("user-1") {
//!     Decision::Allowed => println!("ok"),
//!     Decision::Denied { retry_after } => {
//!         println!("wait {:?}", retry_after);
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The result of a rate limit check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The request is allowed.
    Allowed,
    /// The request is denied. `retry_after` indicates how long to wait before
    /// a request would be allowed.
    Denied {
        /// Duration until a request would be allowed.
        retry_after: Duration,
    },
}

/// A rate limiter that checks whether a keyed request should be allowed.
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given key.
    ///
    /// If allowed, the limiter consumes one unit of capacity. If denied, the
    /// returned [`Decision::Denied`] contains the estimated wait time.
    fn check(&self, key: &str) -> Decision;
}

// ---------------------------------------------------------------------------
// TokenBucket
// ---------------------------------------------------------------------------

struct BucketState {
    tokens: f64,
    last_refill: Instant,
}

/// A token bucket rate limiter.
///
/// Each key gets its own bucket with a fixed `capacity`. Tokens are refilled
/// continuously at `refill_rate` tokens per second. A request consumes one
/// token.
///
/// # Examples
///
/// ```
/// use philiprehberger_rate_limiter::{RateLimiter, TokenBucket, Decision};
///
/// let limiter = TokenBucket::new(10, 2.0);
/// assert!(matches!(limiter.check("k"), Decision::Allowed));
/// ```
pub struct TokenBucket {
    capacity: u32,
    refill_rate: f64,
    state: Mutex<HashMap<String, BucketState>>,
}

impl TokenBucket {
    /// Creates a new `TokenBucket`.
    ///
    /// * `capacity` - Maximum number of tokens per key.
    /// * `refill_rate` - Tokens added per second (continuously).
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimiter for TokenBucket {
    fn check(&self, key: &str) -> Decision {
        let mut map = self.state.lock().unwrap();
        let now = Instant::now();
        let cap = f64::from(self.capacity);

        let entry = map.entry(key.to_owned()).or_insert(BucketState {
            tokens: cap,
            last_refill: now,
        });

        // Refill tokens based on elapsed time.
        let elapsed = now.duration_since(entry.last_refill).as_secs_f64();
        entry.tokens = (entry.tokens + elapsed * self.refill_rate).min(cap);
        entry.last_refill = now;

        if entry.tokens >= 1.0 {
            entry.tokens -= 1.0;
            Decision::Allowed
        } else {
            let deficit = 1.0 - entry.tokens;
            let wait_secs = deficit / self.refill_rate;
            Decision::Denied {
                retry_after: Duration::from_secs_f64(wait_secs),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SlidingWindow
// ---------------------------------------------------------------------------

struct SlidingWindowState {
    timestamps: Vec<Instant>,
}

/// A sliding window rate limiter.
///
/// Tracks individual request timestamps per key and removes expired entries on
/// each check. A request is allowed if the number of requests within the
/// window is below `max_requests`.
///
/// # Examples
///
/// ```
/// use philiprehberger_rate_limiter::{RateLimiter, SlidingWindow, Decision};
/// use std::time::Duration;
///
/// let limiter = SlidingWindow::new(Duration::from_secs(60), 100);
/// assert!(matches!(limiter.check("k"), Decision::Allowed));
/// ```
pub struct SlidingWindow {
    window: Duration,
    max_requests: u32,
    state: Mutex<HashMap<String, SlidingWindowState>>,
}

impl SlidingWindow {
    /// Creates a new `SlidingWindow`.
    ///
    /// * `window` - The sliding window duration.
    /// * `max_requests` - Maximum allowed requests within the window.
    pub fn new(window: Duration, max_requests: u32) -> Self {
        Self {
            window,
            max_requests,
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimiter for SlidingWindow {
    fn check(&self, key: &str) -> Decision {
        let mut map = self.state.lock().unwrap();
        let now = Instant::now();
        let cutoff = now.checked_sub(self.window).unwrap_or(now);

        let entry = map.entry(key.to_owned()).or_insert(SlidingWindowState {
            timestamps: Vec::new(),
        });

        // Remove expired timestamps.
        entry.timestamps.retain(|t| *t > cutoff);

        if (entry.timestamps.len() as u32) < self.max_requests {
            entry.timestamps.push(now);
            Decision::Allowed
        } else {
            // The oldest timestamp in the window determines when a slot opens.
            let oldest = entry.timestamps[0];
            let retry_after = self.window.saturating_sub(now.duration_since(oldest));
            Decision::Denied { retry_after }
        }
    }
}

// ---------------------------------------------------------------------------
// FixedWindow
// ---------------------------------------------------------------------------

struct FixedWindowState {
    count: u32,
    window_start: Instant,
}

/// A fixed window rate limiter.
///
/// Divides time into fixed-size windows. Each key gets a counter that resets
/// when the window expires.
///
/// # Examples
///
/// ```
/// use philiprehberger_rate_limiter::{RateLimiter, FixedWindow, Decision};
/// use std::time::Duration;
///
/// let limiter = FixedWindow::new(Duration::from_secs(60), 100);
/// assert!(matches!(limiter.check("k"), Decision::Allowed));
/// ```
pub struct FixedWindow {
    window: Duration,
    max_requests: u32,
    state: Mutex<HashMap<String, FixedWindowState>>,
}

impl FixedWindow {
    /// Creates a new `FixedWindow`.
    ///
    /// * `window` - The fixed window duration.
    /// * `max_requests` - Maximum allowed requests within each window.
    pub fn new(window: Duration, max_requests: u32) -> Self {
        Self {
            window,
            max_requests,
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimiter for FixedWindow {
    fn check(&self, key: &str) -> Decision {
        let mut map = self.state.lock().unwrap();
        let now = Instant::now();

        let entry = map.entry(key.to_owned()).or_insert(FixedWindowState {
            count: 0,
            window_start: now,
        });

        // Reset counter if the window has expired.
        if now.duration_since(entry.window_start) >= self.window {
            entry.count = 0;
            entry.window_start = now;
        }

        if entry.count < self.max_requests {
            entry.count += 1;
            Decision::Allowed
        } else {
            let elapsed = now.duration_since(entry.window_start);
            let retry_after = self.window.saturating_sub(elapsed);
            Decision::Denied { retry_after }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // --- TokenBucket tests ---

    #[test]
    fn token_bucket_allows_up_to_capacity() {
        let limiter = TokenBucket::new(3, 1.0);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
    }

    #[test]
    fn token_bucket_denies_when_exhausted() {
        let limiter = TokenBucket::new(2, 1.0);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        let result = limiter.check("a");
        assert!(matches!(result, Decision::Denied { .. }));
    }

    #[test]
    fn token_bucket_refills_after_time() {
        let limiter = TokenBucket::new(1, 10.0); // 10 tokens/sec
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert!(matches!(limiter.check("a"), Decision::Denied { .. }));

        // Wait enough for at least 1 token to refill.
        thread::sleep(Duration::from_millis(150));
        assert_eq!(limiter.check("a"), Decision::Allowed);
    }

    #[test]
    fn token_bucket_denied_has_reasonable_retry_after() {
        let limiter = TokenBucket::new(1, 10.0);
        limiter.check("a"); // consume the token
        if let Decision::Denied { retry_after } = limiter.check("a") {
            // Should be around 100ms (1 token / 10 tokens-per-sec)
            assert!(retry_after <= Duration::from_millis(200));
            assert!(retry_after >= Duration::from_millis(1));
        } else {
            panic!("expected Denied");
        }
    }

    // --- SlidingWindow tests ---

    #[test]
    fn sliding_window_allows_within_limit() {
        let limiter = SlidingWindow::new(Duration::from_secs(10), 3);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
    }

    #[test]
    fn sliding_window_denies_over_limit() {
        let limiter = SlidingWindow::new(Duration::from_secs(10), 2);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        let result = limiter.check("a");
        assert!(matches!(result, Decision::Denied { .. }));
    }

    #[test]
    fn sliding_window_allows_after_window_passes() {
        let limiter = SlidingWindow::new(Duration::from_millis(100), 1);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert!(matches!(limiter.check("a"), Decision::Denied { .. }));

        thread::sleep(Duration::from_millis(150));
        assert_eq!(limiter.check("a"), Decision::Allowed);
    }

    // --- FixedWindow tests ---

    #[test]
    fn fixed_window_allows_within_limit() {
        let limiter = FixedWindow::new(Duration::from_secs(10), 3);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
    }

    #[test]
    fn fixed_window_denies_over_limit() {
        let limiter = FixedWindow::new(Duration::from_secs(10), 2);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        let result = limiter.check("a");
        assert!(matches!(result, Decision::Denied { .. }));
    }

    #[test]
    fn fixed_window_resets_on_new_window() {
        let limiter = FixedWindow::new(Duration::from_millis(100), 1);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert!(matches!(limiter.check("a"), Decision::Denied { .. }));

        thread::sleep(Duration::from_millis(150));
        assert_eq!(limiter.check("a"), Decision::Allowed);
    }

    // --- Cross-cutting tests ---

    #[test]
    fn multiple_keys_are_independent() {
        let limiter = TokenBucket::new(1, 0.1);
        assert_eq!(limiter.check("a"), Decision::Allowed);
        assert!(matches!(limiter.check("a"), Decision::Denied { .. }));

        // Different key should still be allowed.
        assert_eq!(limiter.check("b"), Decision::Allowed);
    }

    #[test]
    fn sliding_window_multiple_keys_independent() {
        let limiter = SlidingWindow::new(Duration::from_secs(10), 1);
        assert_eq!(limiter.check("x"), Decision::Allowed);
        assert!(matches!(limiter.check("x"), Decision::Denied { .. }));
        assert_eq!(limiter.check("y"), Decision::Allowed);
    }

    #[test]
    fn fixed_window_multiple_keys_independent() {
        let limiter = FixedWindow::new(Duration::from_secs(10), 1);
        assert_eq!(limiter.check("x"), Decision::Allowed);
        assert!(matches!(limiter.check("x"), Decision::Denied { .. }));
        assert_eq!(limiter.check("y"), Decision::Allowed);
    }

    #[test]
    fn fixed_window_denied_has_retry_after() {
        let limiter = FixedWindow::new(Duration::from_secs(10), 1);
        limiter.check("a");
        if let Decision::Denied { retry_after } = limiter.check("a") {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(10));
        } else {
            panic!("expected Denied");
        }
    }

    #[test]
    fn sliding_window_denied_has_retry_after() {
        let limiter = SlidingWindow::new(Duration::from_secs(10), 1);
        limiter.check("a");
        if let Decision::Denied { retry_after } = limiter.check("a") {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(10));
        } else {
            panic!("expected Denied");
        }
    }
}
