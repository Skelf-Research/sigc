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
}

/// Python module
#[pymodule]
fn pysigc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(backtest, m)?)?;
    m.add_class::<CompiledSignal>()?;
    m.add_class::<BacktestResult>()?;
    Ok(())
}
