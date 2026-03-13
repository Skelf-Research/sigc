//! Visualization module for backtest results
//!
//! Generates SVG charts for equity curves, drawdowns, and other analytics.

use sig_types::{BacktestReport, Result, SigcError};
use std::fs;

/// Chart generator for backtest visualization
pub struct ChartGenerator {
    width: u32,
    height: u32,
    padding: u32,
}

impl Default for ChartGenerator {
    fn default() -> Self {
        ChartGenerator {
            width: 800,
            height: 400,
            padding: 50,
        }
    }
}

impl ChartGenerator {
    /// Create a new chart generator
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chart dimensions
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Generate equity curve SVG
    pub fn equity_curve(&self, returns: &[f64], title: &str) -> String {
        let cumulative = self.cumulative_returns(returns);
        self.line_chart(&cumulative, title, "Cumulative Return", "#2196F3")
    }

    /// Generate drawdown chart SVG
    pub fn drawdown_chart(&self, returns: &[f64], title: &str) -> String {
        let drawdowns = self.calculate_drawdowns(returns);
        self.line_chart(&drawdowns, title, "Drawdown", "#F44336")
    }

    /// Generate rolling Sharpe chart
    pub fn rolling_sharpe(&self, returns: &[f64], window: usize, title: &str) -> String {
        let rolling = self.calculate_rolling_sharpe(returns, window);
        self.line_chart(&rolling, title, "Rolling Sharpe", "#4CAF50")
    }

    /// Generate a basic line chart
    fn line_chart(&self, data: &[f64], title: &str, y_label: &str, color: &str) -> String {
        if data.is_empty() {
            return self.empty_chart(title);
        }

        let chart_width = self.width - 2 * self.padding;
        let chart_height = self.height - 2 * self.padding;

        let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = if (max_val - min_val).abs() < 1e-10 { 1.0 } else { max_val - min_val };

        // Generate path
        let points: Vec<String> = data.iter().enumerate().map(|(i, &val)| {
            let x = self.padding + (i as f64 / (data.len() - 1).max(1) as f64 * chart_width as f64) as u32;
            let y = self.padding + ((max_val - val) / range * chart_height as f64) as u32;
            format!("{},{}", x, y)
        }).collect();

        let path = points.join(" L ");

        format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
  <style>
    .title {{ font: bold 16px sans-serif; }}
    .label {{ font: 12px sans-serif; fill: #666; }}
    .axis {{ stroke: #ccc; stroke-width: 1; }}
    .grid {{ stroke: #eee; stroke-width: 1; }}
    .line {{ fill: none; stroke: {}; stroke-width: 2; }}
  </style>

  <!-- Title -->
  <text x="{}" y="25" class="title" text-anchor="middle">{}</text>

  <!-- Y-axis label -->
  <text x="15" y="{}" class="label" transform="rotate(-90, 15, {})">{}</text>

  <!-- Axes -->
  <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>

  <!-- Grid lines -->
  {}

  <!-- Data line -->
  <path d="M {}" class="line"/>

  <!-- Y-axis labels -->
  <text x="{}" y="{}" class="label" text-anchor="end">{:.2}</text>
  <text x="{}" y="{}" class="label" text-anchor="end">{:.2}</text>
</svg>"#,
            self.width, self.height,
            color,
            self.width / 2, title,
            self.height / 2, self.height / 2, y_label,
            self.padding, self.padding, self.padding, self.height - self.padding,
            self.padding, self.height - self.padding, self.width - self.padding, self.height - self.padding,
            self.generate_grid_lines(),
            path,
            self.padding - 5, self.padding + 5, max_val,
            self.padding - 5, self.height - self.padding, min_val
        )
    }

    /// Generate grid lines
    fn generate_grid_lines(&self) -> String {
        let mut lines = Vec::new();
        let chart_height = self.height - 2 * self.padding;

        for i in 1..5 {
            let y = self.padding + i * chart_height / 5;
            lines.push(format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="grid"/>"#,
                self.padding, y, self.width - self.padding, y
            ));
        }

        lines.join("\n  ")
    }

    /// Generate empty chart placeholder
    fn empty_chart(&self, title: &str) -> String {
        format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
  <text x="{}" y="{}" text-anchor="middle" font-family="sans-serif">{} - No data</text>
</svg>"#,
            self.width, self.height,
            self.width / 2, self.height / 2, title
        )
    }

    /// Calculate cumulative returns
    fn cumulative_returns(&self, returns: &[f64]) -> Vec<f64> {
        let mut cumulative = Vec::with_capacity(returns.len());
        let mut total = 1.0;

        for &r in returns {
            total *= 1.0 + r;
            cumulative.push(total - 1.0);
        }

        cumulative
    }

    /// Calculate drawdowns
    fn calculate_drawdowns(&self, returns: &[f64]) -> Vec<f64> {
        let mut drawdowns = Vec::with_capacity(returns.len());
        let mut peak: f64 = 1.0;
        let mut value: f64 = 1.0;

        for &r in returns {
            value *= 1.0 + r;
            peak = peak.max(value);
            let dd = (value - peak) / peak;
            drawdowns.push(dd);
        }

        drawdowns
    }

    /// Calculate rolling Sharpe ratio
    fn calculate_rolling_sharpe(&self, returns: &[f64], window: usize) -> Vec<f64> {
        if returns.len() < window {
            return vec![0.0; returns.len()];
        }

        let mut result = vec![0.0; window - 1];
        let sqrt_252 = (252.0_f64).sqrt();

        for i in window..=returns.len() {
            let window_returns = &returns[i - window..i];
            let mean: f64 = window_returns.iter().sum::<f64>() / window as f64;
            let variance: f64 = window_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / window as f64;
            let std = variance.sqrt();

            let sharpe = if std > 1e-10 { mean / std * sqrt_252 } else { 0.0 };
            result.push(sharpe);
        }

        result
    }
}

/// Report visualizer combining multiple charts
pub struct ReportVisualizer {
    generator: ChartGenerator,
}

impl ReportVisualizer {
    /// Create a new report visualizer
    pub fn new() -> Self {
        ReportVisualizer {
            generator: ChartGenerator::new(),
        }
    }

    /// Generate full HTML report with charts
    pub fn generate_html(&self, report: &BacktestReport, returns: &[f64]) -> String {
        let equity_svg = self.generator.equity_curve(returns, "Equity Curve");
        let drawdown_svg = self.generator.drawdown_chart(returns, "Drawdown");
        let sharpe_svg = self.generator.rolling_sharpe(returns, 63, "Rolling Sharpe (63-day)");

        let metrics = &report.metrics;

        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Backtest Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f5f5f5; }}
        h1 {{ color: #333; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }}
        .metric-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #333; }}
        .metric-label {{ font-size: 14px; color: #666; margin-top: 5px; }}
        .chart-container {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 20px; }}
        .positive {{ color: #2e7d32; }}
        .negative {{ color: #c62828; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Backtest Report</h1>

        <div class="metrics">
            <div class="metric-card">
                <div class="metric-value {}">{:.2}%</div>
                <div class="metric-label">Total Return</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.2}</div>
                <div class="metric-label">Sharpe Ratio</div>
            </div>
            <div class="metric-card">
                <div class="metric-value negative">{:.2}%</div>
                <div class="metric-label">Max Drawdown</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}x</div>
                <div class="metric-label">Annual Turnover</div>
            </div>
        </div>

        <div class="chart-container">
            {}
        </div>

        <div class="chart-container">
            {}
        </div>

        <div class="chart-container">
            {}
        </div>
    </div>
</body>
</html>"#,
            if metrics.total_return >= 0.0 { "positive" } else { "negative" },
            metrics.total_return * 100.0,
            metrics.sharpe_ratio,
            metrics.max_drawdown * 100.0,
            metrics.turnover,
            equity_svg,
            drawdown_svg,
            sharpe_svg
        )
    }

    /// Save HTML report to file
    pub fn save_html(&self, report: &BacktestReport, returns: &[f64], path: &str) -> Result<()> {
        let html = self.generate_html(report, returns);
        fs::write(path, html)
            .map_err(|e| SigcError::Runtime(format!("Failed to write HTML: {}", e)))
    }
}

impl Default for ReportVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sig_types::BacktestMetrics;

    fn sample_returns() -> Vec<f64> {
        vec![0.01, -0.005, 0.02, 0.015, -0.01, 0.008, -0.003, 0.012, 0.005, -0.007]
    }

    #[test]
    fn test_equity_curve() {
        let generator = ChartGenerator::new();
        let svg = generator.equity_curve(&sample_returns(), "Test");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test"));
    }

    #[test]
    fn test_drawdown_chart() {
        let generator = ChartGenerator::new();
        let svg = generator.drawdown_chart(&sample_returns(), "Drawdown");
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_empty_data() {
        let generator = ChartGenerator::new();
        let svg = generator.equity_curve(&[], "Empty");
        assert!(svg.contains("No data"));
    }

    #[test]
    fn test_cumulative_returns() {
        let generator = ChartGenerator::new();
        let returns = vec![0.1, 0.1, -0.1];
        let cumulative = generator.cumulative_returns(&returns);

        assert_eq!(cumulative.len(), 3);
        assert!((cumulative[0] - 0.1).abs() < 1e-10);
        assert!((cumulative[1] - 0.21).abs() < 1e-10);
    }

    #[test]
    fn test_html_report() {
        let visualizer = ReportVisualizer::new();
        let report = BacktestReport {
            plan_hash: "test".to_string(),
            executed_at: 0,
            metrics: BacktestMetrics {
                total_return: 0.15,
                annualized_return: 0.12,
                sharpe_ratio: 1.5,
                max_drawdown: 0.08,
                turnover: 2.4,
                sortino_ratio: 2.0,
                calmar_ratio: 1.5,
                win_rate: 0.55,
                profit_factor: 1.3,
            },
            positions: None,
            returns_series: vec![0.01, -0.005, 0.02],
            benchmark_metrics: None,
        };

        let html = visualizer.generate_html(&report, &sample_returns());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("15.00%"));
        assert!(html.contains("1.50"));
    }
}
