# Types Module

Core type definitions.

## Numeric Types

### Scalar

```rust
pub type Scalar = f64;
```

### Series

```rust
pub struct Series {
    values: Vec<f64>,
    name: String,
}

impl Series {
    pub fn new(name: &str, values: Vec<f64>) -> Self;
    pub fn len(&self) -> usize;
    pub fn get(&self, index: usize) -> Option<f64>;
    pub fn sum(&self) -> f64;
    pub fn mean(&self) -> f64;
    pub fn std(&self) -> f64;
}
```

### Matrix

```rust
pub struct Matrix {
    data: Vec<Vec<f64>>,
    rows: usize,
    cols: usize,
}
```

## Date Types

### Date

```rust
use chrono::NaiveDate;

pub type Date = NaiveDate;
```

### DateRange

```rust
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self;
    pub fn contains(&self, date: NaiveDate) -> bool;
    pub fn days(&self) -> i64;
}
```

## Portfolio Types

### Weight

```rust
pub struct Weight {
    pub date: NaiveDate,
    pub symbol: String,
    pub weight: f64,
}
```

### Position

```rust
pub struct Position {
    pub date: NaiveDate,
    pub symbol: String,
    pub quantity: f64,
    pub value: f64,
}
```

### Trade

```rust
pub struct Trade {
    pub date: NaiveDate,
    pub symbol: String,
    pub side: TradeSide,
    pub quantity: f64,
    pub price: f64,
    pub cost: f64,
}

pub enum TradeSide {
    Buy,
    Sell,
}
```

## Signal Types

### Signal

```rust
pub struct Signal {
    pub name: String,
    pub values: DataFrame,
}
```

### SignalValue

```rust
pub struct SignalValue {
    pub date: NaiveDate,
    pub symbol: String,
    pub value: f64,
}
```

## Configuration Types

### Config

```rust
pub struct Config {
    pub data: DataConfig,
    pub performance: PerformanceConfig,
    pub risk: RiskConfig,
    pub daemon: Option<DaemonConfig>,
}
```

### DataConfig

```rust
pub struct DataConfig {
    pub source: String,
    pub format: DataFormat,
    pub columns: HashMap<String, ColumnType>,
}

pub enum DataFormat {
    Csv,
    Parquet,
    Arrow,
    Sql,
}
```

### RiskConfig

```rust
pub struct RiskConfig {
    pub max_position: f64,
    pub max_sector: f64,
    pub gross_exposure: f64,
    pub net_exposure: (f64, f64),
}
```

## Error Types

### Error

```rust
pub enum Error {
    ParseError(ParseError),
    TypeError(TypeError),
    DataError(DataError),
    RuntimeError(RuntimeError),
    ConfigError(ConfigError),
    IoError(std::io::Error),
}
```

### ParseError

```rust
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub code: String,
}
```

## Result Types

### Result

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Constraint Types

### Constraint

```rust
pub enum Constraint {
    MaxPosition(f64),
    MaxSector(f64),
    GrossExposure(f64),
    NetExposure(f64, f64),
    MaxTurnover(f64),
    Beta(f64, f64),
}
```

## See Also

- [Strategy Module](strategy.md)
- [Results Module](results.md)
- [Data Module](data.md)
