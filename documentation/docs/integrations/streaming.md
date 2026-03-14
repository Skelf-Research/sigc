# Streaming Data

Real-time and streaming data integration.

## Overview

sigc supports streaming data for:

- Real-time signal updates
- Intraday strategies
- Live monitoring
- Production trading

## Supported Providers

| Provider | Data Types | Latency |
|----------|------------|---------|
| [Polygon](#polygon) | Trades, quotes, bars | ~50ms |
| [Alpaca](#alpaca-streaming) | Trades, quotes, bars | ~100ms |
| [IEX Cloud](#iex-cloud) | Trades, quotes | ~50ms |
| [Custom](#custom-websocket) | Any | Varies |

## Polygon

### Configuration

```yaml
data:
  type: streaming
  provider: polygon

  polygon:
    api_key: ${POLYGON_API_KEY}
    feed: delayed  # delayed | real-time
```

### Subscribe to Symbols

```yaml
data:
  type: streaming
  provider: polygon

  polygon:
    api_key: ${POLYGON_API_KEY}

  subscriptions:
    - type: bars
      symbols: ["AAPL", "MSFT", "GOOGL"]
      timeframe: 1min

    - type: trades
      symbols: ["AAPL"]

    - type: quotes
      symbols: ["AAPL"]
```

### Data Types

| Type | Description | Fields |
|------|-------------|--------|
| `bars` | OHLCV candles | open, high, low, close, volume |
| `trades` | Individual trades | price, size, timestamp |
| `quotes` | Bid/ask quotes | bid, ask, bid_size, ask_size |

## Alpaca Streaming

### Configuration

```yaml
data:
  type: streaming
  provider: alpaca

  alpaca:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}
    feed: iex  # iex | sip
```

### Subscriptions

```yaml
data:
  type: streaming
  provider: alpaca

  subscriptions:
    - type: bars
      symbols: ["AAPL", "MSFT"]
      timeframe: 1min

    - type: trades
      symbols: ["AAPL"]

    - type: quotes
      symbols: ["AAPL"]
```

## IEX Cloud

### Configuration

```yaml
data:
  type: streaming
  provider: iex

  iex:
    api_key: ${IEX_API_KEY}
    sandbox: false
```

### Subscriptions

```yaml
data:
  type: streaming
  provider: iex

  subscriptions:
    - type: last
      symbols: ["AAPL", "MSFT"]

    - type: tops
      symbols: ["AAPL"]
```

## Custom WebSocket

### Configuration

```yaml
data:
  type: streaming
  provider: websocket

  websocket:
    url: wss://your-data-source.com/stream
    auth:
      type: bearer
      token: ${WS_TOKEN}

    subscribe_message: |
      {"action": "subscribe", "symbols": ${symbols}}

    parser:
      type: json
      timestamp_field: "t"
      symbol_field: "s"
      price_field: "p"
      volume_field: "v"
```

## Streaming Strategy

### Real-Time Signal

```sig
data:
  type: streaming
  provider: polygon
  subscriptions:
    - type: bars
      symbols: ["AAPL", "MSFT", "GOOGL"]
      timeframe: 1min

signal intraday_momentum:
  // Rolling calculations on streaming data
  ret_5m = ret(close, 5)
  ret_15m = ret(close, 15)

  momentum = ret_5m - ret_15m

  emit zscore(momentum)

portfolio live:
  weights = rank(intraday_momentum).long_short(top=0.3, bottom=0.3)

  // Real-time execution
  execute:
    on_signal_change: true
    min_change: 0.05  # Minimum 5% weight change
```

### Event-Driven Updates

```yaml
streaming:
  update_policy:
    type: on_bar        # on_bar | on_trade | interval
    # type: interval
    # interval_seconds: 60

  buffer:
    type: rolling
    window: 100         # Keep last 100 bars

  warm_up:
    bars: 20            # Require 20 bars before signals
```

## Buffering and Windowing

### Rolling Buffer

```yaml
streaming:
  buffer:
    type: rolling
    window: 1000        # Keep last 1000 bars
    persist: true       # Save to disk
```

### Time Window

```yaml
streaming:
  buffer:
    type: time
    window: 1h          # Keep last 1 hour
```

### Memory Management

```yaml
streaming:
  memory:
    max_buffer_mb: 500
    gc_interval_seconds: 60
```

## Aggregation

### Bar Aggregation

```yaml
streaming:
  aggregation:
    from: 1min
    to: 5min
    method: ohlcv       # Combine into OHLCV
```

### Custom Aggregation

```yaml
streaming:
  aggregation:
    from: trades
    to: bars
    interval: 1min
    fields:
      open: first
      high: max
      low: min
      close: last
      volume: sum
```

## Signal Updates

### Update Frequency

```yaml
streaming:
  signals:
    update_frequency: on_bar  # on_bar | interval | on_change
    # update_interval_seconds: 30
```

### Throttling

```yaml
streaming:
  signals:
    throttle:
      max_updates_per_minute: 10
      min_change: 0.01    # Minimum signal change to trigger
```

## Execution Integration

### Real-Time Execution

```yaml
streaming:
  execution:
    enabled: true
    provider: alpaca

    triggers:
      - type: signal_change
        threshold: 0.05   # 5% weight change
        action: rebalance

      - type: scheduled
        cron: "0 */30 9-15 * * 1-5"  # Every 30 min during market
        action: rebalance
```

### Order Staging

```yaml
streaming:
  execution:
    staging:
      enabled: true
      review_seconds: 60  # Manual review period
      auto_approve: false
```

## Monitoring

### Metrics

```prometheus
sigc_streaming_messages_total{provider="polygon",type="bars"} 15234
sigc_streaming_latency_ms{provider="polygon"} 45
sigc_streaming_errors_total{provider="polygon"} 3
sigc_streaming_reconnects_total{provider="polygon"} 2
```

### Health Checks

```yaml
streaming:
  health:
    heartbeat_interval_seconds: 30
    max_silence_seconds: 60
    on_disconnect: reconnect  # reconnect | alert | halt
```

## Error Handling

### Reconnection

```yaml
streaming:
  reconnect:
    enabled: true
    max_attempts: 10
    initial_delay_seconds: 1
    max_delay_seconds: 60
    backoff: exponential
```

### Data Gaps

```yaml
streaming:
  gaps:
    detection: true
    max_gap_seconds: 120
    on_gap: backfill  # backfill | interpolate | alert
```

### Stale Data

```yaml
streaming:
  stale_data:
    threshold_seconds: 300
    action: alert  # alert | halt | use_last
```

## Example: Complete Streaming Setup

```yaml
# sigc.yaml
data:
  type: streaming
  provider: polygon

  polygon:
    api_key: ${POLYGON_API_KEY}
    feed: delayed

  subscriptions:
    - type: bars
      symbols: ["AAPL", "MSFT", "GOOGL", "AMZN", "META",
                "NVDA", "TSLA", "JPM", "JNJ", "V"]
      timeframe: 1min

streaming:
  buffer:
    type: rolling
    window: 500
    persist: true

  update_policy:
    type: on_bar

  warm_up:
    bars: 60

  signals:
    update_frequency: on_bar
    throttle:
      max_updates_per_minute: 6

  execution:
    enabled: true
    provider: alpaca
    triggers:
      - type: signal_change
        threshold: 0.05
        action: rebalance

  reconnect:
    enabled: true
    max_attempts: 10

  health:
    heartbeat_interval_seconds: 30
    max_silence_seconds: 120

output:
  type: alpaca
  alpaca:
    paper: true
    orders:
      type: limit
```

## CLI Commands

### Start Streaming

```bash
sigc stream start --config sigc.yaml
```

### Monitor Status

```bash
sigc stream status
```

```
Streaming Status:
  Provider: polygon
  Connected: Yes
  Uptime: 2h 15m
  Messages: 15,234
  Latency: 45ms

Subscriptions:
  bars (1min): 10 symbols
  Last update: 2s ago

Buffer:
  Size: 4,523 bars
  Memory: 125 MB
```

### Pause/Resume

```bash
sigc stream pause
sigc stream resume
```

### View Live Data

```bash
sigc stream watch AAPL
```

## Best Practices

### 1. Start with Delayed Data

```yaml
polygon:
  feed: delayed  # Cheaper, good for testing
```

### 2. Buffer Appropriately

```yaml
buffer:
  window: 500   # Enough for your calculations
```

### 3. Handle Reconnection

```yaml
reconnect:
  enabled: true
  max_attempts: 10
```

### 4. Monitor Latency

Track latency metrics for execution quality.

### 5. Test with Paper Trading

```yaml
output:
  alpaca:
    paper: true
```

## Next Steps

- [Alpaca](alpaca.md) - Trade execution
- [Production](../production/index.md) - Production deployment
- [Monitoring](../production/monitoring.md) - Metrics and alerts
