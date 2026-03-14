# Alpaca Integration

Execute trades through Alpaca Markets.

## Overview

Alpaca provides:

- Commission-free trading
- Paper trading for testing
- REST and WebSocket APIs
- Extended hours trading

## Setup

### Create Account

1. Sign up at [alpaca.markets](https://alpaca.markets)
2. Generate API keys in dashboard
3. Note your API key and secret

### Configure sigc

```yaml
# sigc.yaml
output:
  type: alpaca

  alpaca:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}
    base_url: https://api.alpaca.markets  # Live
    # base_url: https://paper-api.alpaca.markets  # Paper
    paper: true  # Start with paper trading
```

### Environment Variables

```bash
export ALPACA_API_KEY=your_api_key
export ALPACA_API_SECRET=your_api_secret
```

## Paper Trading

Always test with paper trading first:

```yaml
output:
  type: alpaca
  alpaca:
    paper: true  # Paper trading mode
```

Paper trading:

- Uses separate paper account
- Same API, different endpoint
- Real market data
- Simulated execution

## Order Types

### Market Orders

```yaml
output:
  alpaca:
    orders:
      type: market
      time_in_force: day
```

### Limit Orders

```yaml
output:
  alpaca:
    orders:
      type: limit
      limit_offset_pct: 0.1  # 0.1% better than market
      time_in_force: day
```

### Order Parameters

| Parameter | Values | Description |
|-----------|--------|-------------|
| `type` | `market`, `limit`, `stop`, `stop_limit` | Order type |
| `time_in_force` | `day`, `gtc`, `ioc`, `fok` | Time in force |
| `extended_hours` | `true`, `false` | Trade in extended hours |

## Execution Configuration

### Basic Execution

```yaml
output:
  alpaca:
    execution:
      algorithm: immediate  # Submit all orders at once
```

### TWAP Execution

```yaml
output:
  alpaca:
    execution:
      algorithm: twap
      duration_minutes: 30
      slices: 6  # 5-minute intervals
```

### VWAP Execution

```yaml
output:
  alpaca:
    execution:
      algorithm: vwap
      duration_minutes: 60
      participation_rate: 0.05  # 5% of volume
```

### Smart Routing

```yaml
output:
  alpaca:
    execution:
      smart_routing: true
      min_quantity: 100
      round_lots: true
```

## Position Synchronization

### Sync Mode

```yaml
output:
  alpaca:
    sync:
      mode: full  # full | incremental
      reconcile: true
```

### Full Sync

Closes all positions not in target weights:

```yaml
sync:
  mode: full
  close_unlisted: true  # Close positions not in signal
```

### Incremental Sync

Only adjusts positions in target:

```yaml
sync:
  mode: incremental
  close_unlisted: false
```

## Account Information

### Check Account

```bash
sigc alpaca account
```

```
Alpaca Account:
  Account ID: abc123
  Status: ACTIVE
  Cash: $100,000.00
  Portfolio Value: $250,000.00
  Buying Power: $500,000.00
  Equity: $250,000.00
  Margin Multiplier: 2x

Restrictions:
  Trading Blocked: No
  Transfers Blocked: No
```

### Check Positions

```bash
sigc alpaca positions
```

```
Current Positions:
Ticker | Quantity | Market Value | Unrealized P&L
-------+----------+--------------+---------------
AAPL   |      500 |    $92,820   |       +$1,520
MSFT   |      250 |    $93,628   |       +$2,128
GOOGL  |      100 |    $14,021   |         -$179
...
```

## Order Management

### View Orders

```bash
sigc alpaca orders
```

```
Open Orders:
Order ID | Ticker | Side | Qty | Type  | Status
---------+--------+------+-----+-------+--------
ord_123  | NVDA   | buy  | 50  | limit | pending
ord_124  | AMD    | sell | 100 | limit | filled
```

### Cancel Orders

```bash
# Cancel specific order
sigc alpaca cancel ord_123

# Cancel all orders
sigc alpaca cancel --all
```

## Safety Features

### Position Limits

```yaml
output:
  alpaca:
    safety:
      max_position_pct: 0.05
      max_order_value: 50000
```

### Order Validation

```yaml
output:
  alpaca:
    safety:
      require_limit_price: true
      max_slippage_pct: 0.5
      check_buying_power: true
```

### Rate Limiting

```yaml
output:
  alpaca:
    rate_limits:
      orders_per_minute: 60
      requests_per_minute: 200
```

## Monitoring

### Order Status

```bash
sigc alpaca order-status ord_123
```

```
Order ord_123:
  Symbol: AAPL
  Side: buy
  Quantity: 100
  Type: limit
  Limit Price: $185.00
  Status: filled
  Filled Qty: 100
  Filled Avg Price: $184.95
  Submitted: 2024-01-15 09:30:05
  Filled: 2024-01-15 09:30:08
```

### Activity Feed

```bash
sigc alpaca activity --last 24h
```

### WebSocket Streaming

```yaml
output:
  alpaca:
    streaming:
      enabled: true
      events:
        - trade_updates
        - account_updates
```

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `insufficient_funds` | Not enough buying power | Reduce position sizes |
| `invalid_qty` | Fractional shares issue | Use whole shares |
| `market_closed` | Outside trading hours | Wait or use extended hours |
| `asset_not_tradeable` | Stock not available | Check asset status |

### Retry Logic

```yaml
output:
  alpaca:
    retry:
      max_attempts: 3
      delay_seconds: 5
      backoff: exponential
      retryable_errors:
        - connection_error
        - rate_limit
```

### Fallback Behavior

```yaml
output:
  alpaca:
    on_error:
      reject: log_and_continue  # log_and_continue | halt | alert
      insufficient_funds: reduce_size
      market_closed: queue_for_open
```

## Extended Hours

```yaml
output:
  alpaca:
    extended_hours:
      enabled: true
      pre_market: "04:00"   # 4 AM - 9:30 AM
      after_hours: "20:00"  # 4 PM - 8 PM
```

## Multi-Account

### Configure Multiple Accounts

```yaml
output:
  type: alpaca

  accounts:
    - name: main
      api_key: ${ALPACA_MAIN_KEY}
      api_secret: ${ALPACA_MAIN_SECRET}
      allocation: 0.7

    - name: secondary
      api_key: ${ALPACA_SEC_KEY}
      api_secret: ${ALPACA_SEC_SECRET}
      allocation: 0.3
```

## Example: Full Configuration

```yaml
# sigc.yaml
output:
  type: alpaca

  alpaca:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}
    paper: false  # Live trading

    orders:
      type: limit
      limit_offset_pct: 0.05
      time_in_force: day
      extended_hours: false

    execution:
      algorithm: twap
      duration_minutes: 30

    sync:
      mode: full
      reconcile: true
      close_unlisted: true

    safety:
      max_position_pct: 0.05
      max_order_value: 100000
      check_buying_power: true

    retry:
      max_attempts: 3
      delay_seconds: 5

    streaming:
      enabled: true
      events: [trade_updates]
```

## Best Practices

### 1. Start with Paper Trading

```yaml
alpaca:
  paper: true  # Always test first
```

Run paper trading for at least 30 days.

### 2. Use Limit Orders

```yaml
orders:
  type: limit
  limit_offset_pct: 0.1
```

Avoid unexpected fills.

### 3. Set Conservative Limits

```yaml
safety:
  max_position_pct: 0.03
  max_order_value: 25000
```

### 4. Monitor in Real-Time

```yaml
streaming:
  enabled: true
```

### 5. Handle Errors Gracefully

```yaml
on_error:
  reject: log_and_continue
```

## Next Steps

- [Safety Systems](../production/safety-systems.md) - Pre-trade checks
- [Monitoring](../production/monitoring.md) - Production monitoring
- [Yahoo Finance](yahoo-finance.md) - Free data source
