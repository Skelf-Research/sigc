//! Universe management for stock filtering and membership
//!
//! Handles universe definitions, index constituents, and sector mappings.

use std::collections::{HashMap, HashSet};

/// A universe of tradeable assets
#[derive(Debug, Clone)]
pub struct Universe {
    /// Universe name
    pub name: String,
    /// Member assets
    pub members: HashSet<String>,
    /// Sector mappings (asset -> sector)
    pub sectors: HashMap<String, String>,
    /// Industry mappings (asset -> industry)
    pub industries: HashMap<String, String>,
    /// Market cap category (asset -> large/mid/small)
    pub market_cap: HashMap<String, MarketCapCategory>,
}

/// Market capitalization categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketCapCategory {
    Large,
    Mid,
    Small,
    Micro,
}

impl Universe {
    /// Create a new empty universe
    pub fn new(name: &str) -> Self {
        Universe {
            name: name.to_string(),
            members: HashSet::new(),
            sectors: HashMap::new(),
            industries: HashMap::new(),
            market_cap: HashMap::new(),
        }
    }

    /// Add a member to the universe
    pub fn add_member(&mut self, asset: &str) -> &mut Self {
        self.members.insert(asset.to_string());
        self
    }

    /// Add multiple members
    pub fn add_members(&mut self, assets: &[&str]) -> &mut Self {
        for asset in assets {
            self.members.insert(asset.to_string());
        }
        self
    }

    /// Set sector for an asset
    pub fn set_sector(&mut self, asset: &str, sector: &str) -> &mut Self {
        self.sectors.insert(asset.to_string(), sector.to_string());
        self
    }

    /// Set industry for an asset
    pub fn set_industry(&mut self, asset: &str, industry: &str) -> &mut Self {
        self.industries.insert(asset.to_string(), industry.to_string());
        self
    }

    /// Set market cap category for an asset
    pub fn set_market_cap(&mut self, asset: &str, category: MarketCapCategory) -> &mut Self {
        self.market_cap.insert(asset.to_string(), category);
        self
    }

    /// Check if asset is in universe
    pub fn contains(&self, asset: &str) -> bool {
        self.members.contains(asset)
    }

    /// Get number of members
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Get assets by sector
    pub fn by_sector(&self, sector: &str) -> Vec<String> {
        self.members
            .iter()
            .filter(|asset| self.sectors.get(*asset).map(|s| s == sector).unwrap_or(false))
            .cloned()
            .collect()
    }

    /// Get assets by industry
    pub fn by_industry(&self, industry: &str) -> Vec<String> {
        self.members
            .iter()
            .filter(|asset| self.industries.get(*asset).map(|i| i == industry).unwrap_or(false))
            .cloned()
            .collect()
    }

    /// Get assets by market cap
    pub fn by_market_cap(&self, category: MarketCapCategory) -> Vec<String> {
        self.members
            .iter()
            .filter(|asset| self.market_cap.get(*asset).map(|c| *c == category).unwrap_or(false))
            .cloned()
            .collect()
    }

    /// Get all unique sectors
    pub fn sectors_list(&self) -> Vec<String> {
        let mut sectors: Vec<String> = self.sectors.values().cloned().collect();
        sectors.sort();
        sectors.dedup();
        sectors
    }

    /// Get all unique industries
    pub fn industries_list(&self) -> Vec<String> {
        let mut industries: Vec<String> = self.industries.values().cloned().collect();
        industries.sort();
        industries.dedup();
        industries
    }

    /// Filter universe by predicate
    pub fn filter<F>(&self, predicate: F) -> Universe
    where
        F: Fn(&str) -> bool,
    {
        let members: HashSet<String> = self.members
            .iter()
            .filter(|a| predicate(a))
            .cloned()
            .collect();

        let sectors: HashMap<String, String> = self.sectors
            .iter()
            .filter(|(k, _)| members.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let industries: HashMap<String, String> = self.industries
            .iter()
            .filter(|(k, _)| members.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let market_cap: HashMap<String, MarketCapCategory> = self.market_cap
            .iter()
            .filter(|(k, _)| members.contains(*k))
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        Universe {
            name: format!("{}_filtered", self.name),
            members,
            sectors,
            industries,
            market_cap,
        }
    }

    /// Intersect with another universe
    pub fn intersect(&self, other: &Universe) -> Universe {
        let members: HashSet<String> = self.members
            .intersection(&other.members)
            .cloned()
            .collect();

        Universe {
            name: format!("{}_{}_intersect", self.name, other.name),
            members: members.clone(),
            sectors: self.sectors
                .iter()
                .filter(|(k, _)| members.contains(*k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            industries: self.industries
                .iter()
                .filter(|(k, _)| members.contains(*k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            market_cap: self.market_cap
                .iter()
                .filter(|(k, _)| members.contains(*k))
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
        }
    }

    /// Union with another universe
    pub fn union(&self, other: &Universe) -> Universe {
        let mut members = self.members.clone();
        members.extend(other.members.clone());

        let mut sectors = self.sectors.clone();
        sectors.extend(other.sectors.clone());

        let mut industries = self.industries.clone();
        industries.extend(other.industries.clone());

        let mut market_cap = self.market_cap.clone();
        market_cap.extend(other.market_cap.clone());

        Universe {
            name: format!("{}_{}_union", self.name, other.name),
            members,
            sectors,
            industries,
            market_cap,
        }
    }
}

/// Universe manager for handling multiple universes
pub struct UniverseManager {
    universes: HashMap<String, Universe>,
}

impl UniverseManager {
    /// Create a new universe manager
    pub fn new() -> Self {
        UniverseManager {
            universes: HashMap::new(),
        }
    }

    /// Register a universe
    pub fn register(&mut self, universe: Universe) {
        self.universes.insert(universe.name.clone(), universe);
    }

    /// Get a universe by name
    pub fn get(&self, name: &str) -> Option<&Universe> {
        self.universes.get(name)
    }

    /// Get a mutable universe by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Universe> {
        self.universes.get_mut(name)
    }

    /// List all universe names
    pub fn list(&self) -> Vec<String> {
        self.universes.keys().cloned().collect()
    }

    /// Create built-in universes
    pub fn with_builtins(mut self) -> Self {
        // SP500 (sample)
        let mut sp500 = Universe::new("SP500");
        sp500.add_members(&[
            "AAPL", "MSFT", "GOOGL", "AMZN", "META", "NVDA", "TSLA", "BRK.B", "JPM", "JNJ",
            "V", "UNH", "HD", "PG", "MA", "DIS", "PYPL", "ADBE", "NFLX", "CRM",
        ]);

        // Set sectors for sample assets
        for asset in ["AAPL", "MSFT", "GOOGL", "META", "NVDA", "ADBE", "CRM"] {
            sp500.set_sector(asset, "Technology");
            sp500.set_market_cap(asset, MarketCapCategory::Large);
        }
        for asset in ["AMZN", "TSLA", "HD", "NFLX"] {
            sp500.set_sector(asset, "Consumer Discretionary");
            sp500.set_market_cap(asset, MarketCapCategory::Large);
        }
        for asset in ["JPM", "BRK.B", "V", "MA", "PYPL"] {
            sp500.set_sector(asset, "Financials");
            sp500.set_market_cap(asset, MarketCapCategory::Large);
        }
        for asset in ["JNJ", "UNH"] {
            sp500.set_sector(asset, "Healthcare");
            sp500.set_market_cap(asset, MarketCapCategory::Large);
        }
        sp500.set_sector("PG", "Consumer Staples");
        sp500.set_sector("DIS", "Communication Services");

        self.register(sp500);

        // Russell 2000 (sample small caps)
        let mut r2000 = Universe::new("R2000");
        r2000.add_members(&["SMCI", "MARA", "RIOT", "PLUG", "SPWR"]);
        for asset in r2000.members.clone() {
            r2000.set_market_cap(&asset, MarketCapCategory::Small);
        }
        self.register(r2000);

        self
    }
}

impl Default for UniverseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Time-varying universe membership
#[derive(Debug, Clone)]
pub struct DynamicUniverse {
    /// Universe name
    pub name: String,
    /// Membership changes over time (date -> members)
    pub membership: HashMap<String, HashSet<String>>,
}

impl DynamicUniverse {
    /// Create a new dynamic universe
    pub fn new(name: &str) -> Self {
        DynamicUniverse {
            name: name.to_string(),
            membership: HashMap::new(),
        }
    }

    /// Set membership for a date
    pub fn set_members(&mut self, date: &str, members: Vec<String>) {
        self.membership.insert(date.to_string(), members.into_iter().collect());
    }

    /// Get members for a date (uses most recent if exact not found)
    pub fn get_members(&self, date: &str) -> Option<&HashSet<String>> {
        // First try exact match
        if let Some(members) = self.membership.get(date) {
            return Some(members);
        }

        // Find most recent date before target
        let mut dates: Vec<&String> = self.membership.keys().collect();
        dates.sort();

        let mut result = None;
        for d in dates {
            if d.as_str() <= date {
                result = self.membership.get(d);
            } else {
                break;
            }
        }

        result
    }

    /// Check if asset was in universe on date
    pub fn contains(&self, asset: &str, date: &str) -> bool {
        self.get_members(date)
            .map(|m| m.contains(asset))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_basic() {
        let mut universe = Universe::new("test");
        universe.add_members(&["A", "B", "C"]);

        assert_eq!(universe.len(), 3);
        assert!(universe.contains("A"));
        assert!(!universe.contains("D"));
    }

    #[test]
    fn test_universe_sectors() {
        let mut universe = Universe::new("test");
        universe.add_members(&["AAPL", "MSFT", "JPM"]);
        universe.set_sector("AAPL", "Tech");
        universe.set_sector("MSFT", "Tech");
        universe.set_sector("JPM", "Finance");

        let tech = universe.by_sector("Tech");
        assert_eq!(tech.len(), 2);
        assert!(tech.contains(&"AAPL".to_string()));
    }

    #[test]
    fn test_universe_filter() {
        let mut universe = Universe::new("test");
        universe.add_members(&["A", "B", "C", "D"]);
        universe.set_sector("A", "Tech");
        universe.set_sector("B", "Tech");

        let filtered = universe.filter(|a| a == "A" || a == "B");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_universe_intersect() {
        let mut u1 = Universe::new("u1");
        u1.add_members(&["A", "B", "C"]);

        let mut u2 = Universe::new("u2");
        u2.add_members(&["B", "C", "D"]);

        let intersect = u1.intersect(&u2);
        assert_eq!(intersect.len(), 2);
        assert!(intersect.contains("B"));
        assert!(intersect.contains("C"));
    }

    #[test]
    fn test_manager_builtins() {
        let manager = UniverseManager::new().with_builtins();
        assert!(manager.get("SP500").is_some());
        assert!(manager.get("R2000").is_some());
    }

    #[test]
    fn test_dynamic_universe() {
        let mut universe = DynamicUniverse::new("test");
        universe.set_members("2024-01-01", vec!["A".into(), "B".into()]);
        universe.set_members("2024-06-01", vec!["A".into(), "C".into()]);

        assert!(universe.contains("B", "2024-03-01"));
        assert!(!universe.contains("C", "2024-03-01"));
        assert!(universe.contains("C", "2024-07-01"));
    }
}
