//! Result persistence for backtest results
//!
//! Stores and retrieves backtest results from various backends.

use sig_types::{BacktestReport, BacktestMetrics, Result, SigcError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Unique identifier for a stored result
pub type ResultId = String;

/// Metadata about a stored result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResultMetadata {
    pub id: ResultId,
    pub strategy_name: String,
    pub strategy_version: Option<String>,
    pub created_at: String,
    pub start_date: String,
    pub end_date: String,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub tags: HashMap<String, String>,
}

/// Query parameters for retrieving results
#[derive(Debug, Clone, Default)]
pub struct ResultQuery {
    pub strategy_name: Option<String>,
    pub strategy_version: Option<String>,
    pub start_date_after: Option<String>,
    pub end_date_before: Option<String>,
    pub min_sharpe: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub tags: HashMap<String, String>,
    pub limit: Option<usize>,
    pub order_by: Option<String>,
}

impl ResultQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn strategy(mut self, name: &str) -> Self {
        self.strategy_name = Some(name.to_string());
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.strategy_version = Some(version.to_string());
        self
    }

    pub fn min_sharpe(mut self, sharpe: f64) -> Self {
        self.min_sharpe = Some(sharpe);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by = Some(field.to_string());
        self
    }

    pub fn tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
}

/// Trait for storing and retrieving backtest results
pub trait ResultStore: Send + Sync {
    /// Store a backtest result
    fn store(&self, report: &BacktestReport, metadata: ResultMetadata) -> Result<ResultId>;

    /// Retrieve a result by ID
    fn get(&self, id: &ResultId) -> Result<Option<(BacktestReport, ResultMetadata)>>;

    /// Query results by criteria
    fn query(&self, query: &ResultQuery) -> Result<Vec<ResultMetadata>>;

    /// Delete a result
    fn delete(&self, id: &ResultId) -> Result<bool>;

    /// List all result IDs
    fn list_ids(&self) -> Result<Vec<ResultId>>;

    /// Get store name
    fn name(&self) -> &str;

    /// Check if store is available
    fn is_available(&self) -> bool;
}

/// In-memory result store for testing
pub struct MemoryResultStore {
    results: Arc<RwLock<HashMap<ResultId, (BacktestReport, ResultMetadata)>>>,
}

impl MemoryResultStore {
    pub fn new() -> Self {
        MemoryResultStore {
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryResultStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultStore for MemoryResultStore {
    fn store(&self, report: &BacktestReport, metadata: ResultMetadata) -> Result<ResultId> {
        let id = metadata.id.clone();
        let mut results = self.results.write()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;
        results.insert(id.clone(), (report.clone(), metadata));
        Ok(id)
    }

    fn get(&self, id: &ResultId) -> Result<Option<(BacktestReport, ResultMetadata)>> {
        let results = self.results.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;
        Ok(results.get(id).cloned())
    }

    fn query(&self, query: &ResultQuery) -> Result<Vec<ResultMetadata>> {
        let results = self.results.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        let mut matched: Vec<ResultMetadata> = results
            .values()
            .filter(|(_, meta)| {
                // Filter by strategy name
                if let Some(ref name) = query.strategy_name {
                    if &meta.strategy_name != name {
                        return false;
                    }
                }

                // Filter by version
                if let Some(ref version) = query.strategy_version {
                    if meta.strategy_version.as_ref() != Some(version) {
                        return false;
                    }
                }

                // Filter by min sharpe
                if let Some(min_sharpe) = query.min_sharpe {
                    if meta.sharpe_ratio < min_sharpe {
                        return false;
                    }
                }

                // Filter by max drawdown
                if let Some(max_dd) = query.max_drawdown {
                    if meta.max_drawdown > max_dd {
                        return false;
                    }
                }

                // Filter by tags
                for (key, value) in &query.tags {
                    if meta.tags.get(key) != Some(value) {
                        return false;
                    }
                }

                true
            })
            .map(|(_, meta)| meta.clone())
            .collect();

        // Sort
        if let Some(ref order_by) = query.order_by {
            match order_by.as_str() {
                "sharpe_ratio" => matched.sort_by(|a, b| {
                    b.sharpe_ratio.partial_cmp(&a.sharpe_ratio).unwrap_or(std::cmp::Ordering::Equal)
                }),
                "total_return" => matched.sort_by(|a, b| {
                    b.total_return.partial_cmp(&a.total_return).unwrap_or(std::cmp::Ordering::Equal)
                }),
                "created_at" => matched.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                _ => {}
            }
        }

        // Limit
        if let Some(limit) = query.limit {
            matched.truncate(limit);
        }

        Ok(matched)
    }

    fn delete(&self, id: &ResultId) -> Result<bool> {
        let mut results = self.results.write()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;
        Ok(results.remove(id).is_some())
    }

    fn list_ids(&self) -> Result<Vec<ResultId>> {
        let results = self.results.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;
        Ok(results.keys().cloned().collect())
    }

    fn name(&self) -> &str {
        "memory"
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// PostgreSQL result store
pub struct PostgresResultStore {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

impl PostgresResultStore {
    pub fn new(host: &str, port: u16, database: &str, user: &str, password: &str) -> Self {
        PostgresResultStore {
            host: host.to_string(),
            port,
            database: database.to_string(),
            user: user.to_string(),
            password: password.to_string(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        let host = std::env::var("PGHOST").ok()?;
        let port: u16 = std::env::var("PGPORT").ok()?.parse().ok()?;
        let database = std::env::var("PGDATABASE").ok()?;
        let user = std::env::var("PGUSER").ok()?;
        let password = std::env::var("PGPASSWORD").ok()?;

        Some(Self::new(&host, port, &database, &user, &password))
    }

    fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.user, self.password
        )
    }

    fn get_client(&self) -> Result<postgres::Client> {
        postgres::Client::connect(&self.connection_string(), postgres::NoTls)
            .map_err(|e| SigcError::Runtime(format!("Failed to connect: {}", e)))
    }

    /// Initialize database schema
    pub fn init_schema(&self) -> Result<()> {
        let mut client = self.get_client()?;

        client.batch_execute(r#"
            CREATE TABLE IF NOT EXISTS backtest_results (
                id TEXT PRIMARY KEY,
                strategy_name TEXT NOT NULL,
                strategy_version TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                total_return DOUBLE PRECISION NOT NULL,
                annualized_return DOUBLE PRECISION,
                sharpe_ratio DOUBLE PRECISION NOT NULL,
                max_drawdown DOUBLE PRECISION NOT NULL,
                sortino_ratio DOUBLE PRECISION,
                calmar_ratio DOUBLE PRECISION,
                win_rate DOUBLE PRECISION,
                profit_factor DOUBLE PRECISION,
                turnover DOUBLE PRECISION,
                returns_series TEXT NOT NULL,
                tags TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_results_strategy ON backtest_results(strategy_name);
            CREATE INDEX IF NOT EXISTS idx_results_created ON backtest_results(created_at);
            CREATE INDEX IF NOT EXISTS idx_results_sharpe ON backtest_results(sharpe_ratio);
        "#).map_err(|e| SigcError::Runtime(format!("Schema init failed: {}", e)))?;

        Ok(())
    }
}

impl ResultStore for PostgresResultStore {
    fn store(&self, report: &BacktestReport, metadata: ResultMetadata) -> Result<ResultId> {
        let mut client = self.get_client()?;

        // Serialize returns_series as comma-separated string
        let returns_str: String = report.returns_series
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Serialize tags as key=value pairs
        let tags_str: String = metadata.tags
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(";");

        client.execute(
            r#"
            INSERT INTO backtest_results
            (id, strategy_name, strategy_version, start_date, end_date,
             total_return, annualized_return, sharpe_ratio, max_drawdown,
             sortino_ratio, calmar_ratio, win_rate, profit_factor, turnover,
             returns_series, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (id) DO UPDATE SET
                returns_series = EXCLUDED.returns_series,
                tags = EXCLUDED.tags
            "#,
            &[
                &metadata.id,
                &metadata.strategy_name,
                &metadata.strategy_version,
                &metadata.start_date,
                &metadata.end_date,
                &report.metrics.total_return,
                &report.metrics.annualized_return,
                &report.metrics.sharpe_ratio,
                &report.metrics.max_drawdown,
                &report.metrics.sortino_ratio,
                &report.metrics.calmar_ratio,
                &report.metrics.win_rate,
                &report.metrics.profit_factor,
                &report.metrics.turnover,
                &returns_str,
                &tags_str,
            ],
        ).map_err(|e| SigcError::Runtime(format!("Insert failed: {}", e)))?;

        Ok(metadata.id)
    }

    fn get(&self, id: &ResultId) -> Result<Option<(BacktestReport, ResultMetadata)>> {
        let mut client = self.get_client()?;

        let row = client.query_opt(
            r#"
            SELECT id, strategy_name, strategy_version, created_at::text,
                   start_date, end_date, total_return, annualized_return,
                   sharpe_ratio, max_drawdown, sortino_ratio, calmar_ratio,
                   win_rate, profit_factor, turnover, returns_series, tags
            FROM backtest_results WHERE id = $1
            "#,
            &[id],
        ).map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

        match row {
            Some(row) => {
                // Parse returns series
                let returns_str: String = row.get("returns_series");
                let returns_series: Vec<f64> = returns_str
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| s.parse().ok())
                    .collect();

                // Parse tags
                let tags_str: String = row.get("tags");
                let tags: HashMap<String, String> = tags_str
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            Some((parts[0].to_string(), parts[1].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                let created_at: String = row.get("created_at");

                let report = BacktestReport {
                    metrics: BacktestMetrics {
                        total_return: row.get("total_return"),
                        annualized_return: row.get("annualized_return"),
                        sharpe_ratio: row.get("sharpe_ratio"),
                        max_drawdown: row.get("max_drawdown"),
                        sortino_ratio: row.get("sortino_ratio"),
                        calmar_ratio: row.get("calmar_ratio"),
                        win_rate: row.get("win_rate"),
                        profit_factor: row.get("profit_factor"),
                        turnover: row.get("turnover"),
                    },
                    returns_series,
                    positions: None,
                    benchmark_metrics: None,
                    executed_at: 0, // Timestamp not stored, use 0
                    plan_hash: String::new(),
                };

                let metadata = ResultMetadata {
                    id: row.get("id"),
                    strategy_name: row.get("strategy_name"),
                    strategy_version: row.get("strategy_version"),
                    created_at,
                    start_date: row.get("start_date"),
                    end_date: row.get("end_date"),
                    total_return: row.get("total_return"),
                    sharpe_ratio: row.get("sharpe_ratio"),
                    max_drawdown: row.get("max_drawdown"),
                    tags,
                };

                Ok(Some((report, metadata)))
            }
            None => Ok(None),
        }
    }

    fn query(&self, query: &ResultQuery) -> Result<Vec<ResultMetadata>> {
        let mut client = self.get_client()?;

        let mut sql = String::from(
            "SELECT id, strategy_name, strategy_version, created_at::text,
                    start_date, end_date, total_return, sharpe_ratio, max_drawdown, tags
             FROM backtest_results WHERE 1=1"
        );

        let mut params: Vec<Box<dyn postgres::types::ToSql + Sync>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref name) = query.strategy_name {
            sql.push_str(&format!(" AND strategy_name = ${}", param_idx));
            params.push(Box::new(name.clone()));
            param_idx += 1;
        }

        if let Some(ref version) = query.strategy_version {
            sql.push_str(&format!(" AND strategy_version = ${}", param_idx));
            params.push(Box::new(version.clone()));
            param_idx += 1;
        }

        if let Some(min_sharpe) = query.min_sharpe {
            sql.push_str(&format!(" AND sharpe_ratio >= ${}", param_idx));
            params.push(Box::new(min_sharpe));
            param_idx += 1;
        }

        if let Some(max_dd) = query.max_drawdown {
            sql.push_str(&format!(" AND max_drawdown <= ${}", param_idx));
            params.push(Box::new(max_dd));
            let _ = param_idx;
        }

        // Order by
        if let Some(ref order_by) = query.order_by {
            let order_col = match order_by.as_str() {
                "sharpe_ratio" => "sharpe_ratio DESC",
                "total_return" => "total_return DESC",
                "created_at" => "created_at DESC",
                _ => "created_at DESC",
            };
            sql.push_str(&format!(" ORDER BY {}", order_col));
        }

        // Limit
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // Convert params for query
        let param_refs: Vec<&(dyn postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref()).collect();

        let rows = client.query(&sql, &param_refs)
            .map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

        let results: Vec<ResultMetadata> = rows
            .iter()
            .map(|row| {
                // Parse tags
                let tags_str: String = row.get("tags");
                let tags: HashMap<String, String> = tags_str
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            Some((parts[0].to_string(), parts[1].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                ResultMetadata {
                    id: row.get("id"),
                    strategy_name: row.get("strategy_name"),
                    strategy_version: row.get("strategy_version"),
                    created_at: row.get("created_at"),
                    start_date: row.get("start_date"),
                    end_date: row.get("end_date"),
                    total_return: row.get("total_return"),
                    sharpe_ratio: row.get("sharpe_ratio"),
                    max_drawdown: row.get("max_drawdown"),
                    tags,
                }
            })
            .collect();

        Ok(results)
    }

    fn delete(&self, id: &ResultId) -> Result<bool> {
        let mut client = self.get_client()?;

        let rows_affected = client.execute(
            "DELETE FROM backtest_results WHERE id = $1",
            &[id],
        ).map_err(|e| SigcError::Runtime(format!("Delete failed: {}", e)))?;

        Ok(rows_affected > 0)
    }

    fn list_ids(&self) -> Result<Vec<ResultId>> {
        let mut client = self.get_client()?;

        let rows = client.query(
            "SELECT id FROM backtest_results ORDER BY created_at DESC",
            &[],
        ).map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

        Ok(rows.iter().map(|row| row.get("id")).collect())
    }

    fn name(&self) -> &str {
        "postgres"
    }

    fn is_available(&self) -> bool {
        self.get_client().is_ok()
    }
}

/// Registry for managing multiple result stores
pub struct ResultStoreRegistry {
    stores: HashMap<String, Box<dyn ResultStore>>,
    default: Option<String>,
}

impl ResultStoreRegistry {
    pub fn new() -> Self {
        ResultStoreRegistry {
            stores: HashMap::new(),
            default: None,
        }
    }

    pub fn register(&mut self, name: &str, store: Box<dyn ResultStore>) {
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
        self.stores.insert(name.to_string(), store);
    }

    pub fn set_default(&mut self, name: &str) {
        if self.stores.contains_key(name) {
            self.default = Some(name.to_string());
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn ResultStore> {
        self.stores.get(name).map(|s| s.as_ref())
    }

    pub fn default_store(&self) -> Option<&dyn ResultStore> {
        self.default.as_ref().and_then(|name| self.get(name))
    }

    pub fn list(&self) -> Vec<String> {
        self.stores.keys().cloned().collect()
    }
}

impl Default for ResultStoreRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a unique result ID
pub fn generate_result_id() -> ResultId {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let random: u32 = rand::random();
    format!("{:x}-{:08x}", timestamp, random)
}

/// Result comparison between two backtests
#[derive(Debug, Clone)]
pub struct ResultComparison {
    pub result_a: ResultMetadata,
    pub result_b: ResultMetadata,
    pub return_diff: f64,
    pub sharpe_diff: f64,
    pub drawdown_diff: f64,
    pub winner: ComparisonWinner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonWinner {
    A,
    B,
    Tie,
}

impl ResultComparison {
    /// Compare two results
    pub fn compare(a: &ResultMetadata, b: &ResultMetadata) -> Self {
        let return_diff = a.total_return - b.total_return;
        let sharpe_diff = a.sharpe_ratio - b.sharpe_ratio;
        let drawdown_diff = b.max_drawdown - a.max_drawdown; // Lower is better

        // Determine winner based on Sharpe ratio
        let winner = if sharpe_diff > 0.1 {
            ComparisonWinner::A
        } else if sharpe_diff < -0.1 {
            ComparisonWinner::B
        } else {
            ComparisonWinner::Tie
        };

        ResultComparison {
            result_a: a.clone(),
            result_b: b.clone(),
            return_diff,
            sharpe_diff,
            drawdown_diff,
            winner,
        }
    }

    /// Format comparison as a table
    pub fn to_table(&self) -> String {
        format!(
            "| Metric | {} | {} | Diff |\n\
             |--------|------|------|------|\n\
             | Return | {:.2}% | {:.2}% | {:+.2}% |\n\
             | Sharpe | {:.2} | {:.2} | {:+.2} |\n\
             | MaxDD | {:.2}% | {:.2}% | {:+.2}% |",
            self.result_a.id, self.result_b.id,
            self.result_a.total_return * 100.0, self.result_b.total_return * 100.0, self.return_diff * 100.0,
            self.result_a.sharpe_ratio, self.result_b.sharpe_ratio, self.sharpe_diff,
            self.result_a.max_drawdown * 100.0, self.result_b.max_drawdown * 100.0, self.drawdown_diff * 100.0
        )
    }
}

/// Historical performance tracking for a strategy
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    pub strategy_name: String,
    pub results: Vec<ResultMetadata>,
}

impl PerformanceHistory {
    /// Create from query results
    pub fn from_results(strategy_name: &str, results: Vec<ResultMetadata>) -> Self {
        PerformanceHistory {
            strategy_name: strategy_name.to_string(),
            results,
        }
    }

    /// Get best result by Sharpe ratio
    pub fn best_by_sharpe(&self) -> Option<&ResultMetadata> {
        self.results.iter().max_by(|a, b| {
            a.sharpe_ratio.partial_cmp(&b.sharpe_ratio).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get best result by total return
    pub fn best_by_return(&self) -> Option<&ResultMetadata> {
        self.results.iter().max_by(|a, b| {
            a.total_return.partial_cmp(&b.total_return).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get average metrics across all results
    pub fn average_metrics(&self) -> Option<(f64, f64, f64)> {
        if self.results.is_empty() {
            return None;
        }

        let n = self.results.len() as f64;
        let avg_return = self.results.iter().map(|r| r.total_return).sum::<f64>() / n;
        let avg_sharpe = self.results.iter().map(|r| r.sharpe_ratio).sum::<f64>() / n;
        let avg_drawdown = self.results.iter().map(|r| r.max_drawdown).sum::<f64>() / n;

        Some((avg_return, avg_sharpe, avg_drawdown))
    }

    /// Get performance trend (improving/declining/stable)
    pub fn trend(&self) -> PerformanceTrend {
        if self.results.len() < 2 {
            return PerformanceTrend::Insufficient;
        }

        // Compare first half to second half
        let mid = self.results.len() / 2;
        let first_half: f64 = self.results[..mid].iter().map(|r| r.sharpe_ratio).sum::<f64>() / mid as f64;
        let second_half: f64 = self.results[mid..].iter().map(|r| r.sharpe_ratio).sum::<f64>() / (self.results.len() - mid) as f64;

        let diff = second_half - first_half;
        if diff > 0.2 {
            PerformanceTrend::Improving
        } else if diff < -0.2 {
            PerformanceTrend::Declining
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Count results
    pub fn count(&self) -> usize {
        self.results.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTrend {
    Improving,
    Declining,
    Stable,
    Insufficient,
}

impl std::fmt::Display for PerformanceTrend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceTrend::Improving => write!(f, "IMPROVING"),
            PerformanceTrend::Declining => write!(f, "DECLINING"),
            PerformanceTrend::Stable => write!(f, "STABLE"),
            PerformanceTrend::Insufficient => write!(f, "INSUFFICIENT DATA"),
        }
    }
}

/// Helper to load performance history from a store
pub fn load_performance_history(
    store: &dyn ResultStore,
    strategy_name: &str,
) -> Result<PerformanceHistory> {
    let query = ResultQuery::new()
        .strategy(strategy_name)
        .order_by("created_at");

    let results = store.query(&query)?;
    Ok(PerformanceHistory::from_results(strategy_name, results))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_report() -> BacktestReport {
        BacktestReport {
            metrics: BacktestMetrics {
                total_return: 0.15,
                annualized_return: 0.12,
                sharpe_ratio: 1.5,
                max_drawdown: 0.08,
                turnover: 2.0,
                sortino_ratio: 2.0,
                calmar_ratio: 1.5,
                win_rate: 0.55,
                profit_factor: 1.8,
            },
            returns_series: vec![0.01, 0.02, -0.01, 0.03],
            positions: None,
            benchmark_metrics: None,
            executed_at: 1704110400, // 2024-01-01 12:00:00 UTC
            plan_hash: "test-hash".to_string(),
        }
    }

    fn create_test_metadata(id: &str) -> ResultMetadata {
        let mut tags = HashMap::new();
        tags.insert("type".to_string(), "momentum".to_string());

        ResultMetadata {
            id: id.to_string(),
            strategy_name: "test_strategy".to_string(),
            strategy_version: Some("1.0".to_string()),
            created_at: "2024-01-01 12:00:00".to_string(),
            start_date: "2023-01-01".to_string(),
            end_date: "2023-12-31".to_string(),
            total_return: 0.15,
            sharpe_ratio: 1.5,
            max_drawdown: 0.08,
            tags,
        }
    }

    #[test]
    fn test_memory_store_crud() {
        let store = MemoryResultStore::new();
        let report = create_test_report();
        let metadata = create_test_metadata("test-1");

        // Store
        let id = store.store(&report, metadata.clone()).unwrap();
        assert_eq!(id, "test-1");

        // Get
        let (retrieved, meta) = store.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.metrics.total_return, 0.15);
        assert_eq!(meta.strategy_name, "test_strategy");

        // List
        let ids = store.list_ids().unwrap();
        assert_eq!(ids.len(), 1);

        // Delete
        let deleted = store.delete(&id).unwrap();
        assert!(deleted);
        assert!(store.get(&id).unwrap().is_none());
    }

    #[test]
    fn test_memory_store_query() {
        let store = MemoryResultStore::new();

        // Add multiple results
        for i in 0..5 {
            let report = create_test_report();
            let mut metadata = create_test_metadata(&format!("test-{}", i));
            metadata.sharpe_ratio = i as f64 * 0.5;
            metadata.strategy_name = if i < 3 { "strategy_a" } else { "strategy_b" }.to_string();
            store.store(&report, metadata).unwrap();
        }

        // Query by strategy
        let query = ResultQuery::new().strategy("strategy_a");
        let results = store.query(&query).unwrap();
        assert_eq!(results.len(), 3);

        // Query with min sharpe
        let query = ResultQuery::new().min_sharpe(1.0);
        let results = store.query(&query).unwrap();
        assert_eq!(results.len(), 3); // sharpe >= 1.0

        // Query with limit and order
        let query = ResultQuery::new()
            .order_by("sharpe_ratio")
            .limit(2);
        let results = store.query(&query).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].sharpe_ratio >= results[1].sharpe_ratio);
    }

    #[test]
    fn test_result_query_builder() {
        let query = ResultQuery::new()
            .strategy("momentum")
            .version("2.0")
            .min_sharpe(1.5)
            .limit(10)
            .tag("type", "equity");

        assert_eq!(query.strategy_name.as_deref(), Some("momentum"));
        assert_eq!(query.strategy_version.as_deref(), Some("2.0"));
        assert_eq!(query.min_sharpe, Some(1.5));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.tags.get("type").map(|s| s.as_str()), Some("equity"));
    }

    #[test]
    fn test_generate_result_id() {
        let id1 = generate_result_id();
        let id2 = generate_result_id();

        assert!(!id1.is_empty());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_registry() {
        let mut registry = ResultStoreRegistry::new();
        registry.register("memory", Box::new(MemoryResultStore::new()));

        assert!(registry.get("memory").is_some());
        assert!(registry.default_store().is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_result_comparison() {
        let mut meta_a = create_test_metadata("result-a");
        meta_a.total_return = 0.20;
        meta_a.sharpe_ratio = 2.0;
        meta_a.max_drawdown = 0.05;

        let mut meta_b = create_test_metadata("result-b");
        meta_b.total_return = 0.10;
        meta_b.sharpe_ratio = 1.0;
        meta_b.max_drawdown = 0.10;

        let comparison = ResultComparison::compare(&meta_a, &meta_b);

        assert_eq!(comparison.winner, ComparisonWinner::A);
        assert!((comparison.return_diff - 0.10).abs() < 0.001);
        assert!((comparison.sharpe_diff - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_performance_history() {
        let mut results = Vec::new();

        // Add results with increasing sharpe (improving trend)
        for i in 0..6 {
            let mut meta = create_test_metadata(&format!("result-{}", i));
            meta.sharpe_ratio = 1.0 + i as f64 * 0.2;
            meta.total_return = 0.10 + i as f64 * 0.02;
            results.push(meta);
        }

        let history = PerformanceHistory::from_results("test_strategy", results);

        assert_eq!(history.count(), 6);
        assert_eq!(history.trend(), PerformanceTrend::Improving);

        let best = history.best_by_sharpe().unwrap();
        assert!((best.sharpe_ratio - 2.0).abs() < 0.001);

        let (avg_return, avg_sharpe, _) = history.average_metrics().unwrap();
        assert!(avg_return > 0.0);
        assert!(avg_sharpe > 1.0);
    }

    #[test]
    fn test_performance_trend() {
        // Declining trend
        let mut results = Vec::new();
        for i in 0..4 {
            let mut meta = create_test_metadata(&format!("result-{}", i));
            meta.sharpe_ratio = 2.0 - i as f64 * 0.3;
            results.push(meta);
        }

        let history = PerformanceHistory::from_results("declining", results);
        assert_eq!(history.trend(), PerformanceTrend::Declining);
    }
}
