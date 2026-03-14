//! Unified configuration system for sigc runtime
//!
//! Provides a single configuration structure for all runtime components.

use serde::{Deserialize, Serialize};
use sig_types::{Result, SigcError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Main runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Execution configuration
    #[serde(default)]
    pub execution: ExecutionConfig,

    /// Data configuration
    #[serde(default)]
    pub data: DataConfig,

    /// Alerting configuration
    #[serde(default)]
    pub alerts: AlertConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            database: DatabaseConfig::default(),
            execution: ExecutionConfig::default(),
            data: DataConfig::default(),
            alerts: AlertConfig::default(),
            logging: LoggingConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl RuntimeConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| SigcError::Runtime(format!("Failed to read config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| SigcError::Runtime(format!("Failed to parse config: {}", e)))
    }

    /// Load from default locations
    pub fn load() -> Result<Self> {
        // Try in order: ./sigc.toml, ~/.config/sigc/config.toml
        let paths = vec![
            PathBuf::from("sigc.toml"),
            dirs::config_dir()
                .map(|p| p.join("sigc/config.toml"))
                .unwrap_or_default(),
        ];

        for path in paths {
            if path.exists() {
                return Self::from_file(path);
            }
        }

        Ok(Self::default())
    }

    /// Load from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Database
        if let Ok(host) = std::env::var("SIGC_DB_HOST") {
            config.database.host = host;
        }
        if let Ok(port) = std::env::var("SIGC_DB_PORT") {
            config.database.port = port.parse().unwrap_or(5432);
        }
        if let Ok(name) = std::env::var("SIGC_DB_NAME") {
            config.database.database = name;
        }
        if let Ok(user) = std::env::var("SIGC_DB_USER") {
            config.database.user = user;
        }
        if let Ok(pass) = std::env::var("SIGC_DB_PASSWORD") {
            config.database.password = pass;
        }

        // Execution
        if let Ok(threads) = std::env::var("SIGC_THREADS") {
            config.execution.num_threads = threads.parse().unwrap_or(0);
        }

        // Alerts
        if let Ok(url) = std::env::var("SLACK_WEBHOOK_URL") {
            config.alerts.slack_webhook = Some(url);
        }

        // Logging
        if let Ok(level) = std::env::var("SIGC_LOG_LEVEL") {
            config.logging.level = level;
        }
        if let Ok(path) = std::env::var("SIGC_AUDIT_LOG") {
            config.logging.audit_path = Some(path);
        }

        // Cache
        if let Ok(dir) = std::env::var("SIGC_CACHE_DIR") {
            config.cache.directory = PathBuf::from(dir);
        }

        config
    }

    /// Merge with environment variables (env takes precedence)
    pub fn with_env(mut self) -> Self {
        let env_config = Self::from_env();

        // Only override if env var was set (non-default values)
        if env_config.database.host != "localhost" {
            self.database.host = env_config.database.host;
        }
        if env_config.database.port != 5432 {
            self.database.port = env_config.database.port;
        }
        if !env_config.database.database.is_empty() && env_config.database.database != "sigc" {
            self.database.database = env_config.database.database;
        }
        if !env_config.database.user.is_empty() && env_config.database.user != "postgres" {
            self.database.user = env_config.database.user;
        }
        if !env_config.database.password.is_empty() {
            self.database.password = env_config.database.password;
        }

        if env_config.execution.num_threads != 0 {
            self.execution.num_threads = env_config.execution.num_threads;
        }

        if env_config.alerts.slack_webhook.is_some() {
            self.alerts.slack_webhook = env_config.alerts.slack_webhook;
        }

        if env_config.logging.level != "info" {
            self.logging.level = env_config.logging.level;
        }
        if env_config.logging.audit_path.is_some() {
            self.logging.audit_path = env_config.logging.audit_path;
        }

        self
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| SigcError::Runtime(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path.as_ref(), content)
            .map_err(|e| SigcError::Runtime(format!("Failed to write config: {}", e)))
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub query_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "sigc".to_string(),
            user: "postgres".to_string(),
            password: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            query_timeout: Duration::from_secs(300),
        }
    }
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Number of threads (0 = auto)
    pub num_threads: usize,
    /// Chunk size for parallel processing
    pub chunk_size: usize,
    /// Use SIMD optimizations
    pub use_simd: bool,
    /// Minimum data size for SIMD
    pub simd_threshold: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            num_threads: 0,
            chunk_size: 100,
            use_simd: true,
            simd_threshold: 64,
        }
    }
}

/// Data configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// Default data directory
    pub data_dir: PathBuf,
    /// Date column name
    pub date_column: String,
    /// Price columns
    pub price_columns: Vec<String>,
    /// Volume column
    pub volume_column: String,
    /// Adjust for corporate actions
    pub adjust_corporate_actions: bool,
    /// Validate data quality
    pub validate_quality: bool,
    /// Maximum missing data percentage
    pub max_missing_pct: f64,
}

impl Default for DataConfig {
    fn default() -> Self {
        DataConfig {
            data_dir: PathBuf::from("data"),
            date_column: "date".to_string(),
            price_columns: vec![
                "open".to_string(),
                "high".to_string(),
                "low".to_string(),
                "close".to_string(),
            ],
            volume_column: "volume".to_string(),
            adjust_corporate_actions: true,
            validate_quality: true,
            max_missing_pct: 5.0,
        }
    }
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertConfig {
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
    /// Slack channel
    pub slack_channel: Option<String>,
    /// Email server
    pub email_server: Option<String>,
    /// Email recipients
    pub email_recipients: Vec<String>,
    /// Alert on backtest complete
    pub on_backtest_complete: bool,
    /// Alert on errors
    pub on_error: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log file path
    pub file: Option<String>,
    /// Audit log path
    pub audit_path: Option<String>,
    /// Enable JSON logging
    pub json: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            file: None,
            audit_path: None,
            json: false,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache directory
    pub directory: PathBuf,
    /// Enable caching
    pub enabled: bool,
    /// Maximum cache size in MB
    pub max_size_mb: u64,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            directory: dirs::cache_dir()
                .map(|p| p.join("sigc"))
                .unwrap_or_else(|| PathBuf::from(".cache/sigc")),
            enabled: true,
            max_size_mb: 1024,
            ttl_seconds: 86400,
        }
    }
}

/// Backtest-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Initial capital
    pub initial_capital: f64,
    /// Trading costs in basis points
    pub cost_bps: f64,
    /// Slippage in basis points
    pub slippage_bps: f64,
    /// Rebalance frequency
    pub rebalance_frequency: String,
    /// Use leverage
    pub allow_leverage: bool,
    /// Maximum leverage
    pub max_leverage: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        BacktestConfig {
            initial_capital: 1_000_000.0,
            cost_bps: 5.0,
            slippage_bps: 5.0,
            rebalance_frequency: "daily".to_string(),
            allow_leverage: false,
            max_leverage: 1.0,
        }
    }
}

/// Strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyParams {
    /// Named parameters
    pub params: HashMap<String, f64>,
}

impl StrategyParams {
    pub fn new() -> Self {
        StrategyParams {
            params: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: &str, value: f64) -> &mut Self {
        self.params.insert(name.to_string(), value);
        self
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        self.params.get(name).copied()
    }

    pub fn get_or(&self, name: &str, default: f64) -> f64 {
        self.params.get(name).copied().unwrap_or(default)
    }
}

// Helper module for duration serialization
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}s", duration.as_secs());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Parse "30s", "5m", etc.
        let s = s.trim();
        if let Some(secs) = s.strip_suffix('s') {
            let n: u64 = secs.parse().map_err(serde::de::Error::custom)?;
            Ok(Duration::from_secs(n))
        } else if let Some(mins) = s.strip_suffix('m') {
            let n: u64 = mins.parse().map_err(serde::de::Error::custom)?;
            Ok(Duration::from_secs(n * 60))
        } else if let Some(hours) = s.strip_suffix('h') {
            let n: u64 = hours.parse().map_err(serde::de::Error::custom)?;
            Ok(Duration::from_secs(n * 3600))
        } else {
            // Assume seconds
            let n: u64 = s.parse().map_err(serde::de::Error::custom)?;
            Ok(Duration::from_secs(n))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.execution.chunk_size, 100);
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_strategy_params() {
        let mut params = StrategyParams::new();
        params.set("window", 20.0).set("threshold", 0.5);

        assert_eq!(params.get("window"), Some(20.0));
        assert_eq!(params.get_or("missing", 10.0), 10.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = RuntimeConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("[database]"));
        assert!(toml.contains("[execution]"));
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("SIGC_LOG_LEVEL", "debug");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.logging.level, "debug");
        std::env::remove_var("SIGC_LOG_LEVEL");
    }
}
