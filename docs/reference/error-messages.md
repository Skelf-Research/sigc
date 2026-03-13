# Error Messages Guide

Common errors and how to fix them.

## Parse Errors

### "Unexpected token"

**Error:**
```
Parse error: Unexpected token at line 5
```

**Cause:** Syntax error in DSL.

**Fix:** Check syntax:
```
# Wrong
signal test
  emit x

# Correct
signal test:
  emit x
```

### "Expected expression"

**Cause:** Missing value in assignment.

**Fix:**
```
# Wrong
x =

# Correct
x = ret(prices, 20)
```

## Type Errors

### "Undefined identifier"

**Error:**
```
Type error: Undefined identifier 'pricse' at line 8
```

**Cause:** Typo or undefined variable.

**Fix:** Check spelling matches data/param declaration:
```
data:
  prices: load csv from "data.csv"  # Note: 'prices'

signal test:
  r = ret(prices, 20)  # Use 'prices', not 'pricse'
```

### "Type mismatch"

**Cause:** Incompatible types in operation.

**Fix:** Ensure operands match (both numeric, both series).

## Runtime Errors

### "Operator 'X' error: Input count mismatch"

**Error:**
```
Operator 'Mul' error (node #5): Input count mismatch
  Expected 2 input(s), got 1
  Suggestion: 'Mul' requires 2 inputs but only 1 provided.
```

**Cause:** Operator received wrong number of inputs.

**Fix:** Check your expression has correct arguments:
```
# Wrong - missing operand
x = prices *

# Correct
x = prices * volume
```

### "Missing input node"

**Cause:** Reference to undefined computation.

**Fix:** Ensure all variables are defined before use:
```
signal test:
  # x not defined yet!
  y = zscore(x)  # Error

  # Correct order:
  x = ret(prices, 20)
  y = zscore(x)
```

### "Division failed"

**Cause:** Division by zero or NaN.

**Fix:** Add guards:
```
signal safe:
  vol = rolling_std(ret(prices, 1), 20)
  # Avoid division by zero
  safe_vol = where(vol > 0.001, vol, 0.001)
  emit ret(prices, 20) / safe_vol
```

## Data Errors

### "Failed to load CSV"

**Causes:**
- File doesn't exist
- Wrong path
- Permission denied

**Fix:**
```bash
# Check file exists
ls -la data/prices.csv

# Check path in sig file
data:
  prices: load csv from "data/prices.csv"  # Relative to cwd
```

### "Column not found"

**Cause:** CSV missing expected column.

**Fix:** Ensure CSV has `date` column and asset columns:
```csv
date,AAPL,MSFT
2024-01-02,185.64,374.58
```

### "Cast failed"

**Cause:** Non-numeric data in price column.

**Fix:** Clean CSV - ensure all price values are numbers.

## Common Fixes

### Check File Paths

Paths are relative to current working directory:
```bash
pwd  # Check you're in project root
ls data/prices.csv  # Verify file exists
```

### Check Data Format

Required CSV format:
```csv
date,ASSET1,ASSET2,...
2024-01-02,100.0,200.0,...
```

### Check Lookback Periods

Ensure enough data for lookback:
```
# If lookback=60, need 60+ rows of data
r = ret(prices, 60)
```

### Check Parameter Types

Parameters must be numeric:
```
params:
  lookback = 20      # Correct: integer
  threshold = 0.5    # Correct: float
  # name = "test"    # Wrong: string not supported
```

## Getting Help

1. Run with verbose logging:
   ```bash
   sigc -v run strategy.sig
   ```

2. Check IR structure:
   ```bash
   sigc explain strategy.sig
   ```

3. Simplify to isolate error:
   ```
   signal debug:
     x = ret(prices, 20)
     emit x  # Test each step
   ```
