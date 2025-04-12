# Changelog

## 0.2.0 (2026-03-17)

- Add `RateLimiterStats` struct for observability (`stats()` method on all limiters)
- Add `reset_key(key)` to clear rate limit state for a specific key
- Add `cleanup_inactive(max_age)` to remove stale keys not accessed within a duration
- Track `last_accessed` timestamp per key for cleanup support

## 0.1.7

- Add readme, rust-version, documentation to Cargo.toml
- Add Development section to README
## 0.1.6 (2026-03-16)

- Update install snippet to use full version

## 0.1.5 (2026-03-16)

- Add README badges
- Synchronize version across Cargo.toml, README, and CHANGELOG

## 0.1.0 (2026-03-15)

- Initial release
- `TokenBucket` rate limiter with continuous token refill
- `SlidingWindow` rate limiter with per-key timestamp tracking
- `FixedWindow` rate limiter with automatic window reset
- `RateLimiter` trait for polymorphic usage
- Thread-safe (`Send + Sync`) with zero external dependencies
