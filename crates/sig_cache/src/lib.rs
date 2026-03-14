//! Caching layer for sigc using sled and blake3
//!
//! Provides content-addressable storage for IR and computed artifacts.

use rkyv::Deserialize;
use sig_types::{Ir, Result, SigcError};
use std::path::Path;
use tracing::{debug, info};

/// Cache manager backed by sled
///
/// Note: sled::Db is internally Arc-based, so Clone is cheap (reference counting)
#[derive(Clone)]
pub struct Cache {
    db: sled::Db,
}

impl Cache {
    /// Open or create a cache at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path).map_err(|e| SigcError::Cache(e.to_string()))?;
        info!("Cache initialized");
        Ok(Cache { db })
    }

    /// Open an in-memory cache for testing
    pub fn in_memory() -> Result<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open().map_err(|e| SigcError::Cache(e.to_string()))?;
        Ok(Cache { db })
    }

    /// Compute blake3 hash of data
    pub fn hash(data: &[u8]) -> String {
        blake3::hash(data).to_hex().to_string()
    }

    /// Store data with its content hash as key
    pub fn put(&self, data: &[u8]) -> Result<String> {
        let key = Self::hash(data);
        self.db
            .insert(key.as_bytes(), data)
            .map_err(|e| SigcError::Cache(e.to_string()))?;
        debug!(key = %key, "Cached artifact");
        Ok(key)
    }

    /// Store data with a specific key
    pub fn put_keyed(&self, key: &str, data: &[u8]) -> Result<()> {
        self.db
            .insert(key.as_bytes(), data)
            .map_err(|e| SigcError::Cache(e.to_string()))?;
        debug!(key = %key, "Cached artifact with key");
        Ok(())
    }

    /// Retrieve data by key
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let result = self
            .db
            .get(key.as_bytes())
            .map_err(|e| SigcError::Cache(e.to_string()))?
            .map(|v| v.to_vec());
        Ok(result)
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> Result<bool> {
        self.db
            .contains_key(key.as_bytes())
            .map_err(|e| SigcError::Cache(e.to_string()))
    }

    /// Remove a key from the cache
    pub fn remove(&self, key: &str) -> Result<()> {
        self.db
            .remove(key.as_bytes())
            .map_err(|e| SigcError::Cache(e.to_string()))?;
        Ok(())
    }

    /// Flush pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush().map_err(|e| SigcError::Cache(e.to_string()))?;
        Ok(())
    }

    /// Serialize and cache an IR
    pub fn put_ir(&self, source_hash: &str, ir: &Ir) -> Result<()> {
        let bytes = rkyv::to_bytes::<Ir, 1024>(ir)
            .map_err(|e| SigcError::Cache(format!("Serialization failed: {}", e)))?;
        let key = format!("ir:{}", source_hash);
        self.put_keyed(&key, &bytes)?;
        debug!(key = %key, size = bytes.len(), "Cached IR");
        Ok(())
    }

    /// Retrieve and deserialize a cached IR
    pub fn get_ir(&self, source_hash: &str) -> Result<Option<Ir>> {
        let key = format!("ir:{}", source_hash);
        match self.get(&key)? {
            Some(bytes) => {
                let archived = unsafe { rkyv::archived_root::<Ir>(&bytes) };
                let ir: Ir = archived.deserialize(&mut rkyv::Infallible)
                    .map_err(|_| SigcError::Cache("Deserialization failed".into()))?;
                debug!(key = %key, "Cache hit for IR");
                Ok(Some(ir))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sig_types::{IrMetadata, IrNode, Operator, TypeAnnotation, DType, Shape};

    #[test]
    fn test_cache_roundtrip() {
        let cache = Cache::in_memory().unwrap();
        let data = b"test data";
        let key = cache.put(data).unwrap();
        let retrieved = cache.get(&key).unwrap().unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_cache_keyed() {
        let cache = Cache::in_memory().unwrap();
        cache.put_keyed("mykey", b"mydata").unwrap();
        let retrieved = cache.get("mykey").unwrap().unwrap();
        assert_eq!(retrieved, b"mydata");
    }

    #[test]
    fn test_ir_serialization() {
        let cache = Cache::in_memory().unwrap();

        // Create a simple IR
        let ir = Ir {
            nodes: vec![
                IrNode {
                    id: 0,
                    operator: Operator::Add,
                    inputs: vec![1, 2],
                    type_info: TypeAnnotation {
                        dtype: DType::Float64,
                        shape: Shape::scalar(),
                    },
                },
            ],
            outputs: vec![0],
            metadata: IrMetadata {
                source_hash: "test123".to_string(),
                compiled_at: 1234567890,
                compiler_version: "0.1.0".to_string(),
                parameters: vec![],
                data_sources: vec![],
            },
        };

        // Cache it
        cache.put_ir("test123", &ir).unwrap();

        // Retrieve it
        let retrieved = cache.get_ir("test123").unwrap().unwrap();

        assert_eq!(retrieved.nodes.len(), 1);
        assert_eq!(retrieved.outputs, vec![0]);
        assert_eq!(retrieved.metadata.source_hash, "test123");
    }
}
