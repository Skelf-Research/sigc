//! Data connectors for SQL databases and cloud storage
//!
//! Provides unified interface for loading data from various sources.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Connector configuration
#[derive(Debug, Clone)]
pub enum ConnectorConfig {
    /// PostgreSQL connection
    Postgres {
        host: String,
        port: u16,
        database: String,
        user: String,
        password: String,
    },
    /// Snowflake connection
    Snowflake {
        account: String,
        warehouse: String,
        database: String,
        schema: String,
        user: String,
        password: String,
    },
    /// AWS S3
    S3 {
        bucket: String,
        region: String,
        access_key: Option<String>,
        secret_key: Option<String>,
    },
    /// Google Cloud Storage
    Gcs {
        bucket: String,
        project: String,
        credentials_path: Option<String>,
    },
    /// Azure Blob Storage
    Azure {
        container: String,
        account: String,
        access_key: Option<String>,
    },
}

/// Data connector trait
pub trait Connector: Send + Sync {
    /// Load data from a path/query
    fn load(&self, path: &str) -> Result<DataFrame>;

    /// Check if connector is available
    fn is_available(&self) -> bool;

    /// Get connector name
    fn name(&self) -> &str;
}

/// SQL connector for databases
pub struct SqlConnector {
    config: ConnectorConfig,
    name: String,
}

impl SqlConnector {
    /// Create a PostgreSQL connector
    pub fn postgres(host: &str, port: u16, database: &str, user: &str, password: &str) -> Self {
        SqlConnector {
            config: ConnectorConfig::Postgres {
                host: host.to_string(),
                port,
                database: database.to_string(),
                user: user.to_string(),
                password: password.to_string(),
            },
            name: "postgres".to_string(),
        }
    }

    /// Create a Snowflake connector
    pub fn snowflake(
        account: &str,
        warehouse: &str,
        database: &str,
        schema: &str,
        user: &str,
        password: &str,
    ) -> Self {
        SqlConnector {
            config: ConnectorConfig::Snowflake {
                account: account.to_string(),
                warehouse: warehouse.to_string(),
                database: database.to_string(),
                schema: schema.to_string(),
                user: user.to_string(),
                password: password.to_string(),
            },
            name: "snowflake".to_string(),
        }
    }

    /// Build connection string
    #[allow(dead_code)]
    fn connection_string(&self) -> String {
        match &self.config {
            ConnectorConfig::Postgres { host, port, database, user, password } => {
                format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, database)
            }
            ConnectorConfig::Snowflake { account, warehouse, database, schema, user, password } => {
                format!(
                    "snowflake://{}:{}@{}/{}/{}?warehouse={}",
                    user, password, account, database, schema, warehouse
                )
            }
            _ => String::new(),
        }
    }
}

impl Connector for SqlConnector {
    fn load(&self, query: &str) -> Result<DataFrame> {
        match &self.config {
            ConnectorConfig::Postgres { host, port, database, user, password } => {
                self.load_postgres(host, *port, database, user, password, query)
            }
            ConnectorConfig::Snowflake { .. } => {
                // Snowflake requires their specific driver
                Err(SigcError::Runtime(
                    "Snowflake connector requires snowflake-connector. Use ODBC or REST API.".into()
                ))
            }
            _ => Err(SigcError::Runtime("Invalid config for SQL connector".into())),
        }
    }

    fn is_available(&self) -> bool {
        match &self.config {
            ConnectorConfig::Postgres { host, port, database, user, password } => {
                let conn_str = format!(
                    "host={} port={} dbname={} user={} password={}",
                    host, port, database, user, password
                );
                postgres::Client::connect(&conn_str, postgres::NoTls).is_ok()
            }
            _ => false,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl SqlConnector {
    /// Load data from PostgreSQL
    fn load_postgres(
        &self,
        host: &str,
        port: u16,
        database: &str,
        user: &str,
        password: &str,
        query: &str,
    ) -> Result<DataFrame> {
        let conn_str = format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database, user, password
        );

        let mut client = postgres::Client::connect(&conn_str, postgres::NoTls)
            .map_err(|e| SigcError::Runtime(format!("Failed to connect to Postgres: {}", e)))?;

        let rows = client.query(query, &[])
            .map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

        if rows.is_empty() {
            return Err(SigcError::Runtime("Query returned no rows".into()));
        }

        // Get column info from first row
        let columns = rows[0].columns();
        let mut series_data: Vec<(String, Vec<f64>)> = Vec::new();
        let mut string_data: Vec<(String, Vec<String>)> = Vec::new();

        // Initialize columns
        for col in columns {
            let name = col.name().to_string();
            let type_name = col.type_().name();

            match type_name {
                "float4" | "float8" | "numeric" | "int2" | "int4" | "int8" => {
                    series_data.push((name, Vec::with_capacity(rows.len())));
                }
                "text" | "varchar" | "date" | "timestamp" | "timestamptz" => {
                    string_data.push((name, Vec::with_capacity(rows.len())));
                }
                _ => {
                    // Try as string
                    string_data.push((name, Vec::with_capacity(rows.len())));
                }
            }
        }

        // Extract data from rows
        for row in &rows {
            let mut float_idx = 0;
            let mut string_idx = 0;

            for (i, col) in columns.iter().enumerate() {
                let type_name = col.type_().name();

                match type_name {
                    "float4" => {
                        let val: Option<f32> = row.get(i);
                        series_data[float_idx].1.push(val.map(|v| v as f64).unwrap_or(f64::NAN));
                        float_idx += 1;
                    }
                    "float8" | "numeric" => {
                        let val: Option<f64> = row.get(i);
                        series_data[float_idx].1.push(val.unwrap_or(f64::NAN));
                        float_idx += 1;
                    }
                    "int2" => {
                        let val: Option<i16> = row.get(i);
                        series_data[float_idx].1.push(val.unwrap_or(0) as f64);
                        float_idx += 1;
                    }
                    "int4" => {
                        let val: Option<i32> = row.get(i);
                        series_data[float_idx].1.push(val.unwrap_or(0) as f64);
                        float_idx += 1;
                    }
                    "int8" => {
                        let val: Option<i64> = row.get(i);
                        series_data[float_idx].1.push(val.unwrap_or(0) as f64);
                        float_idx += 1;
                    }
                    _ => {
                        // Handle dates, timestamps, and strings as strings
                        let val: Option<String> = row.try_get(i).ok().flatten();
                        string_data[string_idx].1.push(val.unwrap_or_default());
                        string_idx += 1;
                    }
                }
            }
        }

        // Build DataFrame
        let mut df_columns: Vec<Column> = Vec::new();

        for (name, values) in series_data {
            df_columns.push(Column::new(name.into(), values));
        }

        for (name, values) in string_data {
            df_columns.push(Column::new(name.into(), values));
        }

        DataFrame::new(df_columns)
            .map_err(|e| SigcError::Runtime(format!("Failed to create DataFrame: {}", e)))
    }

    /// Execute a query that returns a count
    pub fn query_count(&self, query: &str) -> Result<i64> {
        match &self.config {
            ConnectorConfig::Postgres { host, port, database, user, password } => {
                let conn_str = format!(
                    "host={} port={} dbname={} user={} password={}",
                    host, port, database, user, password
                );

                let mut client = postgres::Client::connect(&conn_str, postgres::NoTls)
                    .map_err(|e| SigcError::Runtime(format!("Connection failed: {}", e)))?;

                let row = client.query_one(query, &[])
                    .map_err(|e| SigcError::Runtime(format!("Query failed: {}", e)))?;

                let value: i64 = row.get(0);
                Ok(value)
            }
            _ => Err(SigcError::Runtime("Not a Postgres connector".into())),
        }
    }

    /// List available tables
    pub fn list_tables(&self) -> Result<Vec<String>> {
        let query = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";
        let df = self.load(query)?;

        let col = df.column("table_name")
            .map_err(|e| SigcError::Runtime(format!("Column error: {}", e)))?;

        let tables: Vec<String> = col.str()
            .map_err(|e| SigcError::Runtime(format!("Cast error: {}", e)))?
            .into_iter()
            .filter_map(|s| s.map(|s| s.to_string()))
            .collect();

        Ok(tables)
    }
}

/// Cloud storage connector
pub struct CloudConnector {
    config: ConnectorConfig,
    name: String,
}

impl CloudConnector {
    /// Create an S3 connector
    pub fn s3(bucket: &str, region: &str) -> Self {
        CloudConnector {
            config: ConnectorConfig::S3 {
                bucket: bucket.to_string(),
                region: region.to_string(),
                access_key: None,
                secret_key: None,
            },
            name: "s3".to_string(),
        }
    }

    /// Create an S3 connector with credentials
    pub fn s3_with_credentials(bucket: &str, region: &str, access_key: &str, secret_key: &str) -> Self {
        CloudConnector {
            config: ConnectorConfig::S3 {
                bucket: bucket.to_string(),
                region: region.to_string(),
                access_key: Some(access_key.to_string()),
                secret_key: Some(secret_key.to_string()),
            },
            name: "s3".to_string(),
        }
    }

    /// Create a GCS connector
    pub fn gcs(bucket: &str, project: &str) -> Self {
        CloudConnector {
            config: ConnectorConfig::Gcs {
                bucket: bucket.to_string(),
                project: project.to_string(),
                credentials_path: None,
            },
            name: "gcs".to_string(),
        }
    }

    /// Create an Azure connector
    pub fn azure(container: &str, account: &str) -> Self {
        CloudConnector {
            config: ConnectorConfig::Azure {
                container: container.to_string(),
                account: account.to_string(),
                access_key: None,
            },
            name: "azure".to_string(),
        }
    }

    /// Get the full URI for a path
    fn get_uri(&self, path: &str) -> String {
        match &self.config {
            ConnectorConfig::S3 { bucket, .. } => {
                format!("s3://{}/{}", bucket, path)
            }
            ConnectorConfig::Gcs { bucket, .. } => {
                format!("gs://{}/{}", bucket, path)
            }
            ConnectorConfig::Azure { container, account, .. } => {
                format!("az://{}.blob.core.windows.net/{}/{}", account, container, path)
            }
            _ => path.to_string(),
        }
    }
}

impl Connector for CloudConnector {
    fn load(&self, path: &str) -> Result<DataFrame> {
        let uri = self.get_uri(path);

        // Determine format from extension
        let is_parquet = path.ends_with(".parquet") || path.ends_with(".pq");
        let is_csv = path.ends_with(".csv") || path.ends_with(".csv.gz");

        if is_parquet {
            // Use object_store for cloud parquet
            // This is a simplified implementation
            LazyFrame::scan_parquet(&uri, ScanArgsParquet::default())
                .map_err(|e| SigcError::Runtime(format!("Failed to scan parquet: {}", e)))?
                .collect()
                .map_err(|e| SigcError::Runtime(format!("Failed to collect: {}", e)))
        } else if is_csv {
            LazyCsvReader::new(&uri)
                .finish()
                .map_err(|e| SigcError::Runtime(format!("Failed to read CSV: {}", e)))?
                .collect()
                .map_err(|e| SigcError::Runtime(format!("Failed to collect: {}", e)))
        } else {
            Err(SigcError::Runtime(format!("Unknown file format: {}", path)))
        }
    }

    fn is_available(&self) -> bool {
        true // Would check actual connectivity
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Connector registry for managing multiple data sources
pub struct ConnectorRegistry {
    connectors: HashMap<String, Box<dyn Connector>>,
}

impl ConnectorRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        ConnectorRegistry {
            connectors: HashMap::new(),
        }
    }

    /// Register a connector
    pub fn register(&mut self, name: &str, connector: Box<dyn Connector>) {
        self.connectors.insert(name.to_string(), connector);
    }

    /// Get a connector by name
    pub fn get(&self, name: &str) -> Option<&dyn Connector> {
        self.connectors.get(name).map(|c| c.as_ref())
    }

    /// Load data using a connector
    pub fn load(&self, connector_name: &str, path: &str) -> Result<DataFrame> {
        let connector = self.connectors.get(connector_name)
            .ok_or_else(|| SigcError::Runtime(format!("Connector not found: {}", connector_name)))?;

        connector.load(path)
    }

    /// List all registered connectors
    pub fn list(&self) -> Vec<String> {
        self.connectors.keys().cloned().collect()
    }

    /// Check if a connector exists
    pub fn has(&self, name: &str) -> bool {
        self.connectors.contains_key(name)
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment-based connector configuration
pub struct ConnectorEnv;

impl ConnectorEnv {
    /// Create S3 connector from environment variables
    pub fn s3_from_env(bucket: &str) -> CloudConnector {
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let access_key = std::env::var("AWS_ACCESS_KEY_ID").ok();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").ok();

        if let (Some(ak), Some(sk)) = (access_key, secret_key) {
            CloudConnector::s3_with_credentials(bucket, &region, &ak, &sk)
        } else {
            CloudConnector::s3(bucket, &region)
        }
    }

    /// Create Postgres connector from environment
    pub fn postgres_from_env() -> Option<SqlConnector> {
        let host = std::env::var("PGHOST").ok()?;
        let port: u16 = std::env::var("PGPORT").ok()?.parse().ok()?;
        let database = std::env::var("PGDATABASE").ok()?;
        let user = std::env::var("PGUSER").ok()?;
        let password = std::env::var("PGPASSWORD").ok()?;

        Some(SqlConnector::postgres(&host, port, &database, &user, &password))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_connection_string() {
        let connector = SqlConnector::postgres("localhost", 5432, "testdb", "user", "pass");
        let conn_str = connector.connection_string();
        assert!(conn_str.contains("postgresql://"));
        assert!(conn_str.contains("localhost:5432"));
    }

    #[test]
    fn test_s3_uri() {
        let connector = CloudConnector::s3("my-bucket", "us-east-1");
        let uri = connector.get_uri("data/prices.parquet");
        assert_eq!(uri, "s3://my-bucket/data/prices.parquet");
    }

    #[test]
    fn test_gcs_uri() {
        let connector = CloudConnector::gcs("my-bucket", "my-project");
        let uri = connector.get_uri("data/prices.parquet");
        assert_eq!(uri, "gs://my-bucket/data/prices.parquet");
    }

    #[test]
    fn test_registry() {
        let mut registry = ConnectorRegistry::new();
        registry.register("s3_data", Box::new(CloudConnector::s3("bucket", "region")));

        assert!(registry.has("s3_data"));
        assert!(!registry.has("nonexistent"));
        assert_eq!(registry.list().len(), 1);
    }
}
