# rs-rate-limiter

Token bucket, sliding window, and fixed window rate limiting.

## Installation

```toml
[dependencies]
philiprehberger-rate-limiter = "0.1"
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
