# Sample Data

This guide covers working with sample data for sigc development and testing.

## Included Sample Data

The documentation includes sample price data you can use for testing:

### Download Sample Data

```bash
# From the docs assets
curl -o data/prices.csv https://docs.skelfresearch.com/sigc/assets/sample-data/prices.csv
```

Or copy from the repository:

```bash
cp /path/to/sigc/documentation/docs/assets/sample-data/prices.csv data/
```

### Sample Data Format

The included `prices.csv` contains:

- **Period**: January - February 2024 (40 trading days)
- **Assets**: 10 stocks (AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA, JPM, V, JNJ)
- **Fields**: Date and adjusted close prices

```csv
date,AAPL,MSFT,GOOGL,AMZN,META,NVDA,TSLA,JPM,V,JNJ
2024-01-02,185.64,374.58,140.25,151.94,346.29,481.68,248.42,170.10,260.35,156.74
2024-01-03,184.25,373.31,139.12,149.93,344.47,477.80,246.98,169.62,258.87,155.89
...
```

## Data Format Requirements

### CSV Files

sigc expects CSV files with:

- **First row**: Column headers
- **First column**: Date (YYYY-MM-DD format)
- **Remaining columns**: Asset prices

```csv
date,ASSET1,ASSET2,ASSET3
2024-01-02,100.00,200.00,300.00
2024-01-03,101.00,199.00,302.00
```

### Supported Date Formats

- `YYYY-MM-DD` (preferred): `2024-01-15`
- `YYYY/MM/DD`: `2024/01/15`
- `MM/DD/YYYY`: `01/15/2024`

### Missing Data

sigc handles missing data (NaN) automatically. Use empty cells or literal "NaN":

```csv
date,AAPL,MSFT
2024-01-02,185.64,374.58
2024-01-03,,373.31
2024-01-04,181.91,
```

Use `fill_nan` to handle missing values in your signals:

```sig
signal cleaned:
  filled = fill_nan(prices, 0)
  emit filled
```

## Creating Your Own Data

### From Yahoo Finance

Using Python:

```python
import yfinance as yf
import pandas as pd

# Download data
tickers = ['AAPL', 'MSFT', 'GOOGL', 'AMZN', 'META']
data = yf.download(tickers, start='2020-01-01', end='2024-01-01')

# Extract adjusted close prices
prices = data['Adj Close']

# Save to CSV
prices.to_csv('data/prices.csv')
```

### From Alpaca

```python
from alpaca.data import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame

client = StockHistoricalDataClient('api_key', 'secret_key')

request = StockBarsRequest(
    symbol_or_symbols=['AAPL', 'MSFT', 'GOOGL'],
    timeframe=TimeFrame.Day,
    start='2020-01-01',
    end='2024-01-01'
)

bars = client.get_stock_bars(request)
df = bars.df.unstack(level=0)['close']
df.to_csv('data/prices.csv')
```

### Generate Synthetic Data

For testing, generate random price data:

```python
import pandas as pd
import numpy as np

np.random.seed(42)

dates = pd.date_range('2024-01-01', periods=252, freq='B')
assets = ['STOCK_A', 'STOCK_B', 'STOCK_C', 'STOCK_D', 'STOCK_E']

# Generate random returns
returns = np.random.normal(0.0005, 0.02, (252, 5))

# Convert to prices
prices = 100 * np.cumprod(1 + returns, axis=0)

df = pd.DataFrame(prices, index=dates, columns=assets)
df.index.name = 'date'
df.to_csv('data/synthetic_prices.csv')
```

## Data Sources for Production

### Free Data Sources

| Source | Data Type | Frequency | Notes |
|--------|-----------|-----------|-------|
| Yahoo Finance | Prices, volumes | Daily | Free, reliable |
| FRED | Economic data | Various | US economic indicators |
| Quandl (free tier) | Various | Daily | Limited free access |
| Alpha Vantage | Prices | Daily | API key required |

### Commercial Data Sources

| Source | Data Type | Notes |
|--------|-----------|-------|
| Bloomberg | Everything | Industry standard |
| Refinitiv | Prices, fundamentals | Good corporate data |
| IEX Cloud | US equities | Developer-friendly |
| Polygon.io | US equities | Real-time available |
| Alpaca | US equities | Free with brokerage |

## Loading Data in sigc

### CSV Files

```sig
data:
  prices: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"
```

### Parquet Files

Parquet is more efficient for large datasets:

```sig
data:
  prices: load parquet from "data/prices.parquet"
```

Convert CSV to Parquet using Python:

```python
import pandas as pd

df = pd.read_csv('data/prices.csv', index_col=0, parse_dates=True)
df.to_parquet('data/prices.parquet')
```

### S3 Data

Load directly from AWS S3:

```sig
data:
  prices: load parquet from "s3://my-bucket/prices.parquet"
```

Set credentials via environment variables:

```bash
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
```

### Multiple Data Sources

Combine multiple data files:

```sig
data:
  prices: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"
  market_cap: load parquet from "data/fundamentals.parquet"
  sectors: load csv from "data/sectors.csv"
```

## Data Quality

### Validation

Before using data, check for issues:

```bash
# Quick Python check
python -c "
import pandas as pd
df = pd.read_csv('data/prices.csv', index_col=0, parse_dates=True)
print(f'Shape: {df.shape}')
print(f'Date range: {df.index[0]} to {df.index[-1]}')
print(f'Missing values:\n{df.isna().sum()}')
print(f'Sample:\n{df.head()}')
"
```

### Common Issues

| Issue | Solution |
|-------|----------|
| Missing dates | Fill forward or interpolate |
| Missing values | Use `fill_nan` or `coalesce` |
| Outliers | Use `winsor` to clip extremes |
| Stale data | Check data freshness before backtesting |
| Survivorship bias | Include delisted securities |

### Corporate Actions

For accurate backtesting, adjust for:

- **Splits**: Divide historical prices by split ratio
- **Dividends**: Use adjusted close prices
- **Mergers**: Handle symbol changes

Yahoo Finance's "Adj Close" already handles splits and dividends.

## Example: Multi-Asset Strategy

Using the sample data:

```sig
// Multi-asset momentum strategy using sample data

data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20
  vol_window = 60

signal momentum:
  // Compute returns
  returns = ret(prices, lookback)

  // Volatility adjustment
  vol = rolling_std(ret(prices, 1), vol_window)
  vol_adj = returns / vol

  // Normalize
  emit zscore(vol_adj)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-16 to 2024-02-29
```

## Next Steps

- [DSL Basics](../language/syntax.md) - Learn the full language syntax
- [Data Loading](../data/index.md) - Advanced data loading options
- [Tutorials](../tutorials/index.md) - Build more strategies
