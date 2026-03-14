# sigc Architecture

## Brokerless Distributed Design

sigc implements a **brokerless distributed architecture** using **nng (Nanomsg)** for inter-process communication. This design avoids external dependencies like Redis, RabbitMQ, or Kafka while still enabling concurrent access and caching.

## Core Principles

### 1. **Single Owner Per Resource**

Each shared resource (like the cache) has exactly one owner process:

```
┌─────────────────────────┐
│    Daemon Process       │
│  (Resource Owner)       │
│   ┌───────────────┐     │
│   │ Cache (Owned) │◄────┼─── Exclusive ownership
│   └───────────────┘     │
│   ┌───────────────┐     │
│   │   Compiler    │     │
│   │   Runtime     │     │
│   └───────────────┘     │
└─────────────────────────┘
```

### 2. **Message Passing via NNG**

Clients communicate with the daemon using REQ/REP (request/reply) pattern:

```
Client 1 ──REQ──┐
                ├──► Daemon (REP)
Client 2 ──REQ──┘
```

**No Shared Memory** = No Lock Contention

### 3. **Retry with Exponential Backoff**

When cache lock is detected (e.g., daemon running but client trying standalone mode):

```rust
// Attempts: 1, 2, 3 with waits: 100ms, 200ms, 400ms
try_open_cache_with_retry(cache_dir, 3)
```

## Architecture Modes

### Standalone Mode

Single process owns cache directly:

```bash
sigc compile strategy.sig    # Opens cache, compiles, closes
sigc run strategy.sig         # Opens cache, runs, closes
```

```
┌─────────────┐
│  sigc CLI   │
│  ┌───────┐  │
│  │ Cache │  │ ← Direct ownership
│  └───────┘  │
│  ┌───────┐  │
│  │Compile│  │
│  │  Run  │  │
│  └───────┘  │
└─────────────┘
```

### Daemon Mode (Recommended for Production)

One daemon process serves multiple clients:

```bash
# Terminal 1: Start daemon
sigc daemon

# Terminal 2-N: Clients send requests
sigc request compile strategy.sig
sigc request run strategy.sig
sigc request status
```

```
        Daemon (Port 7240)
┌─────────────────────────────┐
│  ┌──────────────────┐       │
│  │  Cache (Owned)   │       │
│  │  - IR cache      │       │
│  │  - Result cache  │       │
│  └──────────────────┘       │
│  ┌──────────────────┐       │
│  │  Compiler        │       │
│  │  Runtime         │       │
│  └──────────────────┘       │
│           ▲                 │
└───────────┼─────────────────┘
            │ NNG REQ/REP (tcp://127.0.0.1:7240)
    ┌───────┼────────┐
    │       │        │
┌───┴───┐ ┌─┴────┐ ┌─┴────┐
│Client1│ │Client│ │Client│
│compile│ │ run  │ │status│
└───────┘ └──────┘ └──────┘
```

## Cache Ownership Strategy

### Problem: File Lock Contention

sled (embedded database) uses file locking:

```rust
// ❌ Both try to lock same file
Daemon: sled::open("/home/user/.cache/sigc/db")  // Gets lock
Client: sled::open("/home/user/.cache/sigc/db")  // BLOCKED!
```

### Solution: Exclusive Daemon Ownership

```rust
// ✅ Daemon owns cache
impl Daemon {
    pub fn new(address: &str) -> Result<Self> {
        // Daemon opens cache ONCE
        let cache = Cache::open(&cache_dir)?;

        // Compiler and runtime share cache via Arc (Clone)
        let compiler = Compiler::with_cache(cache.clone());
        let runtime = Runtime::with_cache(cache.clone());

        // Cache stays open for daemon lifetime
        Ok(Daemon { cache, compiler, runtime, ... })
    }
}
```

```rust
// ✅ Clients NEVER touch cache
impl Client {
    pub fn compile(&self, source: &str) -> Result<Response> {
        // Send request over network
        self.request(&Request::Compile { source })
        // No cache access!
    }
}
```

### Fallback: Standalone with Retry

If client detects daemon isn't running:

```rust
match try_open_cache_with_retry(cache_dir, 3) {
    Ok(cache) => {
        // Standalone mode: use cache directly
        let compiler = Compiler::with_cache(cache);
        compiler.compile(source)?
    }
    Err(e) if e.contains("lock") => {
        // Cache locked -> daemon running
        eprintln!("Cache locked. Use 'sigc request compile' instead");
        exit(1);
    }
}
```

## Async Architecture with Tokio

### Hybrid Async/Sync Design

sigc uses a **hybrid async design** that wraps synchronous nng operations with Tokio's async runtime:

```rust
// Async daemon using tokio::task::spawn_blocking
pub async fn run(&mut self) -> Result<()> {
    loop {
        // Wrap blocking recv in spawn_blocking to avoid blocking tokio runtime
        let socket_clone = Arc::clone(&socket);
        let msg = tokio::task::spawn_blocking(move || socket_clone.recv()).await?;

        // Handle request in blocking task (CPU-intensive work)
        let response = tokio::task::spawn_blocking(move || {
            Self::handle_request_sync(&compiler, &runtime, request)
        }).await?;

        // Send response (also wrapped)
        tokio::task::spawn_blocking(move || socket.send(&data)).await?;
    }
}
```

**Benefits:**
- ✅ **Non-blocking**: Tokio runtime stays responsive even during I/O
- ✅ **Brokerless**: Still uses NNG directly (no Redis/Kafka)
- ✅ **Simple**: No need for complex async NNG bindings
- ✅ **Efficient**: CPU-intensive work runs on dedicated blocking threads

### Runtime Pool for Concurrent Execution

```rust
pub struct Daemon {
    socket: Arc<Socket>,                              // Shared socket (read-only)
    compiler: Arc<Compiler>,                           // Shared compiler (immutable)
    runtime_pool: Arc<Vec<Mutex<Runtime>>>,            // Pool of runtime workers
    pool_size: usize,                                  // Number of workers
    requests_handled: Arc<AtomicU64>,                 // Lock-free counter
}
```

**Key Features:**
- `Arc<Vec<Mutex<Runtime>>>`: Pool of independent runtime instances
- **Round-robin selection**: `runtime_idx = request_id % pool_size`
- **True parallelism**: Multiple backtests run simultaneously on different workers
- **Configurable pool size**: Defaults to CPU core count, customizable via `--workers` flag

**Thread Management (Avoiding Core Saturation):**

The daemon uses smart defaults to prevent thread oversubscription:

```
Machine with 8 CPU cores:
- Default pool size: 4 workers (num_cpus / 2)
- Each worker uses Rayon: 2 threads (8 cores / 4 workers)
- Total threads: 4 workers × 2 Rayon threads = 8 threads ✅
```

**Example Usage:**
```bash
# Auto-configured (recommended): pool_size = num_cpus / 2
sigc daemon
→ 8 cores: 4 workers × 2 Rayon threads each = 8 total threads

# Custom pool size with auto Rayon adjustment
sigc daemon --workers 8
→ 8 cores: 8 workers × 1 Rayon thread each = 8 total threads

# Advanced: Manual control via environment variable
RAYON_NUM_THREADS=4 sigc daemon --workers 2
→ 2 workers × 4 Rayon threads each = 8 total threads
```

**Why This Matters:**
- ❌ **Without limits**: 8 workers × 8 Rayon threads = 64 threads → thrashing!
- ✅ **With smart defaults**: Total threads ≈ num_cpus → optimal performance

### Thread-Safety with Arc and Mutex

- `Arc<Socket>`: Shared across async tasks for sending/receiving
- `Arc<Compiler>`: Immutable compilation shared across requests (no mutex needed)
- `Arc<Vec<Mutex<Runtime>>>`: Pool of runtimes, each with exclusive access via Mutex
- `Arc<AtomicU64>`: Lock-free atomic counter for request tracking

## NNG Communication Protocol

### Request Types

```rust
pub enum Request {
    Ping,                          // Health check
    Compile { source: String },    // Compile .sig to IR
    Run { source: String },        // Run backtest
    Status,                        // Get daemon status
    Shutdown,                      // Stop daemon
}
```

### Response Types

```rust
pub enum Response {
    Pong,
    CompileResult { success, nodes, error },
    RunResult { success, total_return, sharpe_ratio, ... },
    Status { version, uptime_secs, requests_handled },
    Shutdown,
    Error { message },
}
```

### Wire Format

JSON over NNG sockets:

```
Client                          Daemon
  |                               |
  |---REQ: {"type":"Compile",---->|
  |        "source":"..."}        | compile()
  |                               |
  |<--REP: {"type":"CompileRes----|
  |         ult","success":true,  |
  |         "nodes":42}            |
```

## Concurrency Model

### Request-Level Concurrency

The daemon uses **multiple runtime workers** for concurrent request handling:

```rust
// Spawn concurrent task for each request
tokio::spawn(async move {
    // Select runtime from pool (round-robin)
    let runtime_idx = (request_id as usize) % pool_size;
    let runtime = &runtime_pool[runtime_idx];

    // Execute on blocking thread pool
    tokio::task::spawn_blocking(move || {
        let mut runtime_guard = runtime.lock().unwrap();
        runtime_guard.run_ir(&ir)
    }).await
});
```

**Benefits:**
- ✅ **Concurrent backtests**: Multiple clients can run backtests simultaneously
- ✅ **No blocking**: Fast requests (ping, status) don't wait for slow ones
- ✅ **Scales with CPUs**: Pool size defaults to core count
- ✅ **Round-robin load balancing**: Requests distributed evenly across workers

### Thread Safety

- `AtomicBool` for kill switch (lock-free)
- `AtomicU64` for request counter (lock-free)
- `Mutex<Runtime>` for mutable runtime access (per-worker serialization)
- `RwLock` for circuit breakers (multiple readers, single writer)
- `Arc` for shared state (reference counting)

### Data-Level Parallelism

```rust
// Rayon for data parallelism
assets.par_iter()
    .map(|asset| process(asset))
    .collect()

// SIMD for compute parallelism
rolling_mean_simd(&data, window)
```

### Async I/O

```rust
// PostgreSQL connection pooling
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&db_url).await?;

// Concurrent queries
let (prices, volumes) = tokio::try_join!(
    fetch_prices(&pool),
    fetch_volumes(&pool)
)?;
```

## Why Brokerless?

### ✅ Advantages

1. **No External Dependencies**
   - No Redis, RabbitMQ, Kafka to install/manage
   - Simpler deployment and testing

2. **Lower Latency**
   - Direct IPC via Unix sockets or TCP loopback
   - No network hop to broker

3. **Simpler Failure Modes**
   - One process (daemon) vs three (app + broker + queue)
   - Easier to reason about failures

4. **Resource Efficiency**
   - No broker memory overhead
   - No serialization to external format

### ❌ Limitations

1. **Single Node Only**
   - Can't distribute across multiple machines
   - Solution: Use for single-node deployments

2. **No Persistence of Requests**
   - If daemon crashes, in-flight requests lost
   - Solution: Clients retry, or use persistent queue if needed

3. **No Fan-Out**
   - One daemon can't broadcast to multiple workers
   - Solution: Sufficient for most quant workloads

## Scaling Strategy

### Vertical Scaling (Current)

```
┌────────────────────────────┐
│     Single Machine         │
│  ┌──────────────────────┐  │
│  │  Daemon (16 cores)   │  │ ← SIMD + Rayon parallelism
│  │  - Cache             │  │
│  │  - Compiler          │  │
│  │  - Runtime           │  │
│  └──────────────────────┘  │
│           ▲                │
│           │ NNG            │
│     100+ clients           │
└────────────────────────────┘
```

### Horizontal Scaling (Future)

For distributed deployments requiring multiple nodes:

```
                  ┌─────────────┐
                  │   Redis     │ ← Shared cache
                  │   Cluster   │
                  └──────┬──────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
    ┌─────┴────┐   ┌─────┴────┐   ┌────┴─────┐
    │ Worker 1 │   │ Worker 2 │   │ Worker 3 │
    │ - Compile│   │ - Compile│   │ - Compile│
    │ - Run    │   │ - Run    │   │ - Run    │
    └──────────┘   └──────────┘   └──────────┘
          ▲              ▲              ▲
          └──────────────┼──────────────┘
                         │
                  ┌──────┴──────┐
                  │ Load Balancer│
                  └─────────────┘
```

**When to use:**
- Processing > 10,000 backtests/day
- Multi-datacenter deployment
- Need high availability (HA)

## Best Practices

### 1. Use Daemon Mode in Production

```bash
# supervisord config
[program:sigc-daemon]
command=/usr/local/bin/sigc daemon
autostart=true
autorestart=true
```

### 2. Graceful Shutdown

```rust
// Daemon handles SIGTERM
tokio::signal::ctrl_c().await?;
tracing::info!("Shutting down gracefully...");
daemon.shutdown()?;
```

### 3. Monitor Daemon Health

```bash
#!/bin/bash
# Health check script
sigc request ping || {
    echo "Daemon unhealthy, restarting..."
    systemctl restart sigc-daemon
}
```

### 4. Cache Eviction Policy

```rust
// Daemon monitors cache size
if cache.size() > MAX_CACHE_SIZE {
    cache.evict_lru(0.2);  // Evict oldest 20%
}
```

## Performance Characteristics

| Operation | Latency | Throughput |
|-----------|---------|------------|
| REQ/REP roundtrip | ~100μs | 10K req/sec |
| Cache hit | ~10μs | 100K ops/sec |
| Cache miss (compile) | ~10ms | 100 ops/sec |
| Backtest (252 days) | ~50ms | 20 runs/sec |

## Monitoring

### Daemon Metrics

```bash
$ sigc request status
Status:
  Version:  0.8.0
  Uptime:   3600s
  Requests: 15234
```

### Cache Metrics

```rust
cache.stats();  // { hits: 1234, misses: 56, evictions: 12 }
```

### System Resources

```bash
$ ps aux | grep sigc-daemon
user  1234  0.5  2.1  512MB  8.1GB  sigc daemon
```

## Future Enhancements

1. ✅ **Async NNG** - ~~Replace blocking I/O with `async-nng`~~ **DONE**: Using Tokio spawn_blocking wrapper
2. **Work Stealing** - Multiple daemon workers with load balancing
3. **Streaming** - Server-sent events for long-running backtests
4. **Distributed Cache** - Optional Redis backend for multi-node
5. **gRPC Alternative** - Option to use gRPC instead of NNG

## Summary

sigc's brokerless architecture provides:

✅ **Simple deployment** - No external dependencies
✅ **Low latency** - Direct IPC communication
✅ **Thread safety** - Proper use of atomic operations and locks
✅ **Scalability** - Vertical scaling via parallelism, horizontal option via Redis
✅ **Robustness** - Retry logic and graceful error handling

The design prioritizes **simplicity and performance** for single-node quant workloads while maintaining extensibility for future distributed scenarios.
