//! Memory-mapped data loading for large datasets
//!
//! Provides efficient access to large data files without loading everything into memory.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::fs::File;
use std::path::Path;

/// Memory-mapped data loader for efficient large file access
pub struct MmapLoader {
    /// Chunk size for streaming reads
    chunk_size: usize,
    /// Whether to use lazy evaluation
    lazy: bool,
}

impl MmapLoader {
    pub fn new() -> Self {
        MmapLoader {
            chunk_size: 100_000,
            lazy: true,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_lazy(mut self, lazy: bool) -> Self {
        self.lazy = lazy;
        self
    }

    /// Load a Parquet file using memory mapping
    pub fn load_parquet<P: AsRef<Path>>(&self, path: P) -> Result<DataFrame> {
        let file = File::open(path.as_ref())
            .map_err(|e| SigcError::Runtime(format!("Failed to open file: {}", e)))?;

        let df = ParquetReader::new(file)
            .use_statistics(true)
            .finish()
            .map_err(|e| SigcError::Runtime(format!("Failed to read Parquet: {}", e)))?;

        Ok(df)
    }

    /// Load a Parquet file lazily for streaming processing
    pub fn load_parquet_lazy<P: AsRef<Path>>(&self, path: P) -> Result<LazyFrame> {
        let lf = LazyFrame::scan_parquet(path.as_ref(), Default::default())
            .map_err(|e| SigcError::Runtime(format!("Failed to scan Parquet: {}", e)))?;

        Ok(lf)
    }

    /// Load CSV with streaming for large files
    pub fn load_csv_streaming<P: AsRef<Path>>(&self, path: P) -> Result<DataFrame> {
        let file = File::open(path.as_ref())
            .map_err(|e| SigcError::Runtime(format!("Failed to open file: {}", e)))?;

        let df = CsvReader::new(file)
            .finish()
            .map_err(|e| SigcError::Runtime(format!("Failed to read CSV: {}", e)))?;

        Ok(df)
    }

    /// Load CSV lazily
    pub fn load_csv_lazy<P: AsRef<Path>>(&self, path: P) -> Result<LazyFrame> {
        let lf = LazyCsvReader::new(path.as_ref())
            .with_has_header(true)
            .finish()
            .map_err(|e| SigcError::Runtime(format!("Failed to scan CSV: {}", e)))?;

        Ok(lf)
    }

    /// Load specific columns only (reduces memory usage)
    pub fn load_parquet_columns<P: AsRef<Path>>(
        &self,
        path: P,
        columns: &[&str],
    ) -> Result<DataFrame> {
        let file = File::open(path.as_ref())
            .map_err(|e| SigcError::Runtime(format!("Failed to open file: {}", e)))?;

        let df = ParquetReader::new(file)
            .with_columns(Some(columns.iter().map(|s| s.to_string()).collect()))
            .finish()
            .map_err(|e| SigcError::Runtime(format!("Failed to read Parquet: {}", e)))?;

        Ok(df)
    }

    /// Load with row filtering using predicate pushdown
    pub fn load_parquet_filtered<P: AsRef<Path>>(
        &self,
        path: P,
        filter: Expr,
    ) -> Result<DataFrame> {
        let lf = LazyFrame::scan_parquet(path.as_ref(), Default::default())
            .map_err(|e| SigcError::Runtime(format!("Failed to scan Parquet: {}", e)))?;

        let df = lf
            .filter(filter)
            .collect()
            .map_err(|e| SigcError::Runtime(format!("Failed to collect: {}", e)))?;

        Ok(df)
    }

    /// Chunk a large DataFrame for parallel processing
    pub fn chunk_dataframe(&self, df: &DataFrame) -> Vec<DataFrame> {
        let n_rows = df.height();
        let n_chunks = (n_rows + self.chunk_size - 1) / self.chunk_size;

        (0..n_chunks)
            .map(|i| {
                let start = i * self.chunk_size;
                let end = ((i + 1) * self.chunk_size).min(n_rows);
                df.slice(start as i64, end - start)
            })
            .collect()
    }
}

impl Default for MmapLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming iterator for large datasets
pub struct DataStream {
    lazy_frame: LazyFrame,
    processed: bool,
}

impl DataStream {
    pub fn new(lf: LazyFrame) -> Self {
        DataStream {
            lazy_frame: lf,
            processed: false,
        }
    }

    /// Apply a filter to the stream
    pub fn filter(mut self, predicate: Expr) -> Self {
        self.lazy_frame = self.lazy_frame.filter(predicate);
        self
    }

    /// Select specific columns
    pub fn select(mut self, exprs: Vec<Expr>) -> Self {
        self.lazy_frame = self.lazy_frame.select(exprs);
        self
    }

    /// Sort by column
    pub fn sort(mut self, by: &str, descending: bool) -> Self {
        self.lazy_frame = self.lazy_frame.sort(
            [by],
            SortMultipleOptions::default().with_order_descending(descending),
        );
        self
    }

    /// Limit rows
    pub fn limit(mut self, n: u32) -> Self {
        self.lazy_frame = self.lazy_frame.limit(n);
        self
    }

    /// Collect the stream into a DataFrame
    pub fn collect(mut self) -> Result<DataFrame> {
        if self.processed {
            return Err(SigcError::Runtime("Stream already processed".into()));
        }
        self.processed = true;

        self.lazy_frame
            .collect()
            .map_err(|e| SigcError::Runtime(format!("Failed to collect stream: {}", e)))
    }
}

/// Data cache using memory-mapped files
pub struct MmapCache {
    cache_dir: std::path::PathBuf,
}

impl MmapCache {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let path = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)
            .map_err(|e| SigcError::Runtime(format!("Failed to create cache dir: {}", e)))?;

        Ok(MmapCache { cache_dir: path })
    }

    /// Cache a DataFrame to Parquet
    pub fn cache(&self, key: &str, df: &DataFrame) -> Result<()> {
        let path = self.cache_dir.join(format!("{}.parquet", key));
        let file = File::create(&path)
            .map_err(|e| SigcError::Runtime(format!("Failed to create cache file: {}", e)))?;

        ParquetWriter::new(file)
            .finish(&mut df.clone())
            .map_err(|e| SigcError::Runtime(format!("Failed to write cache: {}", e)))?;

        Ok(())
    }

    /// Load from cache
    pub fn load(&self, key: &str) -> Result<Option<DataFrame>> {
        let path = self.cache_dir.join(format!("{}.parquet", key));

        if !path.exists() {
            return Ok(None);
        }

        let loader = MmapLoader::new();
        let df = loader.load_parquet(&path)?;
        Ok(Some(df))
    }

    /// Check if key is cached
    pub fn contains(&self, key: &str) -> bool {
        self.cache_dir.join(format!("{}.parquet", key)).exists()
    }

    /// Remove from cache
    pub fn remove(&self, key: &str) -> Result<()> {
        let path = self.cache_dir.join(format!("{}.parquet", key));
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| SigcError::Runtime(format!("Failed to remove cache: {}", e)))?;
        }
        Ok(())
    }

    /// Clear all cache
    pub fn clear(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.cache_dir)
            .map_err(|e| SigcError::Runtime(format!("Failed to read cache dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| SigcError::Runtime(format!("Failed to read entry: {}", e)))?;
            if entry.path().extension().map(|e| e == "parquet").unwrap_or(false) {
                std::fs::remove_file(entry.path())
                    .map_err(|e| SigcError::Runtime(format!("Failed to remove: {}", e)))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_mmap_loader_default() {
        let loader = MmapLoader::new();
        assert_eq!(loader.chunk_size, 100_000);
        assert!(loader.lazy);
    }

    #[test]
    fn test_chunk_dataframe() {
        let loader = MmapLoader::new().with_chunk_size(10);

        let df = DataFrame::new(vec![
            Column::new("a".into(), (0..25).collect::<Vec<i32>>()),
        ])
        .unwrap();

        let chunks = loader.chunk_dataframe(&df);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].height(), 10);
        assert_eq!(chunks[1].height(), 10);
        assert_eq!(chunks[2].height(), 5);
    }

    #[test]
    fn test_csv_loading() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.csv");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "a,b,c").unwrap();
            writeln!(file, "1,2,3").unwrap();
            writeln!(file, "4,5,6").unwrap();
        }

        let loader = MmapLoader::new();
        let df = loader.load_csv_streaming(&file_path).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 3);
    }

    #[test]
    fn test_mmap_cache() {
        let dir = tempdir().unwrap();
        let cache = MmapCache::new(dir.path()).unwrap();

        let df = DataFrame::new(vec![
            Column::new("x".into(), vec![1.0, 2.0, 3.0]),
        ])
        .unwrap();

        cache.cache("test", &df).unwrap();
        assert!(cache.contains("test"));

        let loaded = cache.load("test").unwrap().unwrap();
        assert_eq!(loaded.height(), 3);

        cache.remove("test").unwrap();
        assert!(!cache.contains("test"));
    }
}
