//! Corporate actions handling for price adjustments
//!
//! Handles stock splits, dividends, spinoffs, and symbol changes.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Types of corporate actions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ActionType {
    /// Stock split (e.g., 2-for-1)
    Split { ratio: f64 },
    /// Reverse split (e.g., 1-for-10)
    ReverseSplit { ratio: f64 },
    /// Cash dividend
    Dividend { amount: f64 },
    /// Special dividend
    SpecialDividend { amount: f64 },
    /// Spinoff
    Spinoff {
        new_symbol: String,
        ratio: f64,  // shares of new company per share
    },
    /// Symbol change
    SymbolChange { new_symbol: String },
    /// Merger
    Merger {
        acquirer: String,
        ratio: f64,  // shares of acquirer per share
        cash: f64,   // cash per share
    },
}

/// A corporate action event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorporateAction {
    pub symbol: String,
    pub date: String,
    pub action_type: ActionType,
    pub ex_date: Option<String>,
    pub record_date: Option<String>,
    pub payment_date: Option<String>,
}

impl CorporateAction {
    /// Create a stock split
    pub fn split(symbol: &str, date: &str, ratio: f64) -> Self {
        CorporateAction {
            symbol: symbol.to_string(),
            date: date.to_string(),
            action_type: ActionType::Split { ratio },
            ex_date: Some(date.to_string()),
            record_date: None,
            payment_date: None,
        }
    }

    /// Create a dividend
    pub fn dividend(symbol: &str, date: &str, amount: f64) -> Self {
        CorporateAction {
            symbol: symbol.to_string(),
            date: date.to_string(),
            action_type: ActionType::Dividend { amount },
            ex_date: Some(date.to_string()),
            record_date: None,
            payment_date: None,
        }
    }

    /// Create a symbol change
    pub fn symbol_change(old_symbol: &str, new_symbol: &str, date: &str) -> Self {
        CorporateAction {
            symbol: old_symbol.to_string(),
            date: date.to_string(),
            action_type: ActionType::SymbolChange { new_symbol: new_symbol.to_string() },
            ex_date: None,
            record_date: None,
            payment_date: None,
        }
    }
}

/// Trait for applying corporate action adjustments
pub trait CorporateActionAdjuster: Send + Sync {
    /// Adjust prices for corporate actions
    fn adjust_prices(&self, df: &DataFrame, actions: &[CorporateAction]) -> Result<DataFrame>;

    /// Get adjuster name
    fn name(&self) -> &str;
}

/// Standard corporate action adjuster
pub struct StandardAdjuster {
    /// Whether to adjust for dividends
    pub adjust_dividends: bool,
    /// Whether to adjust for splits
    pub adjust_splits: bool,
    /// Date column name
    pub date_column: String,
    /// Price columns to adjust
    pub price_columns: Vec<String>,
    /// Volume column (adjusted inversely for splits)
    pub volume_column: Option<String>,
}

impl StandardAdjuster {
    pub fn new() -> Self {
        StandardAdjuster {
            adjust_dividends: true,
            adjust_splits: true,
            date_column: "date".to_string(),
            price_columns: vec!["open".to_string(), "high".to_string(), "low".to_string(), "close".to_string()],
            volume_column: Some("volume".to_string()),
        }
    }

    pub fn with_date_column(mut self, col: &str) -> Self {
        self.date_column = col.to_string();
        self
    }

    pub fn with_price_columns(mut self, cols: Vec<String>) -> Self {
        self.price_columns = cols;
        self
    }

    pub fn with_volume_column(mut self, col: &str) -> Self {
        self.volume_column = Some(col.to_string());
        self
    }

    pub fn without_dividends(mut self) -> Self {
        self.adjust_dividends = false;
        self
    }

    /// Calculate cumulative adjustment factor for a date
    fn calculate_adjustment_factor(
        &self,
        date: &str,
        actions: &[CorporateAction],
    ) -> (f64, f64) {
        let mut price_factor = 1.0;
        let mut volume_factor = 1.0;

        for action in actions {
            // Only apply actions that occurred after this date
            if action.date.as_str() > date {
                match &action.action_type {
                    ActionType::Split { ratio } => {
                        if self.adjust_splits {
                            price_factor /= ratio;
                            volume_factor *= ratio;
                        }
                    }
                    ActionType::ReverseSplit { ratio } => {
                        if self.adjust_splits {
                            price_factor *= ratio;
                            volume_factor /= ratio;
                        }
                    }
                    ActionType::Dividend { amount } | ActionType::SpecialDividend { amount } => {
                        if self.adjust_dividends {
                            // Dividend adjustment: price is reduced by dividend amount
                            // This is simplified; proper adjustment uses closing price
                            price_factor *= 1.0 - (amount / 100.0); // Assume percentage
                        }
                    }
                    _ => {}
                }
            }
        }

        (price_factor, volume_factor)
    }
}

impl Default for StandardAdjuster {
    fn default() -> Self {
        Self::new()
    }
}

impl CorporateActionAdjuster for StandardAdjuster {
    fn adjust_prices(&self, df: &DataFrame, actions: &[CorporateAction]) -> Result<DataFrame> {
        if actions.is_empty() {
            return Ok(df.clone());
        }

        let mut result = df.clone();
        let height = df.height();

        // Get date column
        let dates = df.column(&self.date_column)
            .map_err(|e| SigcError::Runtime(format!("Date column error: {}", e)))?;

        // Calculate adjustment factors for each row
        let mut price_factors = Vec::with_capacity(height);
        let mut volume_factors = Vec::with_capacity(height);

        for i in 0..height {
            let date = dates.get(i)
                .map(|v| format!("{:?}", v))
                .unwrap_or_default();

            let (pf, vf) = self.calculate_adjustment_factor(&date, actions);
            price_factors.push(pf);
            volume_factors.push(vf);
        }

        // Adjust price columns
        for col_name in &self.price_columns {
            if let Ok(col) = df.column(col_name) {
                let adjusted: Vec<f64> = col.f64()
                    .map_err(|e| SigcError::Runtime(format!("Column cast error: {}", e)))?
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| v.unwrap_or(f64::NAN) * price_factors[i])
                    .collect();

                let new_col = Column::new(col_name.into(), adjusted);
                result = result.with_column(new_col)
                    .map_err(|e| SigcError::Runtime(format!("Column replace error: {}", e)))?
                    .clone();
            }
        }

        // Adjust volume column
        if let Some(ref vol_col) = self.volume_column {
            if let Ok(col) = df.column(vol_col) {
                let adjusted: Vec<f64> = col.f64()
                    .map_err(|e| SigcError::Runtime(format!("Volume cast error: {}", e)))?
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| v.unwrap_or(0.0) * volume_factors[i])
                    .collect();

                let new_col = Column::new(vol_col.into(), adjusted);
                result = result.with_column(new_col)
                    .map_err(|e| SigcError::Runtime(format!("Volume replace error: {}", e)))?
                    .clone();
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "standard"
    }
}

/// Storage for corporate actions
pub struct CorporateActionStore {
    actions: HashMap<String, Vec<CorporateAction>>,
}

impl CorporateActionStore {
    pub fn new() -> Self {
        CorporateActionStore {
            actions: HashMap::new(),
        }
    }

    /// Add a corporate action
    pub fn add(&mut self, action: CorporateAction) {
        self.actions
            .entry(action.symbol.clone())
            .or_default()
            .push(action);
    }

    /// Get actions for a symbol
    pub fn get(&self, symbol: &str) -> Option<&Vec<CorporateAction>> {
        self.actions.get(symbol)
    }

    /// Get actions for a symbol within a date range
    pub fn get_in_range(&self, symbol: &str, start: &str, end: &str) -> Vec<&CorporateAction> {
        self.actions
            .get(symbol)
            .map(|actions| {
                actions
                    .iter()
                    .filter(|a| a.date.as_str() >= start && a.date.as_str() <= end)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all symbols with actions
    pub fn symbols(&self) -> Vec<&String> {
        self.actions.keys().collect()
    }

    /// Load actions from a DataFrame
    pub fn load_from_df(&mut self, df: &DataFrame) -> Result<usize> {
        let symbol_col = df.column("symbol")
            .map_err(|e| SigcError::Runtime(format!("Symbol column error: {}", e)))?;
        let date_col = df.column("date")
            .map_err(|e| SigcError::Runtime(format!("Date column error: {}", e)))?;
        let action_col = df.column("action_type")
            .map_err(|e| SigcError::Runtime(format!("Action type column error: {}", e)))?;
        let value_col = df.column("value")
            .map_err(|e| SigcError::Runtime(format!("Value column error: {}", e)))?;

        let mut count = 0;

        for i in 0..df.height() {
            let symbol = symbol_col.get(i)
                .map(|v| format!("{:?}", v).trim_matches('"').to_string())
                .unwrap_or_default();
            let date = date_col.get(i)
                .map(|v| format!("{:?}", v).trim_matches('"').to_string())
                .unwrap_or_default();
            let action_type = action_col.get(i)
                .map(|v| format!("{:?}", v).trim_matches('"').to_string())
                .unwrap_or_default();
            let value = value_col.get(i)
                .map(|v| v.extract::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);

            let action = match action_type.as_str() {
                "split" => CorporateAction::split(&symbol, &date, value),
                "dividend" => CorporateAction::dividend(&symbol, &date, value),
                _ => continue,
            };

            self.add(action);
            count += 1;
        }

        Ok(count)
    }
}

impl Default for CorporateActionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol mapping for tracking symbol changes
pub struct SymbolMapper {
    /// Old symbol -> (new symbol, change date)
    mappings: HashMap<String, (String, String)>,
}

impl SymbolMapper {
    pub fn new() -> Self {
        SymbolMapper {
            mappings: HashMap::new(),
        }
    }

    /// Add a symbol change
    pub fn add_change(&mut self, old: &str, new: &str, date: &str) {
        self.mappings.insert(
            old.to_string(),
            (new.to_string(), date.to_string()),
        );
    }

    /// Get current symbol for an old symbol
    pub fn get_current<'a>(&'a self, symbol: &'a str) -> &'a str {
        match self.mappings.get(symbol) {
            Some((new_symbol, _)) => self.get_current(new_symbol), // Recursive for chains
            None => symbol,
        }
    }

    /// Get historical symbol at a given date
    pub fn get_at_date<'a>(&'a self, current_symbol: &'a str, date: &str) -> &'a str {
        // Find if current symbol was renamed from something else after the date
        for (old, (new, change_date)) in &self.mappings {
            if new == current_symbol && change_date.as_str() > date {
                return old;
            }
        }
        current_symbol
    }
}

impl Default for SymbolMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_action() {
        let action = CorporateAction::split("AAPL", "2020-08-31", 4.0);
        match action.action_type {
            ActionType::Split { ratio } => assert_eq!(ratio, 4.0),
            _ => panic!("Expected Split"),
        }
    }

    #[test]
    fn test_dividend_action() {
        let action = CorporateAction::dividend("MSFT", "2024-01-15", 0.75);
        match action.action_type {
            ActionType::Dividend { amount } => assert_eq!(amount, 0.75),
            _ => panic!("Expected Dividend"),
        }
    }

    #[test]
    fn test_symbol_change() {
        let action = CorporateAction::symbol_change("FB", "META", "2022-10-28");
        match action.action_type {
            ActionType::SymbolChange { new_symbol } => assert_eq!(new_symbol, "META"),
            _ => panic!("Expected SymbolChange"),
        }
    }

    #[test]
    fn test_action_store() {
        let mut store = CorporateActionStore::new();

        store.add(CorporateAction::split("AAPL", "2020-08-31", 4.0));
        store.add(CorporateAction::split("AAPL", "2014-06-09", 7.0));
        store.add(CorporateAction::dividend("AAPL", "2024-01-15", 0.24));

        let actions = store.get("AAPL").unwrap();
        assert_eq!(actions.len(), 3);

        let range_actions = store.get_in_range("AAPL", "2020-01-01", "2024-12-31");
        assert_eq!(range_actions.len(), 2);
    }

    #[test]
    fn test_symbol_mapper() {
        let mut mapper = SymbolMapper::new();
        mapper.add_change("FB", "META", "2022-10-28");

        assert_eq!(mapper.get_current("FB"), "META");
        assert_eq!(mapper.get_current("META"), "META");
        assert_eq!(mapper.get_current("AAPL"), "AAPL");

        // Historical lookup
        assert_eq!(mapper.get_at_date("META", "2022-01-01"), "FB");
        assert_eq!(mapper.get_at_date("META", "2023-01-01"), "META");
    }

    #[test]
    fn test_adjuster_config() {
        let adjuster = StandardAdjuster::new()
            .with_date_column("trade_date")
            .with_price_columns(vec!["price".to_string()])
            .without_dividends();

        assert_eq!(adjuster.date_column, "trade_date");
        assert_eq!(adjuster.price_columns.len(), 1);
        assert!(!adjuster.adjust_dividends);
    }

    #[test]
    fn test_adjustment_factor() {
        let adjuster = StandardAdjuster::new();

        let actions = vec![
            CorporateAction::split("AAPL", "2020-08-31", 4.0),
        ];

        // Date before split should be adjusted
        let (price_factor, volume_factor) = adjuster.calculate_adjustment_factor("2020-01-01", &actions);
        assert!((price_factor - 0.25).abs() < 0.001);
        assert!((volume_factor - 4.0).abs() < 0.001);

        // Date after split should not be adjusted
        let (price_factor, volume_factor) = adjuster.calculate_adjustment_factor("2021-01-01", &actions);
        assert!((price_factor - 1.0).abs() < 0.001);
        assert!((volume_factor - 1.0).abs() < 0.001);
    }
}
