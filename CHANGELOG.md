# Changelog
n## 0.1.6 (2026-03-16)

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
