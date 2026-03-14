# Error Messages Reference

Common errors and how to resolve them.

## Parse Errors

### E001: Unexpected Token

```
Error E001: Unexpected token 'emit' at line 5
```

**Cause**: Syntax error in sigc code.

**Solution**: Check the line for missing punctuation or keywords.

### E002: Unknown Identifier

```
Error E002: Unknown identifier 'prces' at line 8
```

**Cause**: Typo in variable or function name.

**Solution**: Check spelling of identifiers.

### E003: Missing Colon

```
Error E003: Expected ':' after section name at line 3
```

**Cause**: Section declaration missing colon.

**Solution**: Add colon after `data`, `signal`, or `portfolio`.

## Type Errors

### E100: Type Mismatch

```
Error E100: Cannot multiply Numeric by Date at line 12
```

**Cause**: Operation on incompatible types.

**Solution**: Ensure operands have compatible types.

### E101: Shape Mismatch

```
Error E101: Cannot add scalar to cross-sectional array at line 15
```

**Cause**: Operations require matching shapes.

**Solution**: Use appropriate broadcasting or aggregation.

### E102: Missing Emit

```
Error E102: Signal 'momentum' does not emit a value
```

**Cause**: Signal block must end with `emit`.

**Solution**: Add `emit expression` at end of signal.

## Data Errors

### E200: File Not Found

```
Error E200: Data file not found: prices.csv
```

**Cause**: Data file doesn't exist at specified path.

**Solution**: Check file path and existence.

### E201: Invalid Format

```
Error E201: Cannot parse 'abc' as Numeric in column 'close' at row 45
```

**Cause**: Data contains invalid values.

**Solution**: Clean data or specify correct column types.

### E202: Missing Column

```
Error E202: Required column 'close' not found
```

**Cause**: Data file missing expected column.

**Solution**: Check column names in data file.

## Runtime Errors

### E300: Division by Zero

```
Error E300: Division by zero at line 10: vol_ratio = ret / vol
```

**Cause**: Dividing by zero value.

**Solution**: Add small epsilon or filter zeros.

```sig
// Fix
vol_ratio = ret / (vol + 0.0001)
```

### E301: NaN Propagation

```
Warning E301: NaN values detected in signal 'momentum' (15% of values)
```

**Cause**: Calculations producing NaN.

**Solution**: Check for log of negative, sqrt of negative, etc.

### E302: Insufficient Data

```
Error E302: Insufficient data for rolling_std(60): only 45 rows
```

**Cause**: Not enough data for lookback window.

**Solution**: Reduce window or get more data.

## Configuration Errors

### E400: Invalid YAML

```
Error E400: Invalid YAML in config.yaml at line 15
```

**Cause**: Syntax error in configuration file.

**Solution**: Check YAML syntax.

### E401: Missing Required Field

```
Error E401: Missing required field 'source' in data section
```

**Cause**: Configuration missing required parameter.

**Solution**: Add the required field.

### E402: Invalid Value

```
Error E402: Invalid value for 'workers': expected integer, got 'auto'
```

**Cause**: Configuration value wrong type.

**Solution**: Check expected type in documentation.

## Backtest Errors

### E500: Date Range Error

```
Error E500: Start date 2025-01-01 is after end date 2024-12-31
```

**Cause**: Invalid date range.

**Solution**: Fix date order.

### E501: No Data in Range

```
Error E501: No data found between 2010-01-01 and 2015-01-01
```

**Cause**: Data doesn't cover specified period.

**Solution**: Adjust date range or get more data.

### E502: Invalid Constraint

```
Error E502: max_position (0.5) exceeds gross_exposure (0.3)
```

**Cause**: Contradictory constraints.

**Solution**: Fix constraint values.

## Broker Errors

### E600: Connection Failed

```
Error E600: Failed to connect to broker: timeout after 30s
```

**Cause**: Network or authentication issue.

**Solution**: Check credentials and network.

### E601: Order Rejected

```
Error E601: Order rejected: insufficient buying power
```

**Cause**: Broker rejected order.

**Solution**: Check account balance and order parameters.

### E602: Invalid Symbol

```
Error E602: Symbol 'AAPL.OLD' not found
```

**Cause**: Invalid or delisted ticker.

**Solution**: Use correct symbol.

## Warning Messages

### W001: Deprecated Feature

```
Warning W001: 'old_function' is deprecated, use 'new_function' instead
```

**Action**: Update code to use new feature.

### W002: High Turnover

```
Warning W002: Annual turnover (850%) exceeds recommended maximum (500%)
```

**Action**: Consider reducing rebalancing frequency.

### W003: Concentration

```
Warning W003: Single position (AAPL) exceeds 10% of portfolio
```

**Action**: Add position limits.

## Getting Help

If you encounter an error not listed here:

1. Check the [documentation](../index.md)
2. Search [GitHub Issues](https://github.com/skelf-Research/sigc/issues)
3. Open a new issue with:
   - Error message
   - Minimal reproducing example
   - sigc version
