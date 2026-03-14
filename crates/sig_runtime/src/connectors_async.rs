//! Async data connectors with connection pooling
//!
//! Provides high-performance async database access using sqlx.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column as SqlxColumn, Row};
use std::time::Duration;

/// Async PostgreSQL connector with connection pooling
pub struct AsyncPgConnector {
    pool: PgPool,
    name: String,
}

/// Configuration for async connector
#[derive(Debug, Clone)]
pub struct AsyncConnectorConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub query_timeout: Duration,
}

impl Default for AsyncConnectorConfig {
    fn default() -> Self {
        AsyncConnectorConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "sigc".to_string(),
            user: "postgres".to_string(),
            password: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            query_timeout: Duration::from_secs(300),
        }
    }
}

impl AsyncConnectorConfig {
    pub fn new(host: &str, port: u16, database: &str, user: &str, password: &str) -> Self {
        AsyncConnectorConfig {
            host: host.to_string(),
            port,
            database: database.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            ..Default::default()
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        Some(AsyncConnectorConfig {
            host: std::env::var("PGHOST").ok()?,
            port: std::env::var("PGPORT").ok()?.parse().ok()?,
            database: std::env::var("PGDATABASE").ok()?,
            user: std::env::var("PGUSER").ok()?,
            password: std::env::var("PGPASSWORD").ok()?,
            ..Default::default()
        })
    }

    pub fn with_max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    pub fn with_query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }

    fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

impl AsyncPgConnector {
    /// Create a new async PostgreSQL connector
    pub async fn new(config: AsyncConnectorConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.connect_timeout)
            .idle_timeout(config.idle_timeout)
            .connect(&config.connection_string())
            .await
            .map_err(|e| SigcError::Runtime(format!("Failed to create pool: {}", e)))?;

        Ok(AsyncPgConnector {
            pool,
            name: "async_postgres".to_string(),
        })
    }

    /// Create from environment variables
    pub async fn from_env() -> Result<Self> {
        let config = AsyncConnectorConfig::from_env()
            .ok_or_else(|| SigcError::Runtime("Missing PostgreSQL environment variables".into()))?;
        Self::new(config).await
    }

    /// Execute a query and return a DataFrame
    pub async fn load(&self, query: &str) -> Result<DataFrame> {
        let rows: Vec<PgRow> = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

        if rows.is_empty() {
            return Err(SigcError::Runtime("Query returned no rows".into()));
        }

        // Build DataFrame from rows
        self.rows_to_dataframe(&rows)
    }

    /// Execute a query with timeout
    pub async fn load_with_timeout(&self, query: &str, timeout: Duration) -> Result<DataFrame> {
        tokio::time::timeout(timeout, self.load(query))
            .await
            .map_err(|_| SigcError::Runtime("Query timed out".into()))?
    }

    /// Execute a count query
    pub async fn query_count(&self, query: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

        Ok(row.0)
    }

    /// Check if connection is healthy
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }

    /// Get connector name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Close the connection pool
    pub async fn close(self) {
        self.pool.close().await;
    }

    fn rows_to_dataframe(&self, rows: &[PgRow]) -> Result<DataFrame> {
        if rows.is_empty() {
            return Err(SigcError::Runtime("No rows to convert".into()));
        }

        let columns = rows[0].columns();
        let mut series_vec: Vec<Column> = Vec::new();

        for col in columns {
            let col_name = col.name();
            let type_info = col.type_info().to_string();

            match type_info.as_str() {
                "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                    let values: Vec<f64> = rows
                        .iter()
                        .map(|row| row.try_get::<f64, _>(col_name).unwrap_or(f64::NAN))
                        .collect();
                    series_vec.push(Column::new(col_name.into(), values));
                }
                "INT2" | "INT4" | "INT8" => {
                    let values: Vec<f64> = rows
                        .iter()
                        .map(|row| row.try_get::<i64, _>(col_name).unwrap_or(0) as f64)
                        .collect();
                    series_vec.push(Column::new(col_name.into(), values));
                }
                _ => {
                    // Handle as string
                    let values: Vec<String> = rows
                        .iter()
                        .map(|row| row.try_get::<String, _>(col_name).unwrap_or_default())
                        .collect();
                    series_vec.push(Column::new(col_name.into(), values));
                }
            }
        }

        DataFrame::new(series_vec)
            .map_err(|e| SigcError::Runtime(format!("Failed to create DataFrame: {}", e)))
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

/// Async connector registry
pub struct AsyncConnectorRegistry {
    connectors: std::collections::HashMap<String, AsyncPgConnector>,
}

impl AsyncConnectorRegistry {
    pub fn new() -> Self {
        AsyncConnectorRegistry {
            connectors: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, connector: AsyncPgConnector) {
        self.connectors.insert(name.to_string(), connector);
    }

    pub fn get(&self, name: &str) -> Option<&AsyncPgConnector> {
        self.connectors.get(name)
    }

    pub async fn load(&self, name: &str, query: &str) -> Result<DataFrame> {
        let connector = self.connectors.get(name)
            .ok_or_else(|| SigcError::Runtime(format!("Connector not found: {}", name)))?;
        connector.load(query).await
    }

    pub fn list(&self) -> Vec<String> {
        self.connectors.keys().cloned().collect()
    }

    pub async fn close_all(self) {
        for (_, connector) in self.connectors {
            connector.close().await;
        }
    }
}

impl Default for AsyncConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AsyncConnectorConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.port, 5432);
    }

    #[test]
    fn test_config_builder() {
        let config = AsyncConnectorConfig::new("db.example.com", 5433, "mydb", "user", "pass")
            .with_max_connections(20)
            .with_query_timeout(Duration::from_secs(60));

        assert_eq!(config.host, "db.example.com");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.query_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_connection_string() {
        let config = AsyncConnectorConfig::new("localhost", 5432, "testdb", "user", "pass");
        let conn_str = config.connection_string();
        assert!(conn_str.contains("postgres://"));
        assert!(conn_str.contains("localhost:5432"));
        assert!(conn_str.contains("testdb"));
    }

    #[test]
    fn test_registry() {
        let registry = AsyncConnectorRegistry::new();
        assert!(registry.list().is_empty());
    }
}
