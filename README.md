# rs-rate-limiter

[![CI](https://github.com/philiprehberger/rs-rate-limiter/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-rate-limiter/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-rate-limiter.svg)](https://crates.io/crates/philiprehberger-rate-limiter)
[![License](https://img.shields.io/github/license/philiprehberger/rs-rate-limiter)](LICENSE)

Token bucket, sliding window, and fixed window rate limiting.

## Installation

```toml
[dependencies]
philiprehberger-rate-limiter = "0.2.0"
```

## Usage

```rust
use philiprehberger_rate_limiter::{RateLimiter, TokenBucket, Decision};

let limiter = TokenBucket::new(10, 2.0); // 10 tokens, refill 2/sec

match limiter.check("user-123") {
    Decision::Allowed => println!("Request allowed"),
    Decision::Denied { retry_after } => {
        println!("Rate limited, retry after {:?}", retry_after);
    }
}
```

### Sliding window

```rust
use philiprehberger_rate_limiter::{RateLimiter, SlidingWindow};
use std::time::Duration;

let limiter = SlidingWindow::new(Duration::from_secs(60), 100);
let decision = limiter.check("client-ip");
```

### Fixed window

```rust
use philiprehberger_rate_limiter::{RateLimiter, FixedWindow};
use std::time::Duration;

let limiter = FixedWindow::new(Duration::from_secs(60), 100);
let decision = limiter.check("api-key");
```

### Stats

All limiter types expose a `stats()` method for observability:

```rust
use philiprehberger_rate_limiter::TokenBucket;

let limiter = TokenBucket::new(10, 2.0);
limiter.check("user-1");
limiter.check("user-2");

let stats = limiter.stats();
println!("Total: {}, Allowed: {}, Denied: {}, Active keys: {}",
    stats.total_requests, stats.allowed, stats.denied, stats.active_keys);
```

### Key management

Reset a specific key or clean up stale entries:

```rust
use philiprehberger_rate_limiter::TokenBucket;
use std::time::Duration;

let limiter = TokenBucket::new(10, 2.0);
limiter.check("user-1");

// Clear state for a single key
limiter.reset_key("user-1");

// Remove keys not accessed in the last 10 minutes
let removed = limiter.cleanup_inactive(Duration::from_secs(600));
```

## API

| Type / Trait | Description |
|--------------|-------------|
| `RateLimiter` | Trait with `fn check(&self, key: &str) -> Decision` |
| `Decision` | `Allowed` or `Denied { retry_after: Duration }` |
| `RateLimiterStats` | Stats struct: `total_requests`, `allowed`, `denied`, `active_keys` |
| `TokenBucket::new(capacity, refill_rate)` | Token bucket limiter (tokens/sec) |
| `SlidingWindow::new(window, max_requests)` | Sliding window counter |
| `FixedWindow::new(window, max_requests)` | Fixed window counter |
| `.stats()` | Returns `RateLimiterStats` for the limiter |
| `.reset_key(key)` | Clears state for a key, returns `true` if it existed |
| `.cleanup_inactive(max_age)` | Removes stale keys, returns count removed |


## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## License

MIT
