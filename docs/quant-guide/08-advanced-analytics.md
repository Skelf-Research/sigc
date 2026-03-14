# Chapter 8: Advanced Analytics

This chapter covers sophisticated quantitative methods for risk analysis, regime detection, and portfolio optimization.

## Factor Models

Factor models decompose returns into systematic (factor) and idiosyncratic components.

### Fama-French Models

The Fama-French model explains returns using market, size, and value factors.

**3-Factor Model**:
```
R - Rf = α + β₁(Rm - Rf) + β₂(SMB) + β₃(HML) + ε
```

Where:
- **Rm - Rf**: Market excess return
- **SMB**: Small Minus Big (size factor)
- **HML**: High Minus Low (value factor)

```rust
use sig_runtime::{FamaFrench, FactorExposures};

// Load factor data
let market = load_series("market_excess.csv");
let smb = load_series("smb.csv");
let hml = load_series("hml.csv");
let rf = load_series("risk_free.csv");

// Create model
let ff = FamaFrench::three_factor(market, smb, hml, rf);

// Calculate exposures
let exposures = ff.calculate_exposures(&returns, 60)?;

// Decompose returns
let decomposition = ff.decompose_returns(&returns, &exposures)?;
```

**5-Factor Model** adds:
- **RMW**: Robust Minus Weak (profitability)
- **CMA**: Conservative Minus Aggressive (investment)

### Barra-Style Risk Models

Barra models use multiple style and industry factors.

```rust
use sig_runtime::BarraModel;

let mut model = BarraModel::new();

// Add style factors
model.add_common_style_factors(
    momentum,
    value,
    size,
    volatility,
    quality,
    growth,
);

// Add industry factors
model.add_industry_factor("technology", tech_exposure);
model.add_industry_factor("healthcare", health_exposure);

// Calculate portfolio exposures
let exposures = model.portfolio_exposures(&weights)?;

// Calculate factor risk
let factor_risk = model.portfolio_factor_risk(&weights)?;
```

### Custom Factor Construction

Build your own factors:

```rust
use sig_runtime::FactorBuilder;

let factors = FactorBuilder::new()
    .momentum_factor(&prices, 60)?    // 60-day momentum
    .value_factor(&prices, &earnings)? // E/P ratio
    .size_factor(&market_cap)?         // Log market cap
    .volatility_factor(&returns, 20)?  // 20-day realized vol
    .build();

// Analyze factor performance
let analysis = analyze_factors(&factors, 252)?;
println!("Factor Sharpe Ratios: {:?}", analysis.factor_sharpe);
```

## Risk Models

### Value at Risk (VaR)

VaR estimates the maximum loss at a given confidence level.

```rust
use sig_runtime::VaRCalculator;

let var_calc = VaRCalculator::new(0.95, 252);

// Historical VaR (empirical quantile)
let hist_var = var_calc.historical_var(&returns)?;

// Parametric VaR (assumes normality)
let param_var = var_calc.parametric_var(&returns)?;

// Cornish-Fisher VaR (adjusts for skewness/kurtosis)
let cf_var = var_calc.cornish_fisher_var(&returns)?;

// Rolling VaR
let rolling_var = var_calc.rolling_var(&returns)?;
```

**Interpretation**: "With 95% confidence, daily losses will not exceed VaR."

### Expected Shortfall (CVaR)

CVaR is the average loss when losses exceed VaR.

```rust
use sig_runtime::CVaRCalculator;

let cvar_calc = CVaRCalculator::new(0.95);
let cvar = cvar_calc.historical_cvar(&returns)?;

// CVaR is always >= VaR
assert!(cvar >= var);
```

**Why CVaR?**: It's coherent (satisfies subadditivity) and captures tail risk better than VaR.

### Comprehensive Risk Report

Generate a full risk analysis:

```rust
use sig_runtime::generate_risk_report;

let report = generate_risk_report(&returns, Some(&factor_exposures))?;

println!("VaR (95%): {:.2}%", report.var_95 * 100.0);
println!("VaR (99%): {:.2}%", report.var_99 * 100.0);
println!("CVaR (95%): {:.2}%", report.cvar_95 * 100.0);
println!("Volatility: {:.2}%", report.volatility * 100.0);
println!("Max Drawdown: {:.2}%", report.max_drawdown * 100.0);

// Stress test results
for (scenario, impact) in &report.stress_results {
    println!("{}: {:.2}%", scenario, impact * 100.0);
}
```

## Stress Testing

Test portfolio against historical and hypothetical scenarios.

### Historical Scenarios

```rust
use sig_runtime::{StressTest, StressScenario};

let mut stress = StressTest::new();
stress.add_historical_scenarios(); // 2008, 2020, 2022

// Apply scenarios
let results = stress.run_all(&factor_exposures);

for (name, impact) in results {
    println!("{}: {:.1}%", name, impact * 100.0);
}
```

### Custom Scenarios

```rust
let mut custom_shocks = HashMap::new();
custom_shocks.insert("market".to_string(), -0.25);
custom_shocks.insert("volatility".to_string(), 2.5);
custom_shocks.insert("growth".to_string(), -0.40);

let scenario = StressScenario {
    name: "tech_crash".to_string(),
    description: "Technology sector crash".to_string(),
    factor_shocks: custom_shocks,
    return_shock: Some(-0.30),
};

stress.add_scenario(scenario);
let impact = stress.apply_scenario("tech_crash", &exposures)?;
```

### Marginal and Component VaR

Understand risk contributions:

```rust
use sig_runtime::{MarginalVaR, component_var};

let marginal_var = MarginalVaR::new(0.95);
let marginal = marginal_var.calculate(&weights, &returns_matrix, portfolio_var)?;

// Component VaR = weight × marginal VaR
let component = component_var(&weights, &marginal);

// Sum of component VaR = total VaR
let total: f64 = component.iter().sum();
```

## Regime Detection

Identify market states to adapt strategy behavior.

### Hidden Markov Models

HMMs model markets as switching between hidden states.

```rust
use sig_runtime::HiddenMarkovModel;

// Create bull/bear model
let mut hmm = HiddenMarkovModel::bull_bear();

// Fit to historical data
hmm.fit(&returns, 100)?;

// Predict states
let states = hmm.predict(&returns)?;

// Get state probabilities
let probs = hmm.state_probabilities_df(&returns)?;
```

**State interpretation**:
- State 0: Bull market (positive mean, low vol)
- State 1: Bear market (negative mean, high vol)

### Volatility Regimes

Simple rule-based regime detection:

```rust
use sig_runtime::VolatilityRegime;

let detector = VolatilityRegime::new(20, 60);
let regimes = detector.detect(&returns)?;

// 0 = Low vol, 1 = Normal, 2 = High vol
let labels = detector.detect_labeled(&returns)?;
```

### Trend Regimes

Detect trending vs ranging markets:

```rust
use sig_runtime::TrendRegime;

let detector = TrendRegime::new(20, 60);
let regimes = detector.detect(&prices)?;

// -1 = Downtrend, 0 = Neutral, 1 = Uptrend
```

### Combined Regime Detection

Use both volatility and trend:

```rust
use sig_runtime::{RegimeDetector, MarketRegime};

let detector = RegimeDetector::new();
let regimes = detector.detect(&prices, &returns)?;

for regime in &regimes {
    match regime {
        MarketRegime::BullQuiet => println!("Bull (Quiet)"),
        MarketRegime::BullVolatile => println!("Bull (Volatile)"),
        MarketRegime::BearQuiet => println!("Bear (Quiet)"),
        MarketRegime::BearVolatile => println!("Bear (Volatile)"),
        MarketRegime::Neutral => println!("Neutral"),
    }
}
```

### K-Means Clustering

Data-driven regime identification:

```rust
use sig_runtime::KMeansRegime;

let mut kmeans = KMeansRegime::new(3); // 3 regimes
kmeans.fit(&returns, 20, 100)?;

let clusters = kmeans.predict(&returns, 20)?;
```

## Portfolio Optimization

### Mean-Variance Optimization

Classic Markowitz optimization:

```rust
use sig_runtime::{MeanVarianceOptimizer, PortfolioConstraints};

let optimizer = MeanVarianceOptimizer::new(0.02); // 2% risk-free rate

// Minimum variance portfolio
let min_var = optimizer.minimum_variance(&assets, &covariance)?;

// Maximum Sharpe ratio portfolio
let max_sharpe = optimizer.max_sharpe(&assets, &expected_returns, &covariance)?;

println!("Optimal weights: {:?}", max_sharpe.weights);
println!("Expected return: {:.2}%", max_sharpe.expected_return * 100.0);
println!("Volatility: {:.2}%", max_sharpe.expected_volatility * 100.0);
println!("Sharpe ratio: {:.2}", max_sharpe.sharpe_ratio);
```

### Constrained Optimization

Add realistic constraints:

```rust
let constraints = PortfolioConstraints {
    min_weight: 0.02,    // Min 2% per asset
    max_weight: 0.20,    // Max 20% per asset
    long_only: true,     // No short selling
    target_vol: Some(0.10), // Target 10% volatility
};

let optimizer = MeanVarianceOptimizer::new(0.02)
    .with_constraints(constraints);

let portfolio = optimizer.max_sharpe(&assets, &returns, &covariance)?;
```

### Risk Parity

Equal risk contribution from each asset:

```rust
use sig_runtime::RiskParityOptimizer;

let optimizer = RiskParityOptimizer::new()
    .with_target_vol(0.10); // Target 10% volatility

let portfolio = optimizer.optimize(&assets, &covariance)?;

// All assets contribute equal risk
```

**Why risk parity?**: Diversifies across risk, not capital. Avoids concentration in high-volatility assets.

### Black-Litterman Model

Combine market equilibrium with your views:

```rust
use sig_runtime::{BlackLitterman, View};

let bl = BlackLitterman::new(2.5, 0.05); // Risk aversion, tau

// Market equilibrium returns
let equilibrium = bl.equilibrium_returns(&market_weights, &covariance);

// Add views
let views = vec![
    View {
        assets: vec!["AAPL".to_string()],
        weights: vec![1.0],
        expected_return: 0.15, // Expect 15% return
        confidence: 0.1,       // High confidence
    },
    View {
        assets: vec!["MSFT".to_string(), "GOOGL".to_string()],
        weights: vec![1.0, -1.0], // Relative view
        expected_return: 0.05,    // MSFT outperforms by 5%
        confidence: 0.2,
    },
];

let portfolio = bl.optimize(&assets, &market_weights, &covariance, &views)?;
```

**Why Black-Litterman?**: Avoids extreme weights from mean-variance. Blends prior (equilibrium) with your views.

### Hierarchical Risk Parity

Cluster-based diversification:

```rust
use sig_runtime::HierarchicalRiskParity;

let hrp = HierarchicalRiskParity::new();
let portfolio = hrp.optimize(&assets, &covariance)?;
```

**Why HRP?**: Robust to estimation error. Doesn't require expected returns.

## Practical Integration

### Regime-Adaptive Strategy

Adjust strategy based on regime:

```sig
data prices = load("prices.csv")
signal returns = prices / lag(prices, 1) - 1
signal vol = std(returns, 20)

// Detect regime
signal long_vol = std(returns, 60)
signal vol_ratio = vol / long_vol

// Momentum signal
signal momentum = prices / lag(prices, 60) - 1

// Regime-adaptive sizing
// High vol regime: reduce exposure
// Low vol regime: increase exposure
signal regime_scale = 1.0 / vol_ratio

// Final signal
signal adjusted = momentum * regime_scale

output adjusted
```

### Factor-Neutral Strategy

Remove factor exposures:

```rust
// Calculate factor betas
let exposures = ff.calculate_exposures(&returns, 60)?;

// Neutralize signal
let neutral_signal = raw_signal
    - exposures.beta_market * market_factor
    - exposures.beta_smb * smb_factor
    - exposures.beta_hml * hml_factor;
```

### Risk-Budgeted Portfolio

Allocate by risk contribution:

```rust
// Define risk budget (equal risk contribution)
let n = assets.len();
let risk_budget: Vec<f64> = vec![1.0 / n as f64; n];

// Optimize for risk parity
let portfolio = RiskParityOptimizer::new().optimize(&assets, &covariance)?;

// Verify risk contributions
let total_vol = portfolio.expected_volatility;
for (i, &w) in portfolio.weights.iter().enumerate() {
    let contrib = w * marginal_var[i] / total_vol;
    println!("{}: {:.1}% risk contribution", assets[i], contrib * 100.0);
}
```

## Key Takeaways

1. **Factor models** decompose risk and help identify exposures
2. **VaR/CVaR** quantify downside risk but have limitations
3. **Stress testing** reveals vulnerabilities to extreme events
4. **Regime detection** allows adaptive strategy behavior
5. **Portfolio optimization** balances return and risk
6. **Constraints matter** - unconstrained optimization produces extreme weights
7. **No free lunch** - more sophisticated doesn't always mean better

## Exercises

1. **Factor analysis**: Calculate your portfolio's Fama-French exposures. Are you taking unintended factor bets?

2. **VaR backtest**: Compare historical and parametric VaR. How often does actual loss exceed VaR?

3. **Stress test**: Run your portfolio through 2008, 2020, and 2022 scenarios. Would you survive?

4. **Regime strategy**: Build a regime detector and compare strategy performance in each regime.

5. **Optimization comparison**: Compare equal-weight, minimum variance, max Sharpe, and risk parity portfolios.

## Further Reading

- Fama & French, "Common risk factors in the returns on stocks and bonds"
- Meucci, "Risk and Asset Allocation"
- López de Prado, "Advances in Financial Machine Learning"
- Roncalli, "Introduction to Risk Parity and Budgeting"
