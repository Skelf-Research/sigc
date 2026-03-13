//! Transaction cost models for realistic backtesting
//!
//! Includes slippage, market impact, commissions, and borrowing costs.

use sig_types::Result;

/// Transaction cost model
#[derive(Debug, Clone)]
pub struct CostModel {
    /// Fixed commission per trade (bps)
    pub commission_bps: f64,
    /// Slippage estimate (bps)
    pub slippage_bps: f64,
    /// Market impact model
    pub impact_model: ImpactModel,
    /// Short borrowing cost (annual rate)
    pub borrow_cost_annual: f64,
    /// Minimum trade size (notional)
    pub min_trade_size: f64,
}

/// Market impact models
#[derive(Debug, Clone)]
pub enum ImpactModel {
    /// No market impact
    None,
    /// Linear impact: cost = coefficient * participation_rate
    Linear { coefficient: f64 },
    /// Square root impact: cost = coefficient * sqrt(participation_rate)
    SquareRoot { coefficient: f64 },
    /// Almgren-Chriss temporary impact
    AlmgrenChriss { eta: f64, gamma: f64 },
}

impl Default for CostModel {
    fn default() -> Self {
        CostModel {
            commission_bps: 1.0,      // 1 bp commission
            slippage_bps: 2.0,        // 2 bp slippage
            impact_model: ImpactModel::SquareRoot { coefficient: 0.1 },
            borrow_cost_annual: 0.005, // 50 bp annual borrow
            min_trade_size: 0.0,
        }
    }
}

impl CostModel {
    /// Create a new cost model with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Zero cost model for testing
    pub fn zero() -> Self {
        CostModel {
            commission_bps: 0.0,
            slippage_bps: 0.0,
            impact_model: ImpactModel::None,
            borrow_cost_annual: 0.0,
            min_trade_size: 0.0,
        }
    }

    /// Institutional cost model
    pub fn institutional() -> Self {
        CostModel {
            commission_bps: 0.5,
            slippage_bps: 1.0,
            impact_model: ImpactModel::SquareRoot { coefficient: 0.05 },
            borrow_cost_annual: 0.003,
            min_trade_size: 10000.0,
        }
    }

    /// Retail cost model
    pub fn retail() -> Self {
        CostModel {
            commission_bps: 5.0,
            slippage_bps: 5.0,
            impact_model: ImpactModel::Linear { coefficient: 0.1 },
            borrow_cost_annual: 0.02,
            min_trade_size: 0.0,
        }
    }

    /// Set commission in basis points
    pub fn with_commission(mut self, bps: f64) -> Self {
        self.commission_bps = bps;
        self
    }

    /// Set slippage in basis points
    pub fn with_slippage(mut self, bps: f64) -> Self {
        self.slippage_bps = bps;
        self
    }

    /// Set market impact model
    pub fn with_impact(mut self, model: ImpactModel) -> Self {
        self.impact_model = model;
        self
    }

    /// Set annual borrowing cost
    pub fn with_borrow_cost(mut self, rate: f64) -> Self {
        self.borrow_cost_annual = rate;
        self
    }

    /// Calculate total trading cost for a trade
    ///
    /// # Arguments
    /// * `notional` - Trade notional value
    /// * `adv` - Average daily volume (optional, for impact calculation)
    /// * `is_short` - Whether this is a short sale
    /// * `holding_days` - Expected holding period (for borrow cost)
    pub fn calculate_cost(
        &self,
        notional: f64,
        adv: Option<f64>,
        is_short: bool,
        holding_days: f64,
    ) -> TradeCost {
        let notional_abs = notional.abs();

        // Commission cost
        let commission = notional_abs * self.commission_bps / 10000.0;

        // Slippage cost
        let slippage = notional_abs * self.slippage_bps / 10000.0;

        // Market impact
        let impact = match (&self.impact_model, adv) {
            (ImpactModel::None, _) => 0.0,
            (ImpactModel::Linear { coefficient }, Some(adv)) if adv > 0.0 => {
                let participation = notional_abs / adv;
                notional_abs * coefficient * participation
            }
            (ImpactModel::SquareRoot { coefficient }, Some(adv)) if adv > 0.0 => {
                let participation = notional_abs / adv;
                notional_abs * coefficient * participation.sqrt()
            }
            (ImpactModel::AlmgrenChriss { eta, gamma }, Some(adv)) if adv > 0.0 => {
                let participation = notional_abs / adv;
                notional_abs * (eta * participation + gamma * participation.sqrt())
            }
            _ => 0.0,
        };

        // Borrowing cost for shorts
        let borrow = if is_short {
            notional_abs * self.borrow_cost_annual * (holding_days / 252.0)
        } else {
            0.0
        };

        let total = commission + slippage + impact + borrow;

        TradeCost {
            commission,
            slippage,
            impact,
            borrow,
            total,
        }
    }

    /// Calculate cost in basis points
    pub fn cost_bps(&self, notional: f64, adv: Option<f64>, is_short: bool, holding_days: f64) -> f64 {
        let cost = self.calculate_cost(notional, adv, is_short, holding_days);
        if notional.abs() > 1e-10 {
            cost.total / notional.abs() * 10000.0
        } else {
            0.0
        }
    }
}

/// Breakdown of trade costs
#[derive(Debug, Clone)]
pub struct TradeCost {
    /// Commission cost
    pub commission: f64,
    /// Slippage cost
    pub slippage: f64,
    /// Market impact cost
    pub impact: f64,
    /// Borrowing cost (shorts only)
    pub borrow: f64,
    /// Total cost
    pub total: f64,
}

impl TradeCost {
    /// Cost in basis points relative to notional
    pub fn as_bps(&self, notional: f64) -> f64 {
        if notional.abs() > 1e-10 {
            self.total / notional.abs() * 10000.0
        } else {
            0.0
        }
    }
}

/// Portfolio-level cost calculator
pub struct PortfolioCostCalculator {
    cost_model: CostModel,
}

impl PortfolioCostCalculator {
    /// Create a new calculator with the given cost model
    pub fn new(cost_model: CostModel) -> Self {
        PortfolioCostCalculator { cost_model }
    }

    /// Calculate total cost for a portfolio rebalance
    ///
    /// # Arguments
    /// * `trades` - List of (asset, notional, adv, is_short)
    /// * `holding_days` - Expected holding period
    pub fn calculate_rebalance_cost(
        &self,
        trades: &[(String, f64, Option<f64>, bool)],
        holding_days: f64,
    ) -> PortfolioCost {
        let mut total_cost = 0.0;
        let mut total_notional = 0.0;
        let mut asset_costs = Vec::new();

        for (asset, notional, adv, is_short) in trades {
            let cost = self.cost_model.calculate_cost(*notional, *adv, *is_short, holding_days);
            total_cost += cost.total;
            total_notional += notional.abs();
            asset_costs.push((asset.clone(), cost));
        }

        let avg_cost_bps = if total_notional > 1e-10 {
            total_cost / total_notional * 10000.0
        } else {
            0.0
        };

        PortfolioCost {
            total_cost,
            total_notional,
            avg_cost_bps,
            asset_costs,
        }
    }

    /// Estimate annual cost given turnover
    pub fn estimate_annual_cost(&self, portfolio_value: f64, annual_turnover: f64) -> f64 {
        let avg_trade = portfolio_value * annual_turnover / 252.0;
        let daily_cost = self.cost_model.calculate_cost(avg_trade, None, false, 1.0).total;
        daily_cost * 252.0
    }
}

/// Portfolio-level cost breakdown
#[derive(Debug, Clone)]
pub struct PortfolioCost {
    /// Total cost in dollars
    pub total_cost: f64,
    /// Total notional traded
    pub total_notional: f64,
    /// Average cost in basis points
    pub avg_cost_bps: f64,
    /// Per-asset cost breakdown
    pub asset_costs: Vec<(String, TradeCost)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cost_model() {
        let model = CostModel::new();
        let cost = model.calculate_cost(100000.0, Some(1000000.0), false, 21.0);

        assert!(cost.commission > 0.0);
        assert!(cost.slippage > 0.0);
        assert!(cost.impact > 0.0);
        assert_eq!(cost.borrow, 0.0); // Not a short
        assert!(cost.total > 0.0);
    }

    #[test]
    fn test_zero_cost_model() {
        let model = CostModel::zero();
        let cost = model.calculate_cost(100000.0, Some(1000000.0), false, 21.0);

        assert_eq!(cost.total, 0.0);
    }

    #[test]
    fn test_short_borrow_cost() {
        let model = CostModel::new();
        let cost = model.calculate_cost(100000.0, Some(1000000.0), true, 21.0);

        assert!(cost.borrow > 0.0);
    }

    #[test]
    fn test_linear_impact() {
        let model = CostModel::new().with_impact(ImpactModel::Linear { coefficient: 0.1 });
        let cost = model.calculate_cost(100000.0, Some(1000000.0), false, 21.0);

        // 10% participation * 0.1 coefficient * 100000 notional = 1000
        assert!(cost.impact > 0.0);
    }

    #[test]
    fn test_sqrt_impact() {
        let model = CostModel::new().with_impact(ImpactModel::SquareRoot { coefficient: 0.1 });

        // Larger trade should have diminishing marginal impact
        let cost_small = model.calculate_cost(10000.0, Some(1000000.0), false, 21.0);
        let cost_large = model.calculate_cost(100000.0, Some(1000000.0), false, 21.0);

        let bps_small = cost_small.impact / 10000.0 * 10000.0;
        let bps_large = cost_large.impact / 100000.0 * 10000.0;

        // Square root model should show decreasing bps for larger trades
        assert!(bps_large < bps_small * 3.2); // sqrt(10) ≈ 3.16
    }

    #[test]
    fn test_portfolio_cost() {
        let model = CostModel::new();
        let calculator = PortfolioCostCalculator::new(model);

        let trades = vec![
            ("AAPL".to_string(), 50000.0, Some(5000000.0), false),
            ("MSFT".to_string(), -30000.0, Some(4000000.0), true),
        ];

        let cost = calculator.calculate_rebalance_cost(&trades, 21.0);

        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.total_notional, 80000.0);
        assert!(cost.avg_cost_bps > 0.0);
        assert_eq!(cost.asset_costs.len(), 2);
    }

    #[test]
    fn test_cost_presets() {
        let retail = CostModel::retail();
        let institutional = CostModel::institutional();

        let notional = 100000.0;
        let adv = Some(1000000.0);

        let retail_cost = retail.cost_bps(notional, adv, false, 21.0);
        let inst_cost = institutional.cost_bps(notional, adv, false, 21.0);

        // Retail should be more expensive
        assert!(retail_cost > inst_cost);
    }
}
