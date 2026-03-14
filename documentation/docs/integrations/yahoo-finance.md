# Yahoo Finance Integration

Free historical market data from Yahoo Finance.

## Overview

Yahoo Finance provides:

- Free historical prices (OHLCV)
- Dividends and splits
- Basic fundamentals
- Wide symbol coverage

## Basic Usage

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT", "GOOGL", "AMZN", "META"]
  start = "2020-01-01"
  end = "2024-12-31"
```

## Configuration

### Symbols List

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT", "GOOGL"]
  start = "2020-01-01"
```

### From File

```sig
data:
  source = "yahoo"
  symbols_file = "universe.txt"  # One symbol per line
  start = "2020-01-01"
```

### Index Components

```sig
data:
  source = "yahoo"
  index = "^GSPC"  # S&P 500 components
  start = "2020-01-01"
```

## Data Fields

### Available Fields

| Field | Description |
|-------|-------------|
| `date` | Trading date |
| `open` | Opening price |
| `high` | Daily high |
| `low` | Daily low |
| `close` | Closing price |
| `adj_close` | Adjusted close |
| `volume` | Trading volume |

### Default Mapping

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT"]
  start = "2020-01-01"
  columns:
    adj_close: Numeric as prices  # Use adjusted close
    volume: Numeric
```

### OHLCV Data

```sig
data:
  source = "yahoo"
  symbols = ["AAPL"]
  start = "2020-01-01"
  columns:
    date: Date
    symbol: Symbol as ticker
    open: Numeric
    high: Numeric
    low: Numeric
    adj_close: Numeric as close
    volume: Numeric
```

## Adjusted Prices

Yahoo provides split and dividend adjusted prices:

```sig
data:
  source = "yahoo"
  symbols = ["AAPL"]
  columns:
    adj_close: Numeric as prices  # Adjusted for splits/dividends
    close: Numeric as raw_prices  # Unadjusted
```

**Always use `adj_close` for backtesting.**

## Corporate Actions

### Dividends

```sig
data dividends:
  source = "yahoo"
  type = "dividends"
  symbols = ["AAPL", "MSFT"]
  start = "2020-01-01"
```

### Splits

```sig
data splits:
  source = "yahoo"
  type = "splits"
  symbols = ["AAPL", "MSFT"]
  start = "2020-01-01"
```

## Caching

### Enable Caching

```yaml
data:
  yahoo:
    cache:
      enabled: true
      directory: /var/cache/sigc/yahoo
      ttl_hours: 24
```

### Clear Cache

```bash
sigc cache clear --source yahoo
```

## Rate Limiting

### Default Limits

Yahoo has rate limits:

- ~2000 requests per hour
- ~100 requests per minute

### Configure Rate Limiting

```yaml
data:
  yahoo:
    rate_limit:
      requests_per_minute: 30
      retry_after_seconds: 60
```

## Error Handling

### Common Issues

| Error | Cause | Solution |
|-------|-------|----------|
| `Invalid symbol` | Symbol not found | Check ticker spelling |
| `No data` | Symbol delisted or no history | Use alternative symbol |
| `Rate limit` | Too many requests | Enable caching, reduce frequency |

### Fallback Data

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT"]
  options:
    on_error = "skip"  # skip | fail
    fallback = "cache"
```

## Large Universes

### Batch Downloading

```yaml
data:
  yahoo:
    batch_size: 50       # Download in batches
    parallel: true        # Parallel requests
    max_workers: 4
```

### Progress Tracking

```bash
sigc data fetch --source yahoo --symbols sp500.txt --progress
```

## Example Strategies

### Basic Momentum

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT", "GOOGL", "AMZN", "META",
             "NVDA", "TSLA", "JPM", "JNJ", "V"]
  start = "2020-01-01"
  columns:
    adj_close: Numeric as prices
    volume: Numeric

signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.3, bottom=0.3)
  backtest from 2020-01-01 to 2024-12-31
```

### With Volume Filter

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT", "GOOGL"]
  start = "2020-01-01"
  columns:
    adj_close: Numeric as prices
    volume: Numeric

signal momentum:
  // Filter low volume stocks
  avg_vol = rolling_mean(volume, 20)
  liquid = avg_vol > 1000000

  mom = zscore(ret(prices, 60))
  filtered = where(liquid, mom, 0)

  emit filtered
```

### Using Technical Indicators

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT", "GOOGL"]
  start = "2020-01-01"
  columns:
    adj_close: Numeric as prices
    high: Numeric
    low: Numeric

signal mean_reversion:
  // RSI signal
  rsi_val = rsi(prices, 14)
  rsi_sig = -zscore((rsi_val - 50) / 25)

  // ATR for position sizing
  atr_val = atr(high, low, prices, 14)
  size_adj = 1 / atr_val

  emit rsi_sig * zscore(size_adj)
```

## Symbol Lookup

### US Stocks

- Use ticker symbol: `AAPL`, `MSFT`
- Include exchange suffix for clarity: `AAPL.US`

### International

- UK: `VOD.L` (London)
- Germany: `BMW.DE` (Frankfurt)
- Japan: `7203.T` (Tokyo)

### ETFs

- SPY, QQQ, IWM, VTI
- International: EFA, EEM, VEA

### Indices

- S&P 500: `^GSPC`
- Dow Jones: `^DJI`
- Nasdaq: `^IXIC`
- VIX: `^VIX`

## Limitations

### Data Quality

- May have occasional errors
- Historical data can change
- Corporate actions may be delayed

### Coverage

- US stocks: Excellent
- International: Good
- OTC/Pink sheets: Limited

### Real-time

- 15-20 minute delay for quotes
- Not suitable for live trading data

## Best Practices

### 1. Use Adjusted Close

```sig
columns:
  adj_close: Numeric as prices  # Always
```

### 2. Enable Caching

```yaml
cache:
  enabled: true
  ttl_hours: 24
```

### 3. Verify Data Quality

```sig
signal quality_check:
  // Check for suspicious data
  daily_ret = ret(prices, 1)
  suspicious = abs(daily_ret) > 0.5

  emit where(not(suspicious), prices, 0)
```

### 4. Handle Missing Symbols

```yaml
options:
  on_error = "skip"
```

### 5. Use for Research, Not Production

For production, use a professional data provider.

## Python Integration

```python
import pysigc
import yfinance as yf

# Fetch data with yfinance
tickers = ["AAPL", "MSFT", "GOOGL"]
data = yf.download(tickers, start="2020-01-01")

# Convert to sigc format
prices = data["Adj Close"].reset_index().melt(
    id_vars=["Date"],
    var_name="ticker",
    value_name="prices"
)

# Run strategy with custom data
results = pysigc.run("strategy.sig", data={"prices": prices})
```

## Next Steps

- [CSV Format](../data/csv.md) - Using local CSV files
- [Parquet Format](../data/parquet.md) - Faster data loading
- [Streaming](streaming.md) - Real-time data
