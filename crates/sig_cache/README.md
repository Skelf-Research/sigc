# sig_cache

Content-addressed caching for deterministic backtesting in sigc.

## Overview

`sig_cache` provides a high-performance caching layer that ensures reproducible backtest results:

- **Content-addressed storage** - Uses Blake3 hashing for cache keys
- **Deterministic outputs** - Same inputs always produce same outputs
- **Persistent storage** - Uses sled for fast embedded database
- **Zero-copy serialization** - Uses rkyv for efficient serialization

## Usage

```rust
use sig_cache::Cache;

let cache = Cache::new("~/.sigc/cache")?;

// Cache compiled plans
let key = cache.hash(&input);
if let Some(cached) = cache.get(&key)? {
    return Ok(cached);
}

let result = expensive_computation()?;
cache.put(&key, &result)?;
```

## Part of sigc

This crate is part of the [sigc](https://github.com/skelf-Research/sigc) quantitative finance platform.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
