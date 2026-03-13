//! Data ingestion layer using Polars and object_store

use futures::TryStreamExt;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as ObjectPath;
use object_store::ObjectStore;
use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::path::Path;
use std::sync::Arc;

/// Data source configuration
#[derive(Debug, Clone)]
pub struct DataSource {
    pub name: String,
    pub path: String,
    pub format: DataFormat,
}

/// Supported data formats
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    Parquet,
    Csv,
}

impl DataFormat {
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".csv") || path.ends_with(".csv.gz") {
            DataFormat::Csv
        } else {
            DataFormat::Parquet
        }
    }
}

/// Date range for filtering data
#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: Option<String>,
    pub end: Option<String>,
}

/// Load data from various sources
pub struct DataLoader {
    runtime: tokio::runtime::Runtime,
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLoader {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        DataLoader { runtime }
    }

    /// Load data from a path (local, S3, or HTTP)
    pub fn load(&self, path: &str) -> Result<DataFrame> {
        let format = DataFormat::from_path(path);
        self.load_with_format(path, format)
    }

    /// Load data with explicit format
    pub fn load_with_format(&self, path: &str, format: DataFormat) -> Result<DataFrame> {
        if path.starts_with("s3://") {
            self.load_s3(path, format)
        } else if path.starts_with("http://") || path.starts_with("https://") {
            self.load_http(path, format)
        } else {
            self.load_local(path, format)
        }
    }

    /// Load data and filter by date range
    pub fn load_with_dates(
        &self,
        path: &str,
        date_col: &str,
        range: &DateRange,
    ) -> Result<DataFrame> {
        let df = self.load(path)?;
        self.filter_dates(df, date_col, range)
    }

    /// Load from local filesystem
    fn load_local(&self, path: &str, format: DataFormat) -> Result<DataFrame> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(SigcError::Runtime(format!(
                "File not found: {}",
                path.display()
            )));
        }

        match format {
            DataFormat::Parquet => {
                LazyFrame::scan_parquet(path, ScanArgsParquet::default())
                    .map_err(|e| SigcError::Runtime(format!("Failed to scan parquet: {}", e)))?
                    .collect()
                    .map_err(|e| SigcError::Runtime(format!("Failed to collect parquet: {}", e)))
            }
            DataFormat::Csv => {
                CsvReadOptions::default()
                    .with_infer_schema_length(Some(10000))
                    .with_has_header(true)
                    .try_into_reader_with_file_path(Some(path.to_path_buf()))
                    .map_err(|e| SigcError::Runtime(format!("Failed to read CSV: {}", e)))?
                    .finish()
                    .map_err(|e| SigcError::Runtime(format!("Failed to parse CSV: {}", e)))
            }
        }
    }

    /// Load from S3
    fn load_s3(&self, path: &str, format: DataFormat) -> Result<DataFrame> {
        self.runtime.block_on(async {
            // Parse S3 URL: s3://bucket/key
            let path = path.strip_prefix("s3://")
                .ok_or_else(|| SigcError::Runtime("Invalid S3 URL format".to_string()))?;
            let (bucket, key) = path.split_once('/').ok_or_else(|| {
                SigcError::Runtime("Invalid S3 path format".to_string())
            })?;

            tracing::info!("Loading from S3: bucket={}, key={}", bucket, key);

            // Build S3 client from environment
            let store = AmazonS3Builder::from_env()
                .with_bucket_name(bucket)
                .build()
                .map_err(|e| SigcError::Runtime(format!("Failed to create S3 client: {}", e)))?;

            self.load_from_object_store(Arc::new(store), key, format).await
        })
    }

    /// Load from HTTP/HTTPS URL
    fn load_http(&self, url: &str, format: DataFormat) -> Result<DataFrame> {
        self.runtime.block_on(async {
            tracing::info!("Loading from HTTP: {}", url);

            // Download to bytes
            let response = reqwest::get(url)
                .await
                .map_err(|e| SigcError::Runtime(format!("HTTP request failed: {}", e)))?;

            let bytes = response
                .bytes()
                .await
                .map_err(|e| SigcError::Runtime(format!("Failed to read response: {}", e)))?;

            self.parse_bytes(&bytes, format)
        })
    }

    /// Load from any object store
    async fn load_from_object_store(
        &self,
        store: Arc<dyn ObjectStore>,
        key: &str,
        format: DataFormat,
    ) -> Result<DataFrame> {
        let path = ObjectPath::from(key);

        // Get object and collect bytes
        let result = store
            .get(&path)
            .await
            .map_err(|e| SigcError::Runtime(format!("Failed to get object: {}", e)))?;

        let bytes = result
            .bytes()
            .await
            .map_err(|e| SigcError::Runtime(format!("Failed to read bytes: {}", e)))?;

        self.parse_bytes(&bytes, format)
    }

    /// Parse bytes into DataFrame
    fn parse_bytes(&self, bytes: &[u8], format: DataFormat) -> Result<DataFrame> {
        match format {
            DataFormat::Parquet => {
                let cursor = std::io::Cursor::new(bytes);
                ParquetReader::new(cursor)
                    .finish()
                    .map_err(|e| SigcError::Runtime(format!("Failed to parse parquet: {}", e)))
            }
            DataFormat::Csv => {
                let cursor = std::io::Cursor::new(bytes);
                CsvReader::new(cursor)
                    .finish()
                    .map_err(|e| SigcError::Runtime(format!("Failed to parse CSV: {}", e)))
            }
        }
    }

    /// Filter DataFrame by date range
    fn filter_dates(
        &self,
        df: DataFrame,
        date_col: &str,
        range: &DateRange,
    ) -> Result<DataFrame> {
        let mut lazy = df.lazy();

        if let Some(start) = &range.start {
            lazy = lazy.filter(col(date_col).gt_eq(lit(start.clone())));
        }

        if let Some(end) = &range.end {
            lazy = lazy.filter(col(date_col).lt_eq(lit(end.clone())));
        }

        lazy.collect()
            .map_err(|e| SigcError::Runtime(format!("Failed to filter dates: {}", e)))
    }

    /// List files in an S3 prefix
    pub fn list_s3(&self, prefix: &str) -> Result<Vec<String>> {
        self.runtime.block_on(async {
            let prefix = prefix.strip_prefix("s3://").unwrap_or(prefix);
            let (bucket, key) = prefix.split_once('/').ok_or_else(|| {
                SigcError::Runtime("Invalid S3 path format".to_string())
            })?;

            let store = AmazonS3Builder::from_env()
                .with_bucket_name(bucket)
                .build()
                .map_err(|e| SigcError::Runtime(format!("Failed to create S3 client: {}", e)))?;

            let path = ObjectPath::from(key);
            let list = store
                .list(Some(&path))
                .try_collect::<Vec<_>>()
                .await
                .map_err(|e| SigcError::Runtime(format!("Failed to list S3: {}", e)))?;

            Ok(list
                .into_iter()
                .map(|meta| format!("s3://{}/{}", bucket, meta.location))
                .collect())
        })
    }

    /// Create a sample price DataFrame for testing
    pub fn sample_prices(n_dates: usize, n_assets: usize) -> Result<DataFrame> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate dates
        let dates: Vec<i32> = (0..n_dates as i32).collect();

        // Generate asset columns with random walk prices
        let mut columns: Vec<Column> = vec![Column::new("date".into(), dates)];

        for i in 0..n_assets {
            let mut prices = Vec::with_capacity(n_dates);
            let mut price = 100.0f64;
            for _ in 0..n_dates {
                price *= 1.0 + rng.gen_range(-0.02..0.02);
                prices.push(price);
            }
            columns.push(Column::new(format!("asset_{}", i).into(), prices));
        }

        DataFrame::new(columns)
            .map_err(|e| SigcError::Runtime(format!("Failed to create DataFrame: {}", e)))
    }
}

/// Multi-source data manager for handling multiple data declarations
pub struct DataManager {
    loader: DataLoader,
    sources: Vec<DataSource>,
}

impl DataManager {
    pub fn new() -> Self {
        DataManager {
            loader: DataLoader::new(),
            sources: Vec::new(),
        }
    }

    /// Add a data source
    pub fn add_source(&mut self, name: String, path: String) {
        let format = DataFormat::from_path(&path);
        self.sources.push(DataSource { name, path, format });
    }

    /// Load all sources and return as a map
    pub fn load_all(&self) -> Result<std::collections::HashMap<String, DataFrame>> {
        let mut data = std::collections::HashMap::new();

        for source in &self.sources {
            tracing::info!("Loading data source '{}' from {}", source.name, source.path);
            let df = self.loader.load_with_format(&source.path, source.format)?;
            tracing::info!("  Loaded {} rows x {} columns", df.height(), df.width());
            data.insert(source.name.clone(), df);
        }

        Ok(data)
    }

    /// Load a specific source by name
    pub fn load_source(&self, name: &str) -> Result<DataFrame> {
        let source = self.sources.iter()
            .find(|s| s.name == name)
            .ok_or_else(|| SigcError::Runtime(format!("Data source '{}' not found", name)))?;

        self.loader.load_with_format(&source.path, source.format)
    }
}

impl Default for DataManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_prices() {
        let df = DataLoader::sample_prices(100, 5).unwrap();
        assert_eq!(df.height(), 100);
        assert_eq!(df.width(), 6); // date + 5 assets
    }

    #[test]
    fn test_format_detection() {
        assert!(matches!(DataFormat::from_path("data.parquet"), DataFormat::Parquet));
        assert!(matches!(DataFormat::from_path("data.csv"), DataFormat::Csv));
        assert!(matches!(DataFormat::from_path("s3://bucket/key.parquet"), DataFormat::Parquet));
    }

    #[test]
    fn test_data_manager() {
        let mut manager = DataManager::new();
        manager.add_source("prices".to_string(), "data/prices.parquet".to_string());
        assert_eq!(manager.sources.len(), 1);
    }
}
