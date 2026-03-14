# Chapter 11: External Integrations

This chapter covers integrating sigc with external data providers and brokerage APIs for live trading.

## Market Data Providers

### Yahoo Finance

Free historical and real-time market data.

```rust
use sig_runtime::{YahooFinance, MarketDataProvider};

let yahoo = YahooFinance::new();

// Get historical OHLCV data
let df = yahoo.get_ohlcv(
    "AAPL",
    "2023-01-01",
    "2023-12-31",
    "1d"
)?;

// Get current quote
let quote = yahoo.get_quote("AAPL")?;
println!("Last: ${}, Bid: ${}, Ask: ${}",
    quote.last, quote.bid, quote.ask);

// Get multiple quotes
let quotes = yahoo.get_quotes(&["AAPL", "MSFT", "GOOGL"])?;
```

**Intervals:**
- `1m`, `5m`, `15m`, `30m` - Intraday
- `1h` - Hourly
- `1d`, `daily` - Daily
- `1wk`, `weekly` - Weekly
- `1mo`, `monthly` - Monthly

### Integration Registry

Manage multiple data providers:

```rust
use sig_runtime::IntegrationRegistry;

let mut registry = IntegrationRegistry::new();

// Register providers
registry.register_provider(YahooFinance::new());

// Get provider by name
let yahoo = registry.get_provider("yahoo_finance")
    .expect("Provider not found");

// List available providers
let providers = registry.list_providers();
```

## Broker Integrations

### Alpaca

Paper and live trading with Alpaca Markets.

#### Setup

```rust
use sig_runtime::AlpacaBroker;

// Paper trading
let alpaca = AlpacaBroker::paper(
    "your_api_key",
    "your_api_secret"
);

// Live trading (use with caution!)
let alpaca = AlpacaBroker::live(
    "your_api_key",
    "your_api_secret"
);
```

#### Account Information

```rust
// Get account
let account = alpaca.get_account()?;
println!("Cash: ${}", account.cash);
println!("Portfolio Value: ${}", account.portfolio_value);
println!("Buying Power: ${}", account.buying_power);

// Get positions
let positions = alpaca.get_positions()?;
for pos in positions {
    println!("{}: {} shares @ ${} (P&L: ${})",
        pos.symbol, pos.qty, pos.avg_entry_price, pos.unrealized_pl);
}
```

#### Order Management

**Market Orders:**

```rust
use sig_runtime::AlpacaOrder;

// Buy 100 shares at market
let order = AlpacaOrder::market_buy("AAPL", 100.0);
let response = alpaca.submit_order(&order)?;
println!("Order ID: {}, Status: {}", response.id, response.status);

// Sell 50 shares at market
let order = AlpacaOrder::market_sell("MSFT", 50.0);
alpaca.submit_order(&order)?;
```

**Limit Orders:**

```rust
// Buy 100 shares at limit price $150
let order = AlpacaOrder::limit_buy("AAPL", 100.0, 150.0);
let response = alpaca.submit_order(&order)?;

// Sell 50 shares at limit price $350
let order = AlpacaOrder::limit_sell("MSFT", 50.0, 350.0);
alpaca.submit_order(&order)?;
```

**Cancel Orders:**

```rust
// Cancel an order
alpaca.cancel_order("order-id-123")?;
```

#### Historical Data

```rust
// Get historical bars from Alpaca
let df = alpaca.get_bars(
    "AAPL",
    "2023-01-01",
    "2023-12-31",
    "1Day"
)?;
```

## Real-Time Streaming

### WebSocket Client

Stream live market data:

```rust
use sig_runtime::StreamingClient;

let mut client = StreamingClient::new("wss://stream.alpaca.markets/v2/iex");

// Subscribe to symbols
client.subscribe("AAPL");
client.subscribe("MSFT");
client.subscribe("GOOGL");

// Unsubscribe
client.unsubscribe("GOOGL");

// Get current subscriptions
let subs = client.subscriptions();
println!("Subscribed to {} symbols", subs.len());
```

### Live Trading Loop

```rust
use sig_runtime::{AlpacaBroker, YahooFinance, StreamingClient};

// Initialize
let alpaca = AlpacaBroker::paper("api_key", "api_secret");
let yahoo = YahooFinance::new();
let mut stream = StreamingClient::new("wss://stream.alpaca.markets/v2/iex");

// Subscribe to universe
let symbols = vec!["AAPL", "MSFT", "GOOGL", "AMZN"];
for symbol in &symbols {
    stream.subscribe(symbol);
}

// Trading loop (pseudocode)
loop {
    // Get current positions
    let positions = alpaca.get_positions()?;

    // Get latest quotes
    let quotes = yahoo.get_quotes(&symbols)?;

    // Run strategy (compute signals)
    let signals = compute_signals(&quotes, &positions)?;

    // Generate orders
    for (symbol, signal) in signals {
        if signal > 0.0 {
            let order = AlpacaOrder::market_buy(symbol, signal * 100.0);
            alpaca.submit_order(&order)?;
        } else if signal < 0.0 {
            let order = AlpacaOrder::market_sell(symbol, -signal * 100.0);
            alpaca.submit_order(&order)?;
        }
    }

    // Sleep until next rebalance
    std::thread::sleep(std::time::Duration::from_secs(60));
}
```

## Data Vendor Framework

### Custom Provider

Implement your own data provider:

```rust
use sig_runtime::{MarketDataProvider, Quote};
use polars::prelude::*;
use sig_types::Result;

struct MyDataProvider {
    api_key: String,
}

impl MarketDataProvider for MyDataProvider {
    fn get_ohlcv(
        &self,
        symbol: &str,
        start: &str,
        end: &str,
        interval: &str,
    ) -> Result<DataFrame> {
        // Implement data fetching logic
        // Return DataFrame with columns: date, open, high, low, close, volume
        todo!()
    }

    fn get_quote(&self, symbol: &str) -> Result<Quote> {
        // Implement quote fetching logic
        todo!()
    }

    fn get_quotes(&self, symbols: &[&str]) -> Result<Vec<Quote>> {
        symbols.iter()
            .map(|s| self.get_quote(s))
            .collect()
    }

    fn name(&self) -> &str {
        "my_provider"
    }
}
```

### Register Custom Provider

```rust
let mut registry = IntegrationRegistry::new();
registry.register_provider(MyDataProvider {
    api_key: "my_api_key".to_string(),
});

// Use it
let provider = registry.get_provider("my_provider").unwrap();
let df = provider.get_ohlcv("AAPL", "2023-01-01", "2023-12-31", "1d")?;
```

## Best Practices

### API Keys

**Never hardcode API keys:**

```rust
// ❌ Bad
let alpaca = AlpacaBroker::paper("AKXXX...", "secretXXX...");

// ✅ Good - use environment variables
use std::env;
let api_key = env::var("ALPACA_API_KEY")
    .expect("ALPACA_API_KEY not set");
let api_secret = env::var("ALPACA_API_SECRET")
    .expect("ALPACA_API_SECRET not set");
let alpaca = AlpacaBroker::paper(&api_key, &api_secret);
```

### Rate Limiting

Respect API rate limits:

```rust
use sig_runtime::RateLimiter;

// Alpaca allows 200 requests per minute
let limiter = RateLimiter::new("alpaca", 200, 200);

for symbol in symbols {
    if !limiter.try_acquire() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let quote = yahoo.get_quote(symbol)?;
    // Process quote...
}
```

### Error Handling

Handle network and API errors gracefully:

```rust
use sig_runtime::CircuitBreaker;

let breaker = CircuitBreaker::new(
    "yahoo_api",
    CircuitBreakerConfig::default()
);

if !breaker.allow_request() {
    return Err("Circuit breaker open".into());
}

match yahoo.get_quote("AAPL") {
    Ok(quote) => {
        breaker.record_success();
        // Process quote...
    }
    Err(e) => {
        breaker.record_failure();
        eprintln!("Failed to get quote: {}", e);
    }
}
```

### Paper Trading First

Always test strategies in paper trading before going live:

```rust
// Start with paper trading
#[cfg(debug_assertions)]
let alpaca = AlpacaBroker::paper(&api_key, &api_secret);

// Only enable live trading in release mode with explicit opt-in
#[cfg(all(not(debug_assertions), feature = "live_trading"))]
let alpaca = AlpacaBroker::live(&api_key, &api_secret);
```

## Security Considerations

1. **API Key Storage**
   - Use environment variables or secure vaults
   - Never commit keys to version control
   - Rotate keys regularly

2. **Network Security**
   - Use HTTPS/WSS only
   - Verify SSL certificates
   - Consider using VPN for sensitive operations

3. **Order Validation**
   - Implement pre-trade checks
   - Set position size limits
   - Use kill switches for emergencies

4. **Audit Logging**
   - Log all orders and fills
   - Track API calls and errors
   - Monitor for unusual activity

## Common Issues

### Connection Errors

```rust
// Retry with exponential backoff
let mut backoff = 1;
let max_retries = 5;

for attempt in 1..=max_retries {
    match yahoo.get_quote("AAPL") {
        Ok(quote) => return Ok(quote),
        Err(e) if attempt < max_retries => {
            eprintln!("Attempt {} failed: {}", attempt, e);
            std::thread::sleep(std::time::Duration::from_secs(backoff));
            backoff *= 2;
        }
        Err(e) => return Err(e),
    }
}
```

### Rate Limit Errors

```rust
// Implement request queue
use std::collections::VecDeque;
use std::time::Instant;

let mut request_times: VecDeque<Instant> = VecDeque::new();
let max_requests_per_minute = 200;

fn rate_limited_request() -> Result<()> {
    let now = Instant::now();

    // Remove requests older than 1 minute
    while let Some(&time) = request_times.front() {
        if now.duration_since(time).as_secs() > 60 {
            request_times.pop_front();
        } else {
            break;
        }
    }

    // Check if we can make request
    if request_times.len() >= max_requests_per_minute {
        let wait_time = 60 - now.duration_since(*request_times.front().unwrap()).as_secs();
        std::thread::sleep(std::time::Duration::from_secs(wait_time));
    }

    request_times.push_back(now);
    // Make request...
    Ok(())
}
```

## Key Takeaways

1. **Multiple providers**: Support Yahoo Finance, Alpaca, and custom providers
2. **Paper trading**: Always test strategies in paper mode first
3. **Security**: Protect API keys and implement proper authentication
4. **Rate limiting**: Respect API limits to avoid being blocked
5. **Error handling**: Implement retries and circuit breakers
6. **Monitoring**: Log all trades and track system health

## Exercises

1. **Data comparison**: Fetch the same symbol from multiple providers and compare prices.

2. **Paper trading**: Implement a simple strategy and run it in paper trading mode for a week.

3. **Order management**: Create a system that tracks open orders and manages cancellations.

4. **Custom provider**: Implement a custom data provider for a specific exchange or data source.

5. **Streaming client**: Build a real-time dashboard that displays live quotes for a watchlist.

## Resources

- [Alpaca API Docs](https://alpaca.markets/docs/)
- [Yahoo Finance API](https://github.com/ranaroussi/yfinance)
- [Safety Systems](09-deployment-safety.md)
- [Production Guide](07-production.md)
