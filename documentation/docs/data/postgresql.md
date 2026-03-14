# PostgreSQL Integration

Load data directly from PostgreSQL databases.

## Basic Usage

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = "SELECT date, ticker, close, volume FROM daily_prices"
```

## Connection String

### Format

```
postgresql://[user[:password]@]host[:port]/database[?options]
```

### Examples

```sig
// Local database
source = "postgresql://localhost/marketdb"

// With credentials
source = "postgresql://user:password@localhost/marketdb"

// Custom port
source = "postgresql://localhost:5433/marketdb"

// SSL required
source = "postgresql://user:password@host/db?sslmode=require"
```

## Authentication

### Connection String (Development)

```sig
data:
  source = "postgresql://user:password@localhost/marketdb"
  query = "..."
```

### Environment Variables (Recommended)

```bash
export PGUSER=your_user
export PGPASSWORD=your_password
export PGHOST=localhost
export PGDATABASE=marketdb
```

```sig
data:
  source = "postgresql:///marketdb"  # Uses env vars
  query = "..."
```

### Password File (~/.pgpass)

```
# ~/.pgpass
hostname:port:database:username:password
```

## Query Syntax

### Basic Query

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = "SELECT date, ticker, close, volume FROM daily_prices"
```

### With Filters

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = """
    SELECT date, ticker, close, volume
    FROM daily_prices
    WHERE date >= '2020-01-01'
      AND ticker IN ('AAPL', 'MSFT', 'GOOGL')
    ORDER BY date, ticker
  """
```

### With Joins

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = """
    SELECT
      p.date,
      p.ticker,
      p.close,
      p.volume,
      f.pe_ratio,
      f.book_value
    FROM daily_prices p
    LEFT JOIN fundamentals f
      ON p.ticker = f.ticker
      AND p.date = f.date
    ORDER BY p.date, p.ticker
  """
```

### Using Parameters

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = """
    SELECT date, ticker, close, volume
    FROM daily_prices
    WHERE date BETWEEN $1 AND $2
  """
  options:
    params = ["2020-01-01", "2024-12-31"]
```

## Column Mapping

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = "SELECT trade_date, symbol, adj_close, shares_traded FROM prices"
  columns:
    trade_date: Date
    symbol: Symbol
    adj_close: Numeric as prices
    shares_traded: Numeric as volume
```

## Connection Options

### SSL Mode

```sig
data:
  source = "postgresql://host/db"
  query = "..."
  options:
    sslmode = "require"  # disable, allow, prefer, require, verify-ca, verify-full
```

### Connection Pool

```sig
data:
  source = "postgresql://host/db"
  query = "..."
  options:
    pool_size = 5
    pool_timeout = 30
```

### Statement Timeout

```sig
data:
  source = "postgresql://host/db"
  query = "..."
  options:
    statement_timeout = 60000  # 60 seconds in milliseconds
```

## Performance

### Index Usage

Ensure your tables have appropriate indexes:

```sql
-- Recommended indexes for market data
CREATE INDEX idx_prices_date ON daily_prices(date);
CREATE INDEX idx_prices_ticker ON daily_prices(ticker);
CREATE INDEX idx_prices_date_ticker ON daily_prices(date, ticker);
```

### Partitioned Tables

For large datasets, use PostgreSQL partitioning:

```sql
-- Partition by date range
CREATE TABLE daily_prices (
    date DATE NOT NULL,
    ticker TEXT NOT NULL,
    close NUMERIC,
    volume BIGINT
) PARTITION BY RANGE (date);

CREATE TABLE daily_prices_2023 PARTITION OF daily_prices
    FOR VALUES FROM ('2023-01-01') TO ('2024-01-01');
```

### Query Optimization

```sig
// Use specific columns
query = "SELECT date, ticker, close FROM prices"  // Not SELECT *

// Add date filters
query = "SELECT ... WHERE date >= '2020-01-01'"

// Use LIMIT for testing
query = "SELECT ... LIMIT 1000"
```

### Parallel Queries

```sig
data:
  source = "postgresql://host/db"
  query = "..."
  options:
    parallel_workers = 4  # Enable parallel query
```

## Multiple Data Sources

### Prices and Fundamentals

```sig
data prices:
  source = "postgresql://localhost/marketdb"
  query = "SELECT date, ticker, close FROM daily_prices"
  columns:
    close: Numeric as prices

data fundamentals:
  source = "postgresql://localhost/marketdb"
  query = "SELECT date, ticker, pe_ratio, book_value FROM fundamentals"

signal combined:
  momentum = zscore(ret(prices.prices, 60))
  value = zscore(fundamentals.book_value)
  emit 0.5 * momentum + 0.5 * value
```

### Read Replica

```sig
data:
  source = "postgresql://read-replica.host/marketdb"
  query = "..."
  options:
    target_session_attrs = "read-only"
```

## TimescaleDB Support

sigc works with TimescaleDB hypertables:

```sql
-- Create hypertable
SELECT create_hypertable('daily_prices', 'date');
```

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = """
    SELECT time_bucket('1 day', date) AS date,
           ticker,
           last(close, date) AS close,
           sum(volume) AS volume
    FROM daily_prices
    WHERE date >= '2020-01-01'
    GROUP BY 1, 2
    ORDER BY 1, 2
  """
```

## Error Handling

### Connection Failed

```
Error: connection refused
```

Check:

1. PostgreSQL is running
2. Host/port are correct
3. Firewall allows connection

### Authentication Failed

```
Error: password authentication failed
```

Check:

1. Username/password correct
2. User has database access
3. pg_hba.conf allows connection

### Query Timeout

```
Error: canceling statement due to statement timeout
```

Increase timeout:

```sig
options:
  statement_timeout = 120000  # 2 minutes
```

### Permission Denied

```
Error: permission denied for table
```

Grant access:

```sql
GRANT SELECT ON daily_prices TO your_user;
```

## Best Practices

### 1. Use Read Replicas for Analysis

```sig
// Don't query production primary for backtesting
source = "postgresql://read-replica/db"
```

### 2. Add Date Filters

```sig
query = """
  SELECT date, ticker, close, volume
  FROM daily_prices
  WHERE date >= '2020-01-01'  -- Always filter!
"""
```

### 3. Use Prepared Statements

```sig
options:
  prepare = true
```

### 4. Set Reasonable Timeouts

```sig
options:
  statement_timeout = 60000
  connect_timeout = 10
```

### 5. Close Connections

sigc automatically manages connection pooling and cleanup.

## Example: Complete Setup

```sig
data:
  source = "postgresql://analytics:${PGPASSWORD}@db.example.com:5432/marketdb"
  query = """
    SELECT
      p.date,
      p.ticker,
      p.adjusted_close AS close,
      p.volume,
      s.sector,
      f.pe_ratio,
      f.book_to_market
    FROM prices p
    JOIN securities s ON p.ticker = s.ticker
    LEFT JOIN fundamentals f
      ON p.ticker = f.ticker
      AND p.date = f.date
    WHERE p.date >= '2015-01-01'
    ORDER BY p.date, p.ticker
  """
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric
    sector: String as sectors
    pe_ratio: Numeric
    book_to_market: Numeric
  options:
    sslmode = "require"
    statement_timeout = 120000

signal momentum:
  raw = zscore(ret(prices, 60))
  neutral = neutralize(raw, by=sectors)
  emit neutral

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Next Steps

- [CSV Format](csv.md) - File-based loading
- [S3 Storage](s3.md) - Cloud storage
- [Data Quality](data-quality.md) - Validating your data
