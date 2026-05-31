# rs-rate-limiter

[![CI](https://github.com/philiprehberger/rs-rate-limiter/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-rate-limiter/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-rate-limiter.svg)](https://crates.io/crates/philiprehberger-rate-limiter)
[![Last updated](https://img.shields.io/github/last-commit/philiprehberger/rs-rate-limiter)](https://github.com/philiprehberger/rs-rate-limiter/commits/main)

![rs-rate-limiter](https://raw.githubusercontent.com/philiprehberger/rs-rate-limiter/main/package-card.webp)

Token bucket, sliding window, and fixed window rate limiting

## Installation

```toml
[dependencies]
philiprehberger-rate-limiter = "0.3.0"
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

### Leaky bucket

```rust
use philiprehberger_rate_limiter::{RateLimiter, LeakyBucket, Decision};

let limiter = LeakyBucket::new(5, 1.0); // capacity 5, leaks 1/sec
match limiter.check("user-1") {
    Decision::Allowed => println!("ok"),
    Decision::Denied { retry_after } => println!("wait {:?}", retry_after),
}
```

### Non-consuming `peek`

```rust
use philiprehberger_rate_limiter::{RateLimiter, TokenBucket, Decision};

let limiter = TokenBucket::new(2, 1.0);
limiter.check("user-1"); // consume

// peek returns what check would return — without mutating state.
match limiter.peek("user-1") {
    Decision::Allowed => {},
    Decision::Denied { retry_after } => {
        eprintln!("would deny: retry in {:?}", retry_after);
    }
}
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
| `LeakyBucket::new(capacity, leak_rate)` | Leaky bucket limiter (drains units/sec) |
| `LeakyBucket::check(key)` | Adds one unit; denies if level would exceed capacity |
| `.stats()` | Returns `RateLimiterStats` for the limiter |
| `.reset_key(key)` | Clears state for a key, returns `true` if it existed |
| `.cleanup_inactive(max_age)` | Removes stale keys, returns count removed |
| `.peek(key)` | Non-consuming check; available on all four limiters |

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Support

If you find this project useful:

⭐ [Star the repo](https://github.com/philiprehberger/rs-rate-limiter)

🐛 [Report issues](https://github.com/philiprehberger/rs-rate-limiter/issues?q=is%3Aissue+is%3Aopen+label%3Abug)

💡 [Suggest features](https://github.com/philiprehberger/rs-rate-limiter/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)

❤️ [Sponsor development](https://github.com/sponsors/philiprehberger)

🌐 [All Open Source Projects](https://philiprehberger.com/open-source-packages)

💻 [GitHub Profile](https://github.com/philiprehberger)

🔗 [LinkedIn Profile](https://www.linkedin.com/in/philiprehberger)

## License

[MIT](LICENSE)
