//! Audit logging for compliance and governance
//!
//! Provides structured logging of all operations for regulatory compliance,
//! model governance, and debugging.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEvent {
    /// Signal compilation started
    CompileStart {
        source_hash: String,
        user: Option<String>,
    },
    /// Signal compilation completed
    CompileComplete {
        source_hash: String,
        node_count: usize,
        duration_ms: u64,
        cache_hit: bool,
    },
    /// Signal compilation failed
    CompileFailed {
        source_hash: String,
        error: String,
    },
    /// Backtest started
    BacktestStart {
        signal_hash: String,
        start_date: String,
        end_date: String,
        parameters: std::collections::HashMap<String, f64>,
    },
    /// Backtest completed
    BacktestComplete {
        signal_hash: String,
        total_return: f64,
        sharpe_ratio: f64,
        max_drawdown: f64,
        duration_ms: u64,
    },
    /// Backtest failed
    BacktestFailed {
        signal_hash: String,
        error: String,
    },
    /// Data loaded
    DataLoaded {
        source: String,
        rows: usize,
        columns: usize,
    },
    /// Cache operation
    CacheOperation {
        operation: String,
        key: String,
        hit: bool,
    },
    /// Parameter optimization
    Optimization {
        signal_hash: String,
        parameter_count: usize,
        best_sharpe: f64,
        combinations_tested: usize,
    },
}

/// Audit log entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub session_id: String,
    pub event: AuditEvent,
}

/// Audit logger for tracking operations
pub struct AuditLogger {
    session_id: String,
    writer: Option<Arc<Mutex<BufWriter<File>>>>,
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        let session_id = format!("{:x}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0));

        AuditLogger {
            session_id,
            writer: None,
            enabled: false,
        }
    }

    /// Create a logger that writes to a file
    pub fn with_file(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let session_id = format!("{:x}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0));

        Ok(AuditLogger {
            session_id,
            writer: Some(Arc::new(Mutex::new(BufWriter::new(file)))),
            enabled: true,
        })
    }

    /// Enable or disable audit logging
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Log an audit event
    pub fn log(&self, event: AuditEvent) {
        if !self.enabled {
            return;
        }

        let entry = AuditEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            session_id: self.session_id.clone(),
            event: event.clone(),
        };

        // Log to tracing
        match &event {
            AuditEvent::CompileStart { source_hash, .. } => {
                tracing::info!(target: "audit", event = "compile_start", source_hash = %source_hash);
            }
            AuditEvent::CompileComplete { source_hash, node_count, duration_ms, cache_hit } => {
                tracing::info!(target: "audit", event = "compile_complete",
                    source_hash = %source_hash, nodes = node_count,
                    duration_ms = duration_ms, cache_hit = cache_hit);
            }
            AuditEvent::CompileFailed { source_hash, error } => {
                tracing::error!(target: "audit", event = "compile_failed",
                    source_hash = %source_hash, error = %error);
            }
            AuditEvent::BacktestStart { signal_hash, start_date, end_date, .. } => {
                tracing::info!(target: "audit", event = "backtest_start",
                    signal_hash = %signal_hash, start = %start_date, end = %end_date);
            }
            AuditEvent::BacktestComplete { signal_hash, total_return, sharpe_ratio, .. } => {
                tracing::info!(target: "audit", event = "backtest_complete",
                    signal_hash = %signal_hash,
                    return_pct = total_return * 100.0,
                    sharpe = sharpe_ratio);
            }
            AuditEvent::BacktestFailed { signal_hash, error } => {
                tracing::error!(target: "audit", event = "backtest_failed",
                    signal_hash = %signal_hash, error = %error);
            }
            AuditEvent::DataLoaded { source, rows, columns } => {
                tracing::debug!(target: "audit", event = "data_loaded",
                    source = %source, rows = rows, columns = columns);
            }
            AuditEvent::CacheOperation { operation, key, hit } => {
                tracing::debug!(target: "audit", event = "cache_op",
                    op = %operation, key = %key, hit = hit);
            }
            AuditEvent::Optimization { signal_hash, combinations_tested, best_sharpe, .. } => {
                tracing::info!(target: "audit", event = "optimization",
                    signal_hash = %signal_hash,
                    combinations = combinations_tested,
                    best_sharpe = best_sharpe);
            }
        }

        // Write to file if configured
        if let Some(ref writer) = self.writer {
            if let Ok(mut w) = writer.lock() {
                if let Ok(json) = serde_json::to_string(&entry) {
                    let _ = writeln!(w, "{}", json);
                    let _ = w.flush();
                }
            }
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global audit logger instance
static AUDIT_LOGGER: std::sync::OnceLock<Mutex<AuditLogger>> = std::sync::OnceLock::new();

/// Initialize the global audit logger
pub fn init_audit_logger(path: Option<PathBuf>) -> std::io::Result<()> {
    let logger = if let Some(p) = path {
        AuditLogger::with_file(p)?
    } else {
        let mut l = AuditLogger::new();
        l.set_enabled(true);
        l
    };

    let _ = AUDIT_LOGGER.set(Mutex::new(logger));
    Ok(())
}

/// Log an audit event to the global logger
pub fn audit_log(event: AuditEvent) {
    if let Some(logger) = AUDIT_LOGGER.get() {
        if let Ok(l) = logger.lock() {
            l.log(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert!(!logger.session_id.is_empty());
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::BacktestComplete {
            signal_hash: "abc123".to_string(),
            total_return: 0.15,
            sharpe_ratio: 1.5,
            max_drawdown: 0.10,
            duration_ms: 1000,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("BacktestComplete"));
        assert!(json.contains("1.5"));
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: 1234567890,
            session_id: "test123".to_string(),
            event: AuditEvent::CompileStart {
                source_hash: "hash123".to_string(),
                user: Some("testuser".to_string()),
            },
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("1234567890"));
        assert!(json.contains("test123"));
    }

    #[test]
    fn test_disabled_logger() {
        let logger = AuditLogger::new();
        // Should not panic even though logging is disabled
        logger.log(AuditEvent::DataLoaded {
            source: "test.csv".to_string(),
            rows: 100,
            columns: 5,
        });
    }
}
