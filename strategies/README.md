# sigc Strategy Library

A collection of example quantitative trading strategies demonstrating sigc's capabilities.

## Categories

### Momentum Strategies
- **time_series_momentum.sig** - Classic time-series momentum (12-1 month)
- **cross_sectional_momentum.sig** - Cross-sectional momentum with industry neutralization
- **momentum_crash_protected.sig** - Momentum with crash protection via dynamic hedging
- **momentum_quality.sig** - Momentum filtered by quality metrics

### Mean Reversion Strategies
- **short_term_reversal.sig** - 5-day reversal strategy
- **pairs_trading.sig** - Statistical pairs trading via cointegration
- **bollinger_mean_reversion.sig** - Bollinger band based mean reversion
- **sector_rotation.sig** - Sector mean reversion

### Volatility Strategies
- **low_volatility.sig** - Low volatility anomaly
- **volatility_timing.sig** - Volatility regime timing
- **variance_risk_premium.sig** - Variance risk premium harvesting
- **vol_of_vol.sig** - Volatility of volatility trading

### Multi-Factor Strategies
- **value_momentum.sig** - Value + momentum combination
- **fama_french.sig** - Fama-French style factor model
- **quality_value_momentum.sig** - Three-factor combination
- **defensive_equity.sig** - Low vol + quality + value

### Technical Strategies
- **trend_following.sig** - Moving average crossover
- **rsi_strategy.sig** - RSI-based mean reversion
- **macd_momentum.sig** - MACD signal strategy
- **breakout.sig** - Price breakout strategy

### Statistical Arbitrage
- **residual_momentum.sig** - Momentum on market-residualized returns
- **industry_momentum.sig** - Industry-level momentum
- **factor_timing.sig** - Dynamic factor allocation

## Usage

```bash
# Compile a strategy
sigc compile strategies/momentum/time_series_momentum.sig

# Run backtest
sigc run strategies/momentum/time_series_momentum.sig

# Compare strategies
sigc diff strategies/momentum/time_series_momentum.sig strategies/mean_reversion/short_term_reversal.sig
```

## Data Requirements

Most strategies expect price data in the standard format:
- Panel data with `date` column and asset columns (AAPL, MSFT, etc.)
- Adjusted for splits and dividends when specified
- CSV or Parquet format

## Contributing

When adding new strategies:
1. Include clear comments explaining the strategy logic
2. Define all parameters with sensible defaults
3. Use meaningful signal names
4. Include realistic transaction costs
