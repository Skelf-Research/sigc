//! Python bindings for sigc using PyO3

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

/// Compile a .sig source string to IR
#[pyfunction]
fn compile(source: &str) -> PyResult<CompiledSignal> {
    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source)
        .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))?;

    Ok(CompiledSignal {
        nodes: ir.nodes.len(),
        outputs: ir.outputs.len(),
        parameters: ir.metadata.parameters.iter()
            .map(|p| (p.name.clone(), p.default_value))
            .collect(),
        data_sources: ir.metadata.data_sources.iter()
            .map(|d| (d.name.clone(), d.path.clone()))
            .collect(),
        ir,
    })
}

/// Run a backtest on a .sig source string
#[pyfunction]
fn backtest(source: &str) -> PyResult<BacktestResult> {
    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source)
        .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))?;

    let mut runtime = sig_runtime::Runtime::new();
    let report = runtime.run_ir(&ir)
        .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))?;

    Ok(BacktestResult {
        total_return: report.metrics.total_return,
        annualized_return: report.metrics.annualized_return,
        sharpe_ratio: report.metrics.sharpe_ratio,
        max_drawdown: report.metrics.max_drawdown,
        turnover: report.metrics.turnover,
        returns_series: report.returns_series,
    })
}

/// Compiled signal representation
#[pyclass]
#[derive(Clone)]
struct CompiledSignal {
    #[pyo3(get)]
    nodes: usize,
    #[pyo3(get)]
    outputs: usize,
    #[pyo3(get)]
    parameters: Vec<(String, f64)>,
    #[pyo3(get)]
    data_sources: Vec<(String, String)>,
    ir: sig_types::Ir,
}

#[pymethods]
impl CompiledSignal {
    fn __repr__(&self) -> String {
        format!(
            "CompiledSignal(nodes={}, outputs={}, params={:?})",
            self.nodes, self.outputs, self.parameters
        )
    }

    /// Run backtest on this compiled signal
    fn backtest(&self) -> PyResult<BacktestResult> {
        let mut runtime = sig_runtime::Runtime::new();
        let report = runtime.run_ir(&self.ir)
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))?;

        Ok(BacktestResult {
            total_return: report.metrics.total_return,
            annualized_return: report.metrics.annualized_return,
            sharpe_ratio: report.metrics.sharpe_ratio,
            max_drawdown: report.metrics.max_drawdown,
            turnover: report.metrics.turnover,
            returns_series: report.returns_series,
        })
    }
}

/// Backtest result with performance metrics
#[pyclass]
#[derive(Clone)]
struct BacktestResult {
    #[pyo3(get)]
    total_return: f64,
    #[pyo3(get)]
    annualized_return: f64,
    #[pyo3(get)]
    sharpe_ratio: f64,
    #[pyo3(get)]
    max_drawdown: f64,
    #[pyo3(get)]
    turnover: f64,
    /// Raw per-period portfolio returns (input to the rigor functions).
    #[pyo3(get)]
    returns_series: Vec<f64>,
}

#[pymethods]
impl BacktestResult {
    fn __repr__(&self) -> String {
        format!(
            "BacktestResult(return={:.2}%, sharpe={:.2}, drawdown={:.2}%)",
            self.total_return * 100.0,
            self.sharpe_ratio,
            self.max_drawdown * 100.0
        )
    }

    /// Convert to dictionary
    fn to_dict(&self) -> PyResult<std::collections::HashMap<String, f64>> {
        let mut map = std::collections::HashMap::new();
        map.insert("total_return".to_string(), self.total_return);
        map.insert("annualized_return".to_string(), self.annualized_return);
        map.insert("sharpe_ratio".to_string(), self.sharpe_ratio);
        map.insert("max_drawdown".to_string(), self.max_drawdown);
        map.insert("turnover".to_string(), self.turnover);
        Ok(map)
    }

    /// Compute the full statistical-rigor report for this backtest.
    #[pyo3(signature = (num_trials=1, variance_of_trial_sr=0.0, periods_per_year=252.0))]
    fn rigor(
        &self,
        num_trials: usize,
        variance_of_trial_sr: f64,
        periods_per_year: f64,
    ) -> PyRigorReport {
        PyRigorReport::from_rust(sig_runtime::RigorReport::compute(
            &self.returns_series,
            num_trials,
            variance_of_trial_sr,
            periods_per_year,
        ))
    }
}

// ===========================================================================
// Statistical rigor (Bailey & Lopez de Prado, Harvey-Liu-Zhu, CSCV, ...)
// ===========================================================================

/// Bonferroni multiple-testing haircut applied to a Sharpe.
#[pyclass(name = "Haircut")]
#[derive(Clone)]
struct PyHaircut {
    #[pyo3(get)] p_single: f64,
    #[pyo3(get)] p_adjusted: f64,
    #[pyo3(get)] haircut_sharpe: f64,
    #[pyo3(get)] haircut_pct: f64,
}

#[pymethods]
impl PyHaircut {
    fn __repr__(&self) -> String {
        format!(
            "Haircut(p_adj={:.4}, haircut_sharpe={:.3}, haircut_pct={:.1}%)",
            self.p_adjusted, self.haircut_sharpe, self.haircut_pct * 100.0
        )
    }
}

/// Probability of Backtest Overfitting via CSCV.
#[pyclass(name = "PboResult")]
#[derive(Clone)]
struct PyPboResult {
    #[pyo3(get)] pbo: f64,
    #[pyo3(get)] n_combinations: usize,
    #[pyo3(get)] logits: Vec<f64>,
}

#[pymethods]
impl PyPboResult {
    fn __repr__(&self) -> String {
        format!("PboResult(pbo={:.3}, n_combinations={})", self.pbo, self.n_combinations)
    }
}

/// Bundled rigor report for a single return series.
#[pyclass(name = "RigorReport")]
#[derive(Clone)]
struct PyRigorReport {
    #[pyo3(get)] annualized_sharpe: f64,
    #[pyo3(get)] psr: f64,
    #[pyo3(get)] deflated_sharpe: f64,
    #[pyo3(get)] permutation_pvalue: f64,
    #[pyo3(get)] min_track_record: Option<f64>,
    #[pyo3(get)] haircut: PyHaircut,
    // Moment summary
    #[pyo3(get)] n: usize,
    #[pyo3(get)] mean: f64,
    #[pyo3(get)] std: f64,
    #[pyo3(get)] skew: f64,
    #[pyo3(get)] kurt: f64,
    #[pyo3(get)] sharpe: f64,
}

impl PyRigorReport {
    fn from_rust(r: sig_runtime::RigorReport) -> PyRigorReport {
        PyRigorReport {
            annualized_sharpe: r.annualized_sharpe,
            psr: r.psr,
            deflated_sharpe: r.deflated_sharpe,
            permutation_pvalue: r.permutation_pvalue,
            min_track_record: r.min_track_record,
            haircut: PyHaircut {
                p_single: r.haircut.p_single,
                p_adjusted: r.haircut.p_adjusted,
                haircut_sharpe: r.haircut.haircut_sharpe,
                haircut_pct: r.haircut.haircut_pct,
            },
            n: r.stats.n,
            mean: r.stats.mean,
            std: r.stats.std,
            skew: r.stats.skew,
            kurt: r.stats.kurt,
            sharpe: r.stats.sharpe,
        }
    }
}

#[pymethods]
impl PyRigorReport {
    fn __repr__(&self) -> String {
        format!(
            "RigorReport(ann_sharpe={:.2}, psr={:.3}, dsr={:.3}, perm_p={:.3})",
            self.annualized_sharpe, self.psr, self.deflated_sharpe, self.permutation_pvalue
        )
    }
}

/// Probability that the true per-period Sharpe exceeds `benchmark_sr`.
#[pyfunction]
#[pyo3(signature = (returns, benchmark_sr=0.0))]
fn probabilistic_sharpe_ratio(returns: Vec<f64>, benchmark_sr: f64) -> f64 {
    let stats = sig_runtime::ReturnStats::from_returns(&returns);
    sig_runtime::probabilistic_sharpe_ratio(&stats, benchmark_sr)
}

/// Deflated Sharpe: PSR vs the expected max Sharpe across `num_trials` trials.
#[pyfunction]
#[pyo3(signature = (returns, num_trials, variance_of_trial_sr))]
fn deflated_sharpe_ratio(returns: Vec<f64>, num_trials: usize, variance_of_trial_sr: f64) -> f64 {
    let stats = sig_runtime::ReturnStats::from_returns(&returns);
    sig_runtime::deflated_sharpe_ratio(&stats, num_trials, variance_of_trial_sr)
}

/// Bonferroni haircut for `num_tests` independent strategies.
#[pyfunction]
fn bonferroni_haircut(returns: Vec<f64>, num_tests: usize) -> PyHaircut {
    let stats = sig_runtime::ReturnStats::from_returns(&returns);
    let h = sig_runtime::bonferroni_haircut(&stats, num_tests);
    PyHaircut {
        p_single: h.p_single,
        p_adjusted: h.p_adjusted,
        haircut_sharpe: h.haircut_sharpe,
        haircut_pct: h.haircut_pct,
    }
}

/// Holm step-down adjusted p-values.
#[pyfunction]
fn holm_adjust(pvalues: Vec<f64>) -> Vec<f64> {
    sig_runtime::holm_adjust(&pvalues)
}

/// Benjamini-Hochberg-Yekutieli adjusted p-values.
#[pyfunction]
fn bhy_adjust(pvalues: Vec<f64>) -> Vec<f64> {
    sig_runtime::bhy_adjust(&pvalues)
}

/// Deterministic sign-flip permutation p-value of the Sharpe ratio.
#[pyfunction]
#[pyo3(signature = (returns, num_perms=1000, seed=0xC0FFEE))]
fn permutation_sharpe_pvalue(returns: Vec<f64>, num_perms: usize, seed: u64) -> f64 {
    sig_runtime::permutation_sharpe_pvalue(&returns, num_perms, seed)
}

/// Min observations until PSR vs `benchmark_sr` reaches `confidence`.
#[pyfunction]
#[pyo3(signature = (returns, benchmark_sr=0.0, confidence=0.95))]
fn min_track_record_length(returns: Vec<f64>, benchmark_sr: f64, confidence: f64) -> Option<f64> {
    let stats = sig_runtime::ReturnStats::from_returns(&returns);
    sig_runtime::min_track_record_length(&stats, benchmark_sr, confidence)
}

/// Probability of Backtest Overfitting via Combinatorially-Symmetric CV.
/// `matrix[c]` is the per-period return series for configuration `c`.
#[pyfunction]
#[pyo3(signature = (matrix, n_splits=10))]
fn pbo_cscv(matrix: Vec<Vec<f64>>, n_splits: usize) -> PyResult<PyPboResult> {
    let r = sig_runtime::pbo_cscv(&matrix, n_splits)
        .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))?;
    Ok(PyPboResult { pbo: r.pbo, n_combinations: r.n_combinations, logits: r.logits })
}

/// Full rigor report for an arbitrary return series.
#[pyfunction]
#[pyo3(signature = (returns, num_trials=1, variance_of_trial_sr=0.0, periods_per_year=252.0))]
fn rigor_report(
    returns: Vec<f64>,
    num_trials: usize,
    variance_of_trial_sr: f64,
    periods_per_year: f64,
) -> PyRigorReport {
    PyRigorReport::from_rust(sig_runtime::RigorReport::compute(
        &returns,
        num_trials,
        variance_of_trial_sr,
        periods_per_year,
    ))
}

/// Python module
#[pymodule]
fn pysigc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(backtest, m)?)?;
    m.add_class::<CompiledSignal>()?;
    m.add_class::<BacktestResult>()?;

    // Rigor
    m.add_class::<PyHaircut>()?;
    m.add_class::<PyPboResult>()?;
    m.add_class::<PyRigorReport>()?;
    m.add_function(wrap_pyfunction!(probabilistic_sharpe_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(deflated_sharpe_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(bonferroni_haircut, m)?)?;
    m.add_function(wrap_pyfunction!(holm_adjust, m)?)?;
    m.add_function(wrap_pyfunction!(bhy_adjust, m)?)?;
    m.add_function(wrap_pyfunction!(permutation_sharpe_pvalue, m)?)?;
    m.add_function(wrap_pyfunction!(min_track_record_length, m)?)?;
    m.add_function(wrap_pyfunction!(pbo_cscv, m)?)?;
    m.add_function(wrap_pyfunction!(rigor_report, m)?)?;
    Ok(())
}
