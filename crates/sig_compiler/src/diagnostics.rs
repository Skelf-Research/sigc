//! Diagnostics for sigc compiler
//!
//! Provides structured error and warning information for IDE integration.

use std::ops::Range;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A diagnostic message with source location
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Range<usize>,
    pub related: Vec<RelatedInfo>,
    pub code: Option<String>,
}

/// Related information for a diagnostic
#[derive(Debug, Clone)]
pub struct RelatedInfo {
    pub message: String,
    pub span: Range<usize>,
}

/// Position in source code
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Line and column range
#[derive(Debug, Clone)]
pub struct LocationRange {
    pub start: Position,
    pub end: Position,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error(message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            related: Vec::new(),
            code: None,
        }
    }

    /// Create a new warning diagnostic
    pub fn warning(message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span,
            related: Vec::new(),
            code: None,
        }
    }

    /// Add an error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add related information
    pub fn with_related(mut self, message: impl Into<String>, span: Range<usize>) -> Self {
        self.related.push(RelatedInfo {
            message: message.into(),
            span,
        });
        self
    }

    /// Convert byte span to line/column range
    pub fn to_location(&self, source: &str) -> LocationRange {
        LocationRange {
            start: byte_to_position(source, self.span.start),
            end: byte_to_position(source, self.span.end),
        }
    }
}

/// Convert byte offset to line/column position
pub fn byte_to_position(source: &str, byte_offset: usize) -> Position {
    let mut line = 1;
    let mut column = 1;

    for (i, c) in source.char_indices() {
        if i >= byte_offset {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    Position { line, column }
}

/// Diagnostic collector for accumulating errors during compilation
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, message: impl Into<String>, span: Range<usize>) {
        self.add(Diagnostic::error(message, span));
    }

    pub fn warning(&mut self, message: impl Into<String>, span: Range<usize>) {
        self.add(Diagnostic::warning(message, span));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Format all diagnostics for display
    pub fn format(&self, source: &str) -> String {
        self.diagnostics
            .iter()
            .map(|d| format_diagnostic(source, d))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Format a single diagnostic for display
pub fn format_diagnostic(source: &str, diagnostic: &Diagnostic) -> String {
    let loc = diagnostic.to_location(source);

    let severity_str = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    };

    let code_str = diagnostic.code
        .as_ref()
        .map(|c| format!("[{}] ", c))
        .unwrap_or_default();

    // Get the source line
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = loc.start.line.saturating_sub(1);
    let line = lines.get(line_idx).unwrap_or(&"");

    let mut result = format!(
        "{}{}: {}\n  --> line {}:{}\n",
        code_str, severity_str, diagnostic.message, loc.start.line, loc.start.column
    );

    let line_num_str = loc.start.line.to_string();
    let padding = " ".repeat(line_num_str.len());

    result.push_str(&format!("   {} |\n", padding));
    result.push_str(&format!("   {} | {}\n", line_num_str, line));

    // Underline
    let underline_start = loc.start.column.saturating_sub(1);
    let underline_len = if loc.start.line == loc.end.line {
        loc.end.column.saturating_sub(loc.start.column).max(1)
    } else {
        line.len().saturating_sub(underline_start).max(1)
    };

    result.push_str(&format!(
        "   {} | {}{}\n",
        padding,
        " ".repeat(underline_start),
        "^".repeat(underline_len)
    ));

    // Related info
    for related in &diagnostic.related {
        let rel_loc = byte_to_position(source, related.span.start);
        result.push_str(&format!(
            "   {} = note: {} (line {}:{})\n",
            padding, related.message, rel_loc.line, rel_loc.column
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("test error", 0..5)
            .with_code("E0001")
            .with_related("see here", 10..15);

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "test error");
        assert_eq!(diag.code, Some("E0001".to_string()));
        assert_eq!(diag.related.len(), 1);
    }

    #[test]
    fn test_byte_to_position() {
        let source = "line1\nline2\nline3";

        assert_eq!(byte_to_position(source, 0).line, 1);
        assert_eq!(byte_to_position(source, 0).column, 1);

        assert_eq!(byte_to_position(source, 6).line, 2);
        assert_eq!(byte_to_position(source, 6).column, 1);

        assert_eq!(byte_to_position(source, 8).line, 2);
        assert_eq!(byte_to_position(source, 8).column, 3);
    }

    #[test]
    fn test_diagnostic_collector() {
        let mut collector = DiagnosticCollector::new();
        collector.error("error 1", 0..5);
        collector.warning("warning 1", 10..15);

        assert!(collector.has_errors());
        assert_eq!(collector.diagnostics().len(), 2);
    }

    #[test]
    fn test_format_diagnostic() {
        let source = "let x = foo\nlet y = bar";
        let diag = Diagnostic::error("undefined identifier 'foo'", 8..11);

        let formatted = format_diagnostic(source, &diag);
        assert!(formatted.contains("error:"));
        assert!(formatted.contains("undefined identifier 'foo'"));
        assert!(formatted.contains("line 1:9"));
    }
}
