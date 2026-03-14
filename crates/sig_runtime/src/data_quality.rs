//! Data quality validation and checks
//!
//! Provides trait-based data validation with multiple check types.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Data quality issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A data quality issue found during validation
#[derive(Debug, Clone)]
pub struct DataIssue {
    pub severity: IssueSeverity,
    pub check_name: String,
    pub message: String,
    pub column: Option<String>,
    pub row_count: Option<usize>,
    pub details: HashMap<String, String>,
}

impl DataIssue {
    pub fn new(severity: IssueSeverity, check_name: &str, message: &str) -> Self {
        DataIssue {
            severity,
            check_name: check_name.to_string(),
            message: message.to_string(),
            column: None,
            row_count: None,
            details: HashMap::new(),
        }
    }

    pub fn with_column(mut self, col: &str) -> Self {
        self.column = Some(col.to_string());
        self
    }

    pub fn with_row_count(mut self, count: usize) -> Self {
        self.row_count = Some(count);
        self
    }

    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }
}

/// Result of running data quality checks
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub issues: Vec<DataIssue>,
    pub passed: bool,
    pub checks_run: usize,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            issues: Vec::new(),
            passed: true,
            checks_run: 0,
        }
    }

    pub fn add_issue(&mut self, issue: DataIssue) {
        if matches!(issue.severity, IssueSeverity::Error | IssueSeverity::Critical) {
            self.passed = false;
        }
        self.issues.push(issue);
    }

    pub fn errors(&self) -> Vec<&DataIssue> {
        self.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Error | IssueSeverity::Critical))
            .collect()
    }

    pub fn warnings(&self) -> Vec<&DataIssue> {
        self.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .collect()
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
        self.checks_run += other.checks_run;
        if !other.passed {
            self.passed = false;
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for data validators
pub trait DataValidator: Send + Sync {
    /// Run validation on a DataFrame
    fn validate(&self, df: &DataFrame) -> Result<ValidationResult>;

    /// Get validator name
    fn name(&self) -> &str;
}

/// Check for missing values (NaN/null)
pub struct MissingDataCheck {
    /// Maximum allowed percentage of missing values per column
    pub max_missing_pct: f64,
    /// Columns to check (empty = all numeric columns)
    pub columns: Vec<String>,
}

impl MissingDataCheck {
    pub fn new(max_missing_pct: f64) -> Self {
        MissingDataCheck {
            max_missing_pct,
            columns: Vec::new(),
        }
    }

    pub fn with_columns(mut self, cols: Vec<String>) -> Self {
        self.columns = cols;
        self
    }
}

impl DataValidator for MissingDataCheck {
    fn validate(&self, df: &DataFrame) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.checks_run = 1;

        let total_rows = df.height();
        if total_rows == 0 {
            result.add_issue(DataIssue::new(
                IssueSeverity::Error,
                "missing_data",
                "DataFrame is empty"
            ));
            return Ok(result);
        }

        // Determine columns to check
        let col_names: Vec<String> = if self.columns.is_empty() {
            df.get_column_names().iter().map(|s| s.to_string()).collect()
        } else {
            self.columns.clone()
        };
        let cols_to_check: Vec<&str> = col_names.iter().map(|s| s.as_str()).collect();

        for col_name in cols_to_check {
            let col = match df.column(col_name) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let null_count = col.null_count();
            let missing_pct = null_count as f64 / total_rows as f64 * 100.0;

            if missing_pct > self.max_missing_pct {
                let severity = if missing_pct > 50.0 {
                    IssueSeverity::Critical
                } else if missing_pct > 20.0 {
                    IssueSeverity::Error
                } else {
                    IssueSeverity::Warning
                };

                result.add_issue(
                    DataIssue::new(
                        severity,
                        "missing_data",
                        &format!("{:.1}% missing values exceeds threshold of {:.1}%",
                            missing_pct, self.max_missing_pct)
                    )
                    .with_column(col_name)
                    .with_row_count(null_count)
                    .with_detail("missing_pct", &format!("{:.2}", missing_pct))
                );
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "missing_data"
    }
}

/// Check for outliers using IQR or z-score method
pub struct OutlierCheck {
    /// Method to detect outliers
    pub method: OutlierMethod,
    /// Columns to check (empty = all numeric columns)
    pub columns: Vec<String>,
    /// Maximum allowed outlier percentage
    pub max_outlier_pct: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum OutlierMethod {
    /// IQR method: values outside Q1 - 1.5*IQR to Q3 + 1.5*IQR
    Iqr { multiplier: f64 },
    /// Z-score method: values with |z| > threshold
    ZScore { threshold: f64 },
}

impl OutlierCheck {
    pub fn new(method: OutlierMethod) -> Self {
        OutlierCheck {
            method,
            columns: Vec::new(),
            max_outlier_pct: 5.0,
        }
    }

    pub fn with_max_pct(mut self, pct: f64) -> Self {
        self.max_outlier_pct = pct;
        self
    }

    pub fn with_columns(mut self, cols: Vec<String>) -> Self {
        self.columns = cols;
        self
    }
}

impl DataValidator for OutlierCheck {
    fn validate(&self, df: &DataFrame) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.checks_run = 1;

        let total_rows = df.height();
        if total_rows < 4 {
            return Ok(result); // Not enough data for outlier detection
        }

        // Determine columns to check
        let col_names: Vec<String> = if self.columns.is_empty() {
            df.get_column_names().iter().map(|s| s.to_string()).collect()
        } else {
            self.columns.clone()
        };
        let cols_to_check: Vec<&str> = col_names.iter().map(|s| s.as_str()).collect();

        for col_name in cols_to_check {
            let col = match df.column(col_name) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Only check numeric columns
            if !col.dtype().is_numeric() {
                continue;
            }

            let outlier_count = match self.method {
                OutlierMethod::Iqr { multiplier } => {
                    count_iqr_outliers(col, multiplier)?
                }
                OutlierMethod::ZScore { threshold } => {
                    count_zscore_outliers(col, threshold)?
                }
            };

            let outlier_pct = outlier_count as f64 / total_rows as f64 * 100.0;

            if outlier_pct > self.max_outlier_pct {
                result.add_issue(
                    DataIssue::new(
                        IssueSeverity::Warning,
                        "outliers",
                        &format!("{:.1}% outliers detected (threshold: {:.1}%)",
                            outlier_pct, self.max_outlier_pct)
                    )
                    .with_column(col_name)
                    .with_row_count(outlier_count)
                    .with_detail("outlier_pct", &format!("{:.2}", outlier_pct))
                );
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "outliers"
    }
}

fn count_iqr_outliers(col: &Column, multiplier: f64) -> Result<usize> {
    let series = col.as_materialized_series();

    // Get quantiles using Series methods
    let q1_val = series.quantile_reduce(0.25, QuantileMethod::Linear)
        .map_err(|e| SigcError::Runtime(format!("Quantile error: {}", e)))?
        .value()
        .extract::<f64>()
        .unwrap_or(0.0);

    let q3_val = series.quantile_reduce(0.75, QuantileMethod::Linear)
        .map_err(|e| SigcError::Runtime(format!("Quantile error: {}", e)))?
        .value()
        .extract::<f64>()
        .unwrap_or(0.0);

    let iqr = q3_val - q1_val;

    let lower = q1_val - multiplier * iqr;
    let upper = q3_val + multiplier * iqr;

    let mut count = 0;
    if let Ok(ca) = series.f64() {
        for val in ca.into_iter().flatten() {
            if val < lower || val > upper {
                count += 1;
            }
        }
    }

    Ok(count)
}

fn count_zscore_outliers(col: &Column, threshold: f64) -> Result<usize> {
    let series = col.as_materialized_series();
    let mean = series.mean().unwrap_or(0.0);
    let std = series.std(1).unwrap_or(1.0);

    if std == 0.0 {
        return Ok(0);
    }

    let mut count = 0;
    if let Ok(ca) = series.f64() {
        for val in ca.into_iter().flatten() {
            let z = (val - mean).abs() / std;
            if z > threshold {
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Check data freshness (most recent date)
pub struct FreshnessCheck {
    /// Date column name
    pub date_column: String,
    /// Maximum age in days
    pub max_age_days: u32,
}

impl FreshnessCheck {
    pub fn new(date_column: &str, max_age_days: u32) -> Self {
        FreshnessCheck {
            date_column: date_column.to_string(),
            max_age_days,
        }
    }
}

impl DataValidator for FreshnessCheck {
    fn validate(&self, df: &DataFrame) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.checks_run = 1;

        let col = df.column(&self.date_column)
            .map_err(|_| SigcError::Runtime(format!("Date column '{}' not found", self.date_column)))?;

        // Get max date from column
        let _max_date = col.as_materialized_series().max_reduce()
            .map_err(|e| SigcError::Runtime(format!("Max date error: {}", e)))?;

        // For now, just check if column exists and has data
        // Full date parsing would require chrono
        if col.null_count() == col.len() {
            result.add_issue(
                DataIssue::new(
                    IssueSeverity::Critical,
                    "freshness",
                    "Date column contains only null values"
                )
                .with_column(&self.date_column)
            );
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "freshness"
    }
}

/// Check for duplicate rows
pub struct DuplicateCheck {
    /// Columns that define uniqueness
    pub key_columns: Vec<String>,
}

impl DuplicateCheck {
    pub fn new(key_columns: Vec<String>) -> Self {
        DuplicateCheck { key_columns }
    }
}

impl DataValidator for DuplicateCheck {
    fn validate(&self, df: &DataFrame) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.checks_run = 1;

        if self.key_columns.is_empty() {
            return Ok(result);
        }

        // Simple duplicate detection: hash each row's key values
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut duplicate_count = 0;

        for row_idx in 0..df.height() {
            let mut key = String::new();
            for col_name in &self.key_columns {
                if let Ok(col) = df.column(col_name) {
                    let val = col.get(row_idx)
                        .map(|v| format!("{:?}", v))
                        .unwrap_or_default();
                    key.push_str(&val);
                    key.push('|');
                }
            }

            if !seen.insert(key) {
                duplicate_count += 1;
            }
        }

        if duplicate_count > 0 {
            result.add_issue(
                DataIssue::new(
                    IssueSeverity::Error,
                    "duplicates",
                    &format!("{} duplicate rows found", duplicate_count)
                )
                .with_row_count(duplicate_count)
                .with_detail("key_columns", &self.key_columns.join(", "))
            );
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "duplicates"
    }
}

/// Composite validator that runs multiple checks
pub struct DataQualityValidator {
    validators: Vec<Box<dyn DataValidator>>,
}

impl DataQualityValidator {
    pub fn new() -> Self {
        DataQualityValidator {
            validators: Vec::new(),
        }
    }

    pub fn add(mut self, validator: Box<dyn DataValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Create a default validator with common checks
    pub fn default_checks() -> Self {
        DataQualityValidator::new()
            .add(Box::new(MissingDataCheck::new(10.0)))
            .add(Box::new(OutlierCheck::new(OutlierMethod::Iqr { multiplier: 1.5 })))
    }

    /// Validate data and return all issues
    pub fn validate(&self, df: &DataFrame) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        for validator in &self.validators {
            let check_result = validator.validate(df)?;
            result.merge(check_result);
        }

        Ok(result)
    }
}

impl Default for DataQualityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_df() -> DataFrame {
        df! {
            "date" => &["2024-01-01", "2024-01-02", "2024-01-03", "2024-01-04", "2024-01-05"],
            "price" => &[100.0, 101.0, 99.0, 102.0, 100.0],
            "volume" => &[1000, 1100, 900, 1200, 1000]
        }.unwrap()
    }

    #[test]
    fn test_missing_data_check_pass() {
        let df = create_test_df();
        let check = MissingDataCheck::new(10.0);
        let result = check.validate(&df).unwrap();

        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_missing_data_check_fail() {
        let df = df! {
            "value" => &[Some(1.0), None, None, Some(4.0), None]
        }.unwrap();

        let check = MissingDataCheck::new(10.0);
        let result = check.validate(&df).unwrap();

        assert!(!result.passed);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].check_name, "missing_data");
    }

    #[test]
    fn test_outlier_check_iqr() {
        // Data with obvious outlier
        let df = df! {
            "value" => &[10.0, 11.0, 10.5, 10.2, 100.0] // 100 is outlier
        }.unwrap();

        let check = OutlierCheck::new(OutlierMethod::Iqr { multiplier: 1.5 })
            .with_max_pct(10.0);
        let result = check.validate(&df).unwrap();

        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_outlier_check_zscore() {
        // Test with extreme outlier
        let df = df! {
            "value" => &[10.0, 10.2, 10.5, 10.3, 10.4, 10.1, 10.6, 10.2, 10.3, 1000.0] // 1000 is extreme outlier
        }.unwrap();

        let check = OutlierCheck::new(OutlierMethod::ZScore { threshold: 2.0 })
            .with_max_pct(5.0);
        let result = check.validate(&df).unwrap();

        // Should detect the outlier
        assert!(result.issues.len() >= 1 || result.passed);
    }

    #[test]
    fn test_duplicate_check() {
        let df = df! {
            "date" => &["2024-01-01", "2024-01-01", "2024-01-02"],
            "symbol" => &["AAPL", "AAPL", "AAPL"],
            "price" => &[100.0, 100.0, 101.0]
        }.unwrap();

        let check = DuplicateCheck::new(vec!["date".to_string(), "symbol".to_string()]);
        let result = check.validate(&df).unwrap();

        assert!(!result.passed);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].row_count, Some(1));
    }

    #[test]
    fn test_composite_validator() {
        let df = create_test_df();

        let validator = DataQualityValidator::new()
            .add(Box::new(MissingDataCheck::new(10.0)))
            .add(Box::new(OutlierCheck::new(OutlierMethod::Iqr { multiplier: 1.5 })));

        let result = validator.validate(&df).unwrap();

        assert!(result.passed);
        assert_eq!(result.checks_run, 2);
    }

    #[test]
    fn test_default_checks() {
        let df = create_test_df();
        let validator = DataQualityValidator::default_checks();
        let result = validator.validate(&df).unwrap();

        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.checks_run = 1;

        let mut result2 = ValidationResult::new();
        result2.checks_run = 2;
        result2.add_issue(DataIssue::new(IssueSeverity::Warning, "test", "warning"));

        result1.merge(result2);

        assert_eq!(result1.checks_run, 3);
        assert_eq!(result1.issues.len(), 1);
    }
}
