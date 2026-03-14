# Custom Factors

Build your own proprietary factors.

## Building Custom Factors

### Step 1: Hypothesis

Start with an economic intuition:
- "Companies with improving margins will outperform"
- "High insider buying signals confidence"
- "Low short interest means less downside"

### Step 2: Implementation

```sig
// Margin improvement factor
signal margin_improvement:
  current_margin = gross_margin
  past_margin = lag(gross_margin, 4)  // 4 quarters ago

  improvement = current_margin - past_margin

  emit zscore(improvement)
```

### Step 3: Validation

```sig
portfolio test_factor:
  weights = rank(margin_improvement).long_short(top=0.2, bottom=0.2)
  backtest walk_forward(train_years=5, test_years=1) from 2010-01-01 to 2024-12-31
```

## Example Custom Factors

### Earnings Surprise

```sig
signal earnings_surprise:
  // Actual vs estimate
  surprise = (actual_eps - estimated_eps) / abs(estimated_eps)

  emit zscore(surprise)
```

### Analyst Revision

```sig
signal revision_momentum:
  current_estimate = analyst_eps_estimate
  past_estimate = lag(analyst_eps_estimate, 21)

  revision = (current_estimate - past_estimate) / abs(past_estimate)

  emit zscore(revision)
```

### Price-to-52-Week High

```sig
signal nearness_to_high:
  high_52w = rolling_max(prices, 252)
  nearness = prices / high_52w

  // Near high = momentum, far from high = value opportunity
  emit zscore(nearness)
```

## Combining Custom Factors

```sig
signal proprietary:
  f1 = margin_improvement
  f2 = earnings_surprise
  f3 = revision_momentum

  // Equal weight or IC-weighted
  emit (f1 + f2 + f3) / 3
```

## Testing Checklist

- [ ] Economic rationale
- [ ] Out-of-sample testing
- [ ] Transaction cost analysis
- [ ] Factor correlation check
- [ ] Capacity analysis

## See Also

- [Classic Multi-Factor](classic-multi-factor.md)
- [Factor Timing](factor-timing.md)
