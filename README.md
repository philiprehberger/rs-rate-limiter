# rs-rate-limiter

[![CI](https://github.com/philiprehberger/rs-rate-limiter/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-rate-limiter/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-rate-limiter.svg)](https://crates.io/crates/philiprehberger-rate-limiter)
[![License](https://img.shields.io/github/license/philiprehberger/rs-rate-limiter)](LICENSE)

Token bucket, sliding window, and fixed window rate limiting.

## Installation

```toml
[dependencies]
philiprehberger-rate-limiter = "0.1.6"
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

## API

| Type / Trait | Description |
|--------------|-------------|
| `RateLimiter` | Trait with `fn check(&self, key: &str) -> Decision` |
| `Decision` | `Allowed` or `Denied { retry_after: Duration }` |
| `TokenBucket::new(capacity, refill_rate)` | Token bucket limiter (tokens/sec) |
| `SlidingWindow::new(window, max_requests)` | Sliding window counter |
| `FixedWindow::new(window, max_requests)` | Fixed window counter |

## License

MIT
