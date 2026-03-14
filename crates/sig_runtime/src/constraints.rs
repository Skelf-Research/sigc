//! Position constraints and risk limits
//!
//! Enforces position-level and portfolio-level constraints on weights.

use sig_types::Result;
use std::collections::HashMap;

/// Position-level constraint
#[derive(Debug, Clone)]
pub enum PositionConstraint {
    /// Maximum absolute weight per position
    MaxWeight(f64),
    /// Minimum absolute weight per position (filter small positions)
    MinWeight(f64),
    /// Maximum long weight per position
    MaxLongWeight(f64),
    /// Maximum short weight per position
    MaxShortWeight(f64),
}

/// Portfolio-level constraint
#[derive(Debug, Clone)]
pub enum PortfolioConstraint {
    /// Maximum gross exposure (sum of absolute weights)
    MaxGrossExposure(f64),
    /// Maximum net exposure (sum of weights)
    MaxNetExposure(f64),
    /// Minimum net exposure
    MinNetExposure(f64),
    /// Target net exposure (will scale to achieve)
    TargetNetExposure(f64),
    /// Maximum number of long positions
    MaxLongs(usize),
    /// Maximum number of short positions
    MaxShorts(usize),
    /// Maximum total positions
    MaxPositions(usize),
    /// Long/short ratio constraint (long_weight / short_weight)
    LongShortRatio { min: f64, max: f64 },
}

/// Sector-level constraint
#[derive(Debug, Clone)]
pub struct SectorConstraint {
    /// Sector name
    pub sector: String,
    /// Maximum absolute weight in sector
    pub max_weight: Option<f64>,
    /// Minimum absolute weight in sector
    pub min_weight: Option<f64>,
    /// Maximum deviation from benchmark weight
    pub max_active_weight: Option<f64>,
}

/// Turnover constraint
#[derive(Debug, Clone)]
pub struct TurnoverConstraint {
    /// Maximum one-way turnover per rebalance
    pub max_turnover: f64,
    /// Maximum annualized turnover
    pub max_annual_turnover: Option<f64>,
}

/// Complete constraint set for portfolio optimization
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    /// Position-level constraints
    pub position: Vec<PositionConstraint>,
    /// Portfolio-level constraints
    pub portfolio: Vec<PortfolioConstraint>,
    /// Sector constraints
    pub sector: Vec<SectorConstraint>,
    /// Turnover constraint
    pub turnover: Option<TurnoverConstraint>,
}

impl ConstraintSet {
    /// Create an empty constraint set
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a position constraint
    pub fn add_position(mut self, constraint: PositionConstraint) -> Self {
        self.position.push(constraint);
        self
    }

    /// Add a portfolio constraint
    pub fn add_portfolio(mut self, constraint: PortfolioConstraint) -> Self {
        self.portfolio.push(constraint);
        self
    }

    /// Add a sector constraint
    pub fn add_sector(mut self, constraint: SectorConstraint) -> Self {
        self.sector.push(constraint);
        self
    }

    /// Set turnover constraint
    pub fn with_turnover(mut self, max_turnover: f64) -> Self {
        self.turnover = Some(TurnoverConstraint {
            max_turnover,
            max_annual_turnover: None,
        });
        self
    }

    /// Common constraint set for long-short equity
    pub fn long_short_equity() -> Self {
        Self::new()
            .add_position(PositionConstraint::MaxWeight(0.05))
            .add_position(PositionConstraint::MinWeight(0.001))
            .add_portfolio(PortfolioConstraint::MaxGrossExposure(2.0))
            .add_portfolio(PortfolioConstraint::TargetNetExposure(0.0))
            .with_turnover(0.3)
    }

    /// Common constraint set for long-only
    pub fn long_only() -> Self {
        Self::new()
            .add_position(PositionConstraint::MaxWeight(0.10))
            .add_position(PositionConstraint::MinWeight(0.005))
            .add_position(PositionConstraint::MaxShortWeight(0.0))
            .add_portfolio(PortfolioConstraint::MaxGrossExposure(1.0))
            .add_portfolio(PortfolioConstraint::TargetNetExposure(1.0))
            .with_turnover(0.5)
    }
}

/// Constraint enforcer that adjusts weights to satisfy constraints
pub struct ConstraintEnforcer {
    constraints: ConstraintSet,
    sector_mapping: HashMap<String, String>,
}

impl ConstraintEnforcer {
    /// Create a new constraint enforcer
    pub fn new(constraints: ConstraintSet) -> Self {
        ConstraintEnforcer {
            constraints,
            sector_mapping: HashMap::new(),
        }
    }

    /// Set sector mapping (asset -> sector)
    pub fn with_sectors(mut self, mapping: HashMap<String, String>) -> Self {
        self.sector_mapping = mapping;
        self
    }

    /// Apply constraints to weights, returning adjusted weights
    pub fn apply(
        &self,
        weights: &HashMap<String, f64>,
        prev_weights: Option<&HashMap<String, f64>>,
    ) -> Result<HashMap<String, f64>> {
        let mut adjusted = weights.clone();

        // Apply position-level constraints first
        self.apply_position_constraints(&mut adjusted)?;

        // Apply sector constraints
        self.apply_sector_constraints(&mut adjusted)?;

        // Apply portfolio-level constraints
        self.apply_portfolio_constraints(&mut adjusted)?;

        // Apply turnover constraint
        if let Some(ref turnover) = self.constraints.turnover {
            if let Some(prev) = prev_weights {
                self.apply_turnover_constraint(&mut adjusted, prev, turnover)?;
            }
        }

        Ok(adjusted)
    }

    /// Apply position-level constraints
    fn apply_position_constraints(&self, weights: &mut HashMap<String, f64>) -> Result<()> {
        for constraint in &self.constraints.position {
            match constraint {
                PositionConstraint::MaxWeight(max) => {
                    for weight in weights.values_mut() {
                        if weight.abs() > *max {
                            *weight = max * weight.signum();
                        }
                    }
                }
                PositionConstraint::MinWeight(min) => {
                    weights.retain(|_, w| w.abs() >= *min);
                }
                PositionConstraint::MaxLongWeight(max) => {
                    for weight in weights.values_mut() {
                        if *weight > *max {
                            *weight = *max;
                        }
                    }
                }
                PositionConstraint::MaxShortWeight(max) => {
                    for weight in weights.values_mut() {
                        if *weight < -(*max) {
                            *weight = -(*max);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Apply sector constraints
    fn apply_sector_constraints(&self, weights: &mut HashMap<String, f64>) -> Result<()> {
        if self.constraints.sector.is_empty() {
            return Ok(());
        }

        // Calculate current sector weights
        let mut sector_weights: HashMap<String, f64> = HashMap::new();
        for (asset, weight) in weights.iter() {
            if let Some(sector) = self.sector_mapping.get(asset) {
                *sector_weights.entry(sector.clone()).or_insert(0.0) += weight.abs();
            }
        }

        // Apply sector constraints
        for constraint in &self.constraints.sector {
            let current = sector_weights.get(&constraint.sector).copied().unwrap_or(0.0);

            if let Some(max) = constraint.max_weight {
                if current > max && current > 0.0 {
                    let scale = max / current;
                    for (asset, weight) in weights.iter_mut() {
                        if self.sector_mapping.get(asset) == Some(&constraint.sector) {
                            *weight *= scale;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply portfolio-level constraints
    fn apply_portfolio_constraints(&self, weights: &mut HashMap<String, f64>) -> Result<()> {
        for constraint in &self.constraints.portfolio {
            match constraint {
                PortfolioConstraint::MaxGrossExposure(max) => {
                    let gross: f64 = weights.values().map(|w| w.abs()).sum();
                    if gross > *max {
                        let scale = max / gross;
                        for weight in weights.values_mut() {
                            *weight *= scale;
                        }
                    }
                }
                PortfolioConstraint::MaxNetExposure(max) => {
                    let net: f64 = weights.values().sum();
                    if net > *max {
                        // Reduce long positions proportionally
                        let long_sum: f64 = weights.values().filter(|w| **w > 0.0).sum();
                        if long_sum > 0.0 {
                            let reduction = (net - max) / long_sum;
                            for weight in weights.values_mut() {
                                if *weight > 0.0 {
                                    *weight *= 1.0 - reduction;
                                }
                            }
                        }
                    }
                }
                PortfolioConstraint::MinNetExposure(min) => {
                    let net: f64 = weights.values().sum();
                    if net < *min {
                        // Reduce short positions proportionally
                        let short_sum: f64 = weights.values().filter(|w| **w < 0.0).map(|w| w.abs()).sum();
                        if short_sum > 0.0 {
                            let reduction = (min - net) / short_sum;
                            for weight in weights.values_mut() {
                                if *weight < 0.0 {
                                    *weight *= 1.0 - reduction;
                                }
                            }
                        }
                    }
                }
                PortfolioConstraint::TargetNetExposure(target) => {
                    let net: f64 = weights.values().sum();
                    let diff = target - net;
                    if diff.abs() > 1e-10 {
                        // Adjust proportionally
                        let long_sum: f64 = weights.values().filter(|w| **w > 0.0).sum();
                        let short_sum: f64 = weights.values().filter(|w| **w < 0.0).map(|w| w.abs()).sum();

                        if diff > 0.0 && long_sum > 0.0 {
                            // Need more long exposure
                            let scale = 1.0 + diff / long_sum;
                            for weight in weights.values_mut() {
                                if *weight > 0.0 {
                                    *weight *= scale;
                                }
                            }
                        } else if diff < 0.0 && short_sum > 0.0 {
                            // Need more short exposure
                            let scale = 1.0 + diff.abs() / short_sum;
                            for weight in weights.values_mut() {
                                if *weight < 0.0 {
                                    *weight *= scale;
                                }
                            }
                        }
                    }
                }
                PortfolioConstraint::MaxLongs(max) => {
                    let mut longs: Vec<_> = weights.iter()
                        .filter(|(_, w)| **w > 0.0)
                        .map(|(k, v)| (k.clone(), *v))
                        .collect();
                    if longs.len() > *max {
                        longs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                        for (asset, _) in longs.iter().skip(*max) {
                            weights.remove(asset);
                        }
                    }
                }
                PortfolioConstraint::MaxShorts(max) => {
                    let mut shorts: Vec<_> = weights.iter()
                        .filter(|(_, w)| **w < 0.0)
                        .map(|(k, v)| (k.clone(), *v))
                        .collect();
                    if shorts.len() > *max {
                        shorts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                        for (asset, _) in shorts.iter().skip(*max) {
                            weights.remove(asset);
                        }
                    }
                }
                PortfolioConstraint::MaxPositions(max) => {
                    if weights.len() > *max {
                        let mut positions: Vec<_> = weights.iter()
                            .map(|(k, v)| (k.clone(), v.abs()))
                            .collect();
                        positions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                        let to_keep: std::collections::HashSet<_> = positions.iter()
                            .take(*max)
                            .map(|(k, _)| k.clone())
                            .collect();
                        weights.retain(|k, _| to_keep.contains(k));
                    }
                }
                PortfolioConstraint::LongShortRatio { min, max } => {
                    let long_sum: f64 = weights.values().filter(|w| **w > 0.0).sum();
                    let short_sum: f64 = weights.values().filter(|w| **w < 0.0).map(|w| w.abs()).sum();

                    if short_sum > 0.0 {
                        let ratio = long_sum / short_sum;
                        if ratio < *min {
                            // Scale down shorts
                            let scale = long_sum / (min * short_sum);
                            for weight in weights.values_mut() {
                                if *weight < 0.0 {
                                    *weight *= scale;
                                }
                            }
                        } else if ratio > *max {
                            // Scale down longs
                            let scale = max * short_sum / long_sum;
                            for weight in weights.values_mut() {
                                if *weight > 0.0 {
                                    *weight *= scale;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Apply turnover constraint
    fn apply_turnover_constraint(
        &self,
        weights: &mut HashMap<String, f64>,
        prev_weights: &HashMap<String, f64>,
        constraint: &TurnoverConstraint,
    ) -> Result<()> {
        // Calculate one-way turnover
        let turnover: f64 = weights.iter()
            .map(|(asset, &new_weight)| {
                let old_weight = prev_weights.get(asset).copied().unwrap_or(0.0);
                (new_weight - old_weight).abs()
            })
            .sum::<f64>() / 2.0; // One-way turnover

        if turnover > constraint.max_turnover {
            // Blend old and new weights to reduce turnover
            let blend = constraint.max_turnover / turnover;
            for (asset, weight) in weights.iter_mut() {
                let old = prev_weights.get(asset).copied().unwrap_or(0.0);
                *weight = old + blend * (*weight - old);
            }
        }

        Ok(())
    }

    /// Validate weights against constraints (returns violations)
    pub fn validate(&self, weights: &HashMap<String, f64>) -> Vec<String> {
        let mut violations = Vec::new();

        // Check position constraints
        for constraint in &self.constraints.position {
            match constraint {
                PositionConstraint::MaxWeight(max) => {
                    for (asset, weight) in weights {
                        if weight.abs() > *max {
                            violations.push(format!(
                                "{} weight {} exceeds max {}",
                                asset, weight, max
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        // Check portfolio constraints
        let gross: f64 = weights.values().map(|w| w.abs()).sum();
        let net: f64 = weights.values().sum();

        for constraint in &self.constraints.portfolio {
            match constraint {
                PortfolioConstraint::MaxGrossExposure(max) => {
                    if gross > *max {
                        violations.push(format!(
                            "Gross exposure {} exceeds max {}",
                            gross, max
                        ));
                    }
                }
                PortfolioConstraint::MaxNetExposure(max) => {
                    if net > *max {
                        violations.push(format!(
                            "Net exposure {} exceeds max {}",
                            net, max
                        ));
                    }
                }
                _ => {}
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_weight_constraint() {
        let constraints = ConstraintSet::new()
            .add_position(PositionConstraint::MaxWeight(0.05));

        let enforcer = ConstraintEnforcer::new(constraints);

        let mut weights = HashMap::new();
        weights.insert("A".to_string(), 0.10);
        weights.insert("B".to_string(), -0.08);
        weights.insert("C".to_string(), 0.03);

        let adjusted = enforcer.apply(&weights, None).unwrap();

        assert!((adjusted["A"] - 0.05).abs() < 1e-10);
        assert!((adjusted["B"] - (-0.05)).abs() < 1e-10);
        assert!((adjusted["C"] - 0.03).abs() < 1e-10);
    }

    #[test]
    fn test_max_gross_exposure() {
        let constraints = ConstraintSet::new()
            .add_portfolio(PortfolioConstraint::MaxGrossExposure(1.0));

        let enforcer = ConstraintEnforcer::new(constraints);

        let mut weights = HashMap::new();
        weights.insert("A".to_string(), 0.8);
        weights.insert("B".to_string(), -0.6);
        weights.insert("C".to_string(), 0.6);

        let adjusted = enforcer.apply(&weights, None).unwrap();
        let gross: f64 = adjusted.values().map(|w| w.abs()).sum();

        assert!((gross - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_turnover_constraint() {
        let constraints = ConstraintSet::new()
            .with_turnover(0.1);

        let enforcer = ConstraintEnforcer::new(constraints);

        let mut prev = HashMap::new();
        prev.insert("A".to_string(), 0.5);
        prev.insert("B".to_string(), 0.5);

        let mut new_weights = HashMap::new();
        new_weights.insert("A".to_string(), 0.0);
        new_weights.insert("B".to_string(), 1.0);

        let adjusted = enforcer.apply(&new_weights, Some(&prev)).unwrap();

        // Turnover should be limited
        let turnover: f64 = adjusted.iter()
            .map(|(k, &v)| (v - prev.get(k).unwrap_or(&0.0)).abs())
            .sum::<f64>() / 2.0;

        assert!(turnover <= 0.1 + 1e-10);
    }

    #[test]
    fn test_long_short_preset() {
        let constraints = ConstraintSet::long_short_equity();
        assert!(!constraints.position.is_empty());
        assert!(!constraints.portfolio.is_empty());
        assert!(constraints.turnover.is_some());
    }

    #[test]
    fn test_long_only_preset() {
        let constraints = ConstraintSet::long_only();

        let enforcer = ConstraintEnforcer::new(constraints);

        let mut weights = HashMap::new();
        weights.insert("A".to_string(), -0.5);
        weights.insert("B".to_string(), 0.5);

        let adjusted = enforcer.apply(&weights, None).unwrap();

        // Short position should be zero
        assert!((adjusted.get("A").copied().unwrap_or(0.0)).abs() < 1e-10);
    }

    #[test]
    fn test_validation() {
        let constraints = ConstraintSet::new()
            .add_position(PositionConstraint::MaxWeight(0.05))
            .add_portfolio(PortfolioConstraint::MaxGrossExposure(1.0));

        let enforcer = ConstraintEnforcer::new(constraints);

        let mut weights = HashMap::new();
        weights.insert("A".to_string(), 0.10);
        weights.insert("B".to_string(), 1.0);

        let violations = enforcer.validate(&weights);
        assert!(!violations.is_empty());
    }
}
