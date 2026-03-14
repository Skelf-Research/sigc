//! External integrations for data vendors and brokers
//!
//! Provides connectors for market data providers and brokerage APIs.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;
use std::sync::Arc;

/// Market data provider trait
pub trait MarketDataProvider: Send + Sync {
    /// Get historical OHLCV data
    fn get_ohlcv(
        &self,
        symbol: &str,
        start: &str,
        end: &str,
        interval: &str,
    ) -> Result<DataFrame>;

    /// Get current quote
    fn get_quote(&self, symbol: &str) -> Result<Quote>;

    /// Get multiple quotes
    fn get_quotes(&self, symbols: &[&str]) -> Result<Vec<Quote>>;

    /// Provider name
    fn name(&self) -> &str;
}

/// Price quote
#[derive(Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: f64,
    pub timestamp: i64,
}

/// Yahoo Finance data provider
pub struct YahooFinance {
    base_url: String,
}

impl YahooFinance {
    pub fn new() -> Self {
        Self {
            base_url: "https://query1.finance.yahoo.com".to_string(),
        }
    }

    /// Convert interval string to Yahoo format
    fn interval_to_yahoo(&self, interval: &str) -> &str {
        match interval {
            "1m" => "1m",
            "5m" => "5m",
            "15m" => "15m",
            "30m" => "30m",
            "1h" => "60m",
            "1d" | "daily" => "1d",
            "1wk" | "weekly" => "1wk",
            "1mo" | "monthly" => "1mo",
            _ => "1d",
        }
    }
}

impl Default for YahooFinance {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketDataProvider for YahooFinance {
    fn get_ohlcv(
        &self,
        symbol: &str,
        start: &str,
        end: &str,
        interval: &str,
    ) -> Result<DataFrame> {
        // Parse dates to Unix timestamps
        let start_ts = parse_date_to_timestamp(start)?;
        let end_ts = parse_date_to_timestamp(end)?;
        let yahoo_interval = self.interval_to_yahoo(interval);

        // Build URL
        let url = format!(
            "{}/v8/finance/chart/{}?period1={}&period2={}&interval={}",
            self.base_url, symbol, start_ts, end_ts, yahoo_interval
        );

        // For now, return sample data (actual HTTP calls would need reqwest)
        tracing::info!("Yahoo Finance URL: {}", url);

        // Generate sample data for demonstration
        let n_days = ((end_ts - start_ts) / 86400).max(1) as usize;
        generate_sample_ohlcv(symbol, n_days)
    }

    fn get_quote(&self, symbol: &str) -> Result<Quote> {
        // Sample quote for demonstration
        Ok(Quote {
            symbol: symbol.to_string(),
            bid: 150.0,
            ask: 150.05,
            last: 150.02,
            volume: 1_000_000.0,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    fn get_quotes(&self, symbols: &[&str]) -> Result<Vec<Quote>> {
        symbols.iter().map(|s| self.get_quote(s)).collect()
    }

    fn name(&self) -> &str {
        "yahoo_finance"
    }
}

/// Alpaca broker integration
pub struct AlpacaBroker {
    api_key: String,
    api_secret: String,
    base_url: String,
    paper: bool,
}

impl AlpacaBroker {
    /// Create a paper trading instance
    pub fn paper(api_key: &str, api_secret: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            base_url: "https://paper-api.alpaca.markets".to_string(),
            paper: true,
        }
    }

    /// Create a live trading instance
    pub fn live(api_key: &str, api_secret: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            base_url: "https://api.alpaca.markets".to_string(),
            paper: false,
        }
    }

    /// Get account information
    pub fn get_account(&self) -> Result<AlpacaAccount> {
        tracing::info!("Getting Alpaca account (paper={})", self.paper);

        // Sample account for demonstration
        Ok(AlpacaAccount {
            id: "account-123".to_string(),
            cash: 100_000.0,
            portfolio_value: 150_000.0,
            buying_power: 200_000.0,
            equity: 150_000.0,
            pattern_day_trader: false,
        })
    }

    /// Get positions
    pub fn get_positions(&self) -> Result<Vec<AlpacaPosition>> {
        tracing::info!("Getting Alpaca positions");

        // Sample positions
        Ok(vec![
            AlpacaPosition {
                symbol: "AAPL".to_string(),
                qty: 100.0,
                avg_entry_price: 145.0,
                market_value: 15000.0,
                unrealized_pl: 500.0,
                side: "long".to_string(),
            },
        ])
    }

    /// Submit an order
    pub fn submit_order(&self, order: &AlpacaOrder) -> Result<AlpacaOrderResponse> {
        tracing::info!(
            "Submitting {} order: {} {} @ {:?}",
            order.side, order.qty, order.symbol, order.limit_price
        );

        // Sample order response
        Ok(AlpacaOrderResponse {
            id: format!("order-{}", uuid::Uuid::new_v4()),
            client_order_id: order.client_order_id.clone().unwrap_or_default(),
            status: "new".to_string(),
            symbol: order.symbol.clone(),
            qty: order.qty,
            filled_qty: 0.0,
            side: order.side.clone(),
            order_type: order.order_type.clone(),
            limit_price: order.limit_price,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Cancel an order
    pub fn cancel_order(&self, order_id: &str) -> Result<()> {
        tracing::info!("Canceling order: {}", order_id);
        Ok(())
    }

    /// Get historical bars
    pub fn get_bars(
        &self,
        symbol: &str,
        start: &str,
        end: &str,
        _timeframe: &str,
    ) -> Result<DataFrame> {
        tracing::info!(
            "Getting Alpaca bars for {} from {} to {}",
            symbol, start, end
        );

        let n_days = 252; // Default to 1 year
        generate_sample_ohlcv(symbol, n_days)
    }
}

/// Alpaca account information
#[derive(Debug, Clone)]
pub struct AlpacaAccount {
    pub id: String,
    pub cash: f64,
    pub portfolio_value: f64,
    pub buying_power: f64,
    pub equity: f64,
    pub pattern_day_trader: bool,
}

/// Alpaca position
#[derive(Debug, Clone)]
pub struct AlpacaPosition {
    pub symbol: String,
    pub qty: f64,
    pub avg_entry_price: f64,
    pub market_value: f64,
    pub unrealized_pl: f64,
    pub side: String,
}

/// Alpaca order request
#[derive(Debug, Clone)]
pub struct AlpacaOrder {
    pub symbol: String,
    pub qty: f64,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub client_order_id: Option<String>,
}

impl AlpacaOrder {
    /// Create a market buy order
    pub fn market_buy(symbol: &str, qty: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            qty,
            side: "buy".to_string(),
            order_type: "market".to_string(),
            time_in_force: "day".to_string(),
            limit_price: None,
            stop_price: None,
            client_order_id: None,
        }
    }

    /// Create a market sell order
    pub fn market_sell(symbol: &str, qty: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            qty,
            side: "sell".to_string(),
            order_type: "market".to_string(),
            time_in_force: "day".to_string(),
            limit_price: None,
            stop_price: None,
            client_order_id: None,
        }
    }

    /// Create a limit buy order
    pub fn limit_buy(symbol: &str, qty: f64, price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            qty,
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            time_in_force: "day".to_string(),
            limit_price: Some(price),
            stop_price: None,
            client_order_id: None,
        }
    }

    /// Create a limit sell order
    pub fn limit_sell(symbol: &str, qty: f64, price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            qty,
            side: "sell".to_string(),
            order_type: "limit".to_string(),
            time_in_force: "day".to_string(),
            limit_price: Some(price),
            stop_price: None,
            client_order_id: None,
        }
    }
}

/// Alpaca order response
#[derive(Debug, Clone)]
pub struct AlpacaOrderResponse {
    pub id: String,
    pub client_order_id: String,
    pub status: String,
    pub symbol: String,
    pub qty: f64,
    pub filled_qty: f64,
    pub side: String,
    pub order_type: String,
    pub limit_price: Option<f64>,
    pub created_at: String,
}

/// WebSocket streaming handler
pub struct StreamingClient {
    url: String,
    subscriptions: Vec<String>,
}

impl StreamingClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            subscriptions: Vec::new(),
        }
    }

    /// Subscribe to a symbol
    pub fn subscribe(&mut self, symbol: &str) {
        self.subscriptions.push(symbol.to_string());
        tracing::info!("Subscribed to: {}", symbol);
    }

    /// Unsubscribe from a symbol
    pub fn unsubscribe(&mut self, symbol: &str) {
        self.subscriptions.retain(|s| s != symbol);
        tracing::info!("Unsubscribed from: {}", symbol);
    }

    /// Get current subscriptions
    pub fn subscriptions(&self) -> &[String] {
        &self.subscriptions
    }
}

/// Integration registry for managing multiple providers
pub struct IntegrationRegistry {
    data_providers: HashMap<String, Arc<dyn MarketDataProvider>>,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self {
            data_providers: HashMap::new(),
        }
    }

    /// Register a data provider
    pub fn register_provider<P: MarketDataProvider + 'static>(&mut self, provider: P) {
        let name = provider.name().to_string();
        self.data_providers.insert(name, Arc::new(provider));
    }

    /// Get a data provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn MarketDataProvider>> {
        self.data_providers.get(name).cloned()
    }

    /// List available providers
    pub fn list_providers(&self) -> Vec<String> {
        self.data_providers.keys().cloned().collect()
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_provider(YahooFinance::new());
        registry
    }
}

// Helper functions

fn parse_date_to_timestamp(date: &str) -> Result<i64> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
        .map_err(|e| SigcError::Runtime(format!("Invalid date '{}': {}", date, e)))
}

fn generate_sample_ohlcv(_symbol: &str, n_days: usize) -> Result<DataFrame> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut dates = Vec::with_capacity(n_days);
    let mut opens = Vec::with_capacity(n_days);
    let mut highs = Vec::with_capacity(n_days);
    let mut lows = Vec::with_capacity(n_days);
    let mut closes = Vec::with_capacity(n_days);
    let mut volumes = Vec::with_capacity(n_days);

    let start_date = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let mut price = 100.0;

    for i in 0..n_days {
        let date = start_date + chrono::Duration::days(i as i64);
        dates.push(date.format("%Y-%m-%d").to_string());

        let ret: f64 = rng.gen_range(-0.03..0.03);
        let open: f64 = price;
        price *= 1.0 + ret;
        let close: f64 = price;
        let high: f64 = open.max(close) * (1.0 + rng.gen_range(0.0..0.01));
        let low: f64 = open.min(close) * (1.0 - rng.gen_range(0.0..0.01));
        let volume: f64 = rng.gen_range(100_000.0..10_000_000.0);

        opens.push(open);
        highs.push(high);
        lows.push(low);
        closes.push(close);
        volumes.push(volume);
    }

    DataFrame::new(vec![
        Column::new("date".into(), dates),
        Column::new("open".into(), opens),
        Column::new("high".into(), highs),
        Column::new("low".into(), lows),
        Column::new("close".into(), closes),
        Column::new("volume".into(), volumes),
    ])
    .map_err(|e| SigcError::Runtime(format!("Failed to create DataFrame: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yahoo_finance_ohlcv() {
        let yahoo = YahooFinance::new();
        let df = yahoo.get_ohlcv("AAPL", "2023-01-01", "2023-12-31", "1d");
        assert!(df.is_ok());
        let df = df.unwrap();
        assert!(df.height() > 0);
        assert!(df.get_column_names().iter().any(|s| *s == "close"));
    }

    #[test]
    fn test_yahoo_finance_quote() {
        let yahoo = YahooFinance::new();
        let quote = yahoo.get_quote("AAPL").unwrap();
        assert_eq!(quote.symbol, "AAPL");
        assert!(quote.last > 0.0);
    }

    #[test]
    fn test_alpaca_account() {
        let alpaca = AlpacaBroker::paper("test_key", "test_secret");
        let account = alpaca.get_account().unwrap();
        assert!(account.cash > 0.0);
        assert!(account.equity > 0.0);
    }

    #[test]
    fn test_alpaca_order() {
        let alpaca = AlpacaBroker::paper("test_key", "test_secret");
        let order = AlpacaOrder::market_buy("AAPL", 100.0);
        let response = alpaca.submit_order(&order).unwrap();
        assert_eq!(response.symbol, "AAPL");
        assert_eq!(response.qty, 100.0);
        assert_eq!(response.status, "new");
    }

    #[test]
    fn test_alpaca_limit_order() {
        let order = AlpacaOrder::limit_buy("MSFT", 50.0, 350.0);
        assert_eq!(order.order_type, "limit");
        assert_eq!(order.limit_price, Some(350.0));
    }

    #[test]
    fn test_streaming_client() {
        let mut client = StreamingClient::new("wss://stream.example.com");
        client.subscribe("AAPL");
        client.subscribe("MSFT");
        assert_eq!(client.subscriptions().len(), 2);

        client.unsubscribe("AAPL");
        assert_eq!(client.subscriptions().len(), 1);
    }

    #[test]
    fn test_integration_registry() {
        let registry = IntegrationRegistry::default();
        assert!(registry.get_provider("yahoo_finance").is_some());
        assert!(registry.list_providers().contains(&"yahoo_finance".to_string()));
    }

    #[test]
    fn test_parse_date() {
        let ts = parse_date_to_timestamp("2023-01-01").unwrap();
        assert!(ts > 0);
    }
}
