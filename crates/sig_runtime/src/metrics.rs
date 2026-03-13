//! Observability and metrics for monitoring sigc performance
//!
//! Provides Prometheus-compatible metrics for monitoring compilation,
//! execution, and resource usage.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Global metrics registry
pub struct MetricsRegistry {
    // Counters
    pub compilations_total: AtomicU64,
    pub compilations_failed: AtomicU64,
    pub backtests_total: AtomicU64,
    pub backtests_failed: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub data_loads: AtomicU64,
    pub optimizations_total: AtomicU64,

    // Gauges (using AtomicU64 for simplicity, store as milliunits)
    pub active_backtests: AtomicU64,
    pub cache_size_bytes: AtomicU64,

    // Histograms (simplified as recent values)
    compile_durations: Mutex<Vec<Duration>>,
    backtest_durations: Mutex<Vec<Duration>>,
    data_load_durations: Mutex<Vec<Duration>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        MetricsRegistry {
            compilations_total: AtomicU64::new(0),
            compilations_failed: AtomicU64::new(0),
            backtests_total: AtomicU64::new(0),
            backtests_failed: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            data_loads: AtomicU64::new(0),
            optimizations_total: AtomicU64::new(0),
            active_backtests: AtomicU64::new(0),
            cache_size_bytes: AtomicU64::new(0),
            compile_durations: Mutex::new(Vec::new()),
            backtest_durations: Mutex::new(Vec::new()),
            data_load_durations: Mutex::new(Vec::new()),
        }
    }

    /// Record a successful compilation
    pub fn record_compile(&self, duration: Duration) {
        self.compilations_total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut durations) = self.compile_durations.lock() {
            durations.push(duration);
            // Keep only last 1000 samples
            if durations.len() > 1000 {
                durations.remove(0);
            }
        }
    }

    /// Record a failed compilation
    pub fn record_compile_failed(&self) {
        self.compilations_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful backtest
    pub fn record_backtest(&self, duration: Duration) {
        self.backtests_total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut durations) = self.backtest_durations.lock() {
            durations.push(duration);
            if durations.len() > 1000 {
                durations.remove(0);
            }
        }
    }

    /// Record a failed backtest
    pub fn record_backtest_failed(&self) {
        self.backtests_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a data load
    pub fn record_data_load(&self, duration: Duration) {
        self.data_loads.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut durations) = self.data_load_durations.lock() {
            durations.push(duration);
            if durations.len() > 1000 {
                durations.remove(0);
            }
        }
    }

    /// Record an optimization run
    pub fn record_optimization(&self) {
        self.optimizations_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active backtests counter
    pub fn start_backtest(&self) {
        self.active_backtests.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active backtests counter
    pub fn end_backtest(&self) {
        self.active_backtests.fetch_sub(1, Ordering::Relaxed);
    }

    /// Set cache size
    pub fn set_cache_size(&self, bytes: u64) {
        self.cache_size_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Get average compile duration
    pub fn avg_compile_duration(&self) -> Option<Duration> {
        if let Ok(durations) = self.compile_durations.lock() {
            if durations.is_empty() {
                None
            } else {
                let total: Duration = durations.iter().sum();
                Some(total / durations.len() as u32)
            }
        } else {
            None
        }
    }

    /// Get average backtest duration
    pub fn avg_backtest_duration(&self) -> Option<Duration> {
        if let Ok(durations) = self.backtest_durations.lock() {
            if durations.is_empty() {
                None
            } else {
                let total: Duration = durations.iter().sum();
                Some(total / durations.len() as u32)
            }
        } else {
            None
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Counters
        output.push_str(&format!(
            "# HELP sigc_compilations_total Total number of compilations\n\
             # TYPE sigc_compilations_total counter\n\
             sigc_compilations_total {}\n\n",
            self.compilations_total.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_compilations_failed Total number of failed compilations\n\
             # TYPE sigc_compilations_failed counter\n\
             sigc_compilations_failed {}\n\n",
            self.compilations_failed.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_backtests_total Total number of backtests\n\
             # TYPE sigc_backtests_total counter\n\
             sigc_backtests_total {}\n\n",
            self.backtests_total.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_backtests_failed Total number of failed backtests\n\
             # TYPE sigc_backtests_failed counter\n\
             sigc_backtests_failed {}\n\n",
            self.backtests_failed.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_cache_hits Total cache hits\n\
             # TYPE sigc_cache_hits counter\n\
             sigc_cache_hits {}\n\n",
            self.cache_hits.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_cache_misses Total cache misses\n\
             # TYPE sigc_cache_misses counter\n\
             sigc_cache_misses {}\n\n",
            self.cache_misses.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_data_loads Total data load operations\n\
             # TYPE sigc_data_loads counter\n\
             sigc_data_loads {}\n\n",
            self.data_loads.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_optimizations_total Total optimization runs\n\
             # TYPE sigc_optimizations_total counter\n\
             sigc_optimizations_total {}\n\n",
            self.optimizations_total.load(Ordering::Relaxed)
        ));

        // Gauges
        output.push_str(&format!(
            "# HELP sigc_active_backtests Currently running backtests\n\
             # TYPE sigc_active_backtests gauge\n\
             sigc_active_backtests {}\n\n",
            self.active_backtests.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP sigc_cache_size_bytes Cache size in bytes\n\
             # TYPE sigc_cache_size_bytes gauge\n\
             sigc_cache_size_bytes {}\n\n",
            self.cache_size_bytes.load(Ordering::Relaxed)
        ));

        // Summary statistics
        if let Some(avg) = self.avg_compile_duration() {
            output.push_str(&format!(
                "# HELP sigc_compile_duration_avg_ms Average compile duration\n\
                 # TYPE sigc_compile_duration_avg_ms gauge\n\
                 sigc_compile_duration_avg_ms {}\n\n",
                avg.as_millis()
            ));
        }

        if let Some(avg) = self.avg_backtest_duration() {
            output.push_str(&format!(
                "# HELP sigc_backtest_duration_avg_ms Average backtest duration\n\
                 # TYPE sigc_backtest_duration_avg_ms gauge\n\
                 sigc_backtest_duration_avg_ms {}\n\n",
                avg.as_millis()
            ));
        }

        output
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "counters": {
                "compilations_total": self.compilations_total.load(Ordering::Relaxed),
                "compilations_failed": self.compilations_failed.load(Ordering::Relaxed),
                "backtests_total": self.backtests_total.load(Ordering::Relaxed),
                "backtests_failed": self.backtests_failed.load(Ordering::Relaxed),
                "cache_hits": self.cache_hits.load(Ordering::Relaxed),
                "cache_misses": self.cache_misses.load(Ordering::Relaxed),
                "data_loads": self.data_loads.load(Ordering::Relaxed),
                "optimizations_total": self.optimizations_total.load(Ordering::Relaxed),
            },
            "gauges": {
                "active_backtests": self.active_backtests.load(Ordering::Relaxed),
                "cache_size_bytes": self.cache_size_bytes.load(Ordering::Relaxed),
            },
            "summaries": {
                "avg_compile_duration_ms": self.avg_compile_duration().map(|d| d.as_millis()),
                "avg_backtest_duration_ms": self.avg_backtest_duration().map(|d| d.as_millis()),
            }
        })
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance
static METRICS: std::sync::OnceLock<MetricsRegistry> = std::sync::OnceLock::new();

/// Get the global metrics registry
pub fn metrics() -> &'static MetricsRegistry {
    METRICS.get_or_init(MetricsRegistry::new)
}

/// Timer guard for automatic duration recording
pub struct Timer {
    start: Instant,
    record_fn: Box<dyn FnOnce(Duration) + Send>,
}

impl Timer {
    /// Create a timer that records compile duration
    pub fn compile() -> Self {
        Timer {
            start: Instant::now(),
            record_fn: Box::new(|d| metrics().record_compile(d)),
        }
    }

    /// Create a timer that records backtest duration
    pub fn backtest() -> Self {
        metrics().start_backtest();
        Timer {
            start: Instant::now(),
            record_fn: Box::new(|d| {
                metrics().end_backtest();
                metrics().record_backtest(d);
            }),
        }
    }

    /// Create a timer that records data load duration
    pub fn data_load() -> Self {
        Timer {
            start: Instant::now(),
            record_fn: Box::new(|d| metrics().record_data_load(d)),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        // Take the closure and call it
        let record_fn = std::mem::replace(
            &mut self.record_fn,
            Box::new(|_| {}),
        );
        record_fn(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new();

        registry.record_compile(Duration::from_millis(100));
        registry.record_compile(Duration::from_millis(200));
        registry.record_backtest(Duration::from_millis(500));
        registry.record_cache_hit();
        registry.record_cache_miss();

        assert_eq!(registry.compilations_total.load(Ordering::Relaxed), 2);
        assert_eq!(registry.backtests_total.load(Ordering::Relaxed), 1);
        assert_eq!(registry.cache_hits.load(Ordering::Relaxed), 1);
        assert_eq!(registry.cache_misses.load(Ordering::Relaxed), 1);

        let avg = registry.avg_compile_duration().unwrap();
        assert_eq!(avg.as_millis(), 150);
    }

    #[test]
    fn test_prometheus_export() {
        let registry = MetricsRegistry::new();
        registry.record_compile(Duration::from_millis(100));

        let output = registry.export_prometheus();
        assert!(output.contains("sigc_compilations_total 1"));
    }

    #[test]
    fn test_json_export() {
        let registry = MetricsRegistry::new();
        registry.record_backtest(Duration::from_millis(1000));

        let json = registry.export_json();
        assert_eq!(json["counters"]["backtests_total"], 1);
    }

    #[test]
    fn test_global_metrics() {
        let m = metrics();
        m.record_optimization();
        assert!(m.optimizations_total.load(Ordering::Relaxed) >= 1);
    }
}
