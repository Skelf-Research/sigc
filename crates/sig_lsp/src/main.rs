//! sigc Language Server Protocol implementation
//!
//! Provides IDE features for sigc files:
//! - Diagnostics (parse errors, undefined identifiers)
//! - Hover (type information, function signatures)
//! - Completion (operators, functions, variables)
//! - Go to definition
//! - Document symbols

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use sig_compiler::Compiler;

/// Document state stored by the server
struct Document {
    content: String,
    #[allow(dead_code)]
    version: i32,
}

/// The sigc language server
struct SigcServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Document>>>,
    compiler: Compiler,
}

impl SigcServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            compiler: Compiler::new(),
        }
    }

    /// Analyze a document and publish diagnostics
    async fn analyze(&self, uri: &Url, content: &str) {
        let diagnostics = self.get_diagnostics(content);
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    /// Get diagnostics for source code
    fn get_diagnostics(&self, source: &str) -> Vec<Diagnostic> {
        match self.compiler.compile(source) {
            Ok(_) => vec![],
            Err(e) => {
                // Parse the error message to extract location
                let message = e.to_string();

                // Try to extract line number from error message
                // Format: "error: ... --> line X:Y"
                let (line, col) = extract_location(&message).unwrap_or((0, 0));

                vec![Diagnostic {
                    range: Range {
                        start: Position {
                            line: line.saturating_sub(1) as u32,
                            character: col.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: line.saturating_sub(1) as u32,
                            character: (col + 10) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("sigc".to_string()),
                    message: clean_error_message(&message),
                    related_information: None,
                    tags: None,
                    data: None,
                }]
            }
        }
    }

    /// Get hover information at a position
    fn get_hover(&self, source: &str, position: Position) -> Option<Hover> {
        let offset = position_to_offset(source, position)?;
        let word = get_word_at_offset(source, offset)?;

        // Check if it's a built-in function
        if let Some(doc) = get_function_documentation(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                }),
                range: None,
            });
        }

        // Check if it's a keyword
        if let Some(doc) = get_keyword_documentation(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                }),
                range: None,
            });
        }

        None
    }

    /// Get completion items at a position
    fn get_completions(&self, source: &str, _position: Position) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Add built-in functions
        items.extend(get_function_completions());

        // Add keywords
        items.extend(get_keyword_completions());

        // Add variables from the document
        items.extend(self.get_variable_completions(source));

        items
    }

    /// Extract variable names from source
    fn get_variable_completions(&self, source: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Simple regex-free extraction of variable assignments
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(eq_pos) = trimmed.find('=') {
                let name = trimmed[..eq_pos].trim();
                // Skip keyword assignments (params, data declarations)
                if !name.contains(':') && !name.contains(' ') && !name.is_empty() {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: Some("Variable".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        items
    }

    /// Get document symbols (outline)
    fn get_symbols(&self, source: &str) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Signal blocks
            if trimmed.starts_with("signal ") {
                if let Some(name) = trimmed
                    .strip_prefix("signal ")
                    .and_then(|s| s.strip_suffix(':'))
                {
                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: name.to_string(),
                        detail: Some("Signal".to_string()),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: i as u32, character: 0 },
                            end: Position { line: i as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: i as u32, character: 7 },
                            end: Position { line: i as u32, character: (7 + name.len()) as u32 },
                        },
                        children: None,
                    });
                }
            }

            // Portfolio blocks
            if trimmed.starts_with("portfolio ") {
                if let Some(name) = trimmed
                    .strip_prefix("portfolio ")
                    .and_then(|s| s.strip_suffix(':'))
                {
                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: name.to_string(),
                        detail: Some("Portfolio".to_string()),
                        kind: SymbolKind::CLASS,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: i as u32, character: 0 },
                            end: Position { line: i as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: i as u32, character: 10 },
                            end: Position { line: i as u32, character: (10 + name.len()) as u32 },
                        },
                        children: None,
                    });
                }
            }

            // User functions
            if trimmed.starts_with("fn ") {
                if let Some(paren_pos) = trimmed.find('(') {
                    let name = &trimmed[3..paren_pos];
                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: name.to_string(),
                        detail: Some("Function".to_string()),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: i as u32, character: 0 },
                            end: Position { line: i as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: i as u32, character: 3 },
                            end: Position { line: i as u32, character: (3 + name.len()) as u32 },
                        },
                        children: None,
                    });
                }
            }
        }

        symbols
    }

    /// Find definition location for a symbol
    fn get_definition(&self, source: &str, position: Position) -> Option<Location> {
        let offset = position_to_offset(source, position)?;
        let word = get_word_at_offset(source, offset)?;

        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check signal definitions: "signal name:"
            if trimmed.starts_with("signal ") {
                if let Some(name) = trimmed
                    .strip_prefix("signal ")
                    .and_then(|s| s.strip_suffix(':'))
                {
                    if name == word {
                        return Some(Location {
                            uri: Url::parse("file:///").unwrap(), // Will be replaced with actual URI
                            range: Range {
                                start: Position { line: i as u32, character: 7 },
                                end: Position { line: i as u32, character: (7 + name.len()) as u32 },
                            },
                        });
                    }
                }
            }

            // Check portfolio definitions: "portfolio name:"
            if trimmed.starts_with("portfolio ") {
                if let Some(name) = trimmed
                    .strip_prefix("portfolio ")
                    .and_then(|s| s.strip_suffix(':'))
                {
                    if name == word {
                        return Some(Location {
                            uri: Url::parse("file:///").unwrap(),
                            range: Range {
                                start: Position { line: i as u32, character: 10 },
                                end: Position { line: i as u32, character: (10 + name.len()) as u32 },
                            },
                        });
                    }
                }
            }

            // Check function definitions: "fn name("
            if trimmed.starts_with("fn ") {
                if let Some(paren_pos) = trimmed.find('(') {
                    let name = &trimmed[3..paren_pos];
                    if name == word {
                        return Some(Location {
                            uri: Url::parse("file:///").unwrap(),
                            range: Range {
                                start: Position { line: i as u32, character: 3 },
                                end: Position { line: i as u32, character: (3 + name.len()) as u32 },
                            },
                        });
                    }
                }
            }

            // Check variable assignments: "name = ..."
            if let Some(eq_pos) = trimmed.find('=') {
                let name = trimmed[..eq_pos].trim();
                // Skip keyword assignments and declarations with colons
                if !name.contains(':') && !name.contains(' ') && !name.is_empty() && name == word {
                    let start_char = line.find(name).unwrap_or(0);
                    return Some(Location {
                        uri: Url::parse("file:///").unwrap(),
                        range: Range {
                            start: Position { line: i as u32, character: start_char as u32 },
                            end: Position { line: i as u32, character: (start_char + name.len()) as u32 },
                        },
                    });
                }
            }
        }

        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SigcServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "sigc-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "sigc language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.documents.write().await.insert(
            uri.clone(),
            Document {
                content: content.clone(),
                version,
            },
        );

        self.analyze(&uri, &content).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            self.documents.write().await.insert(
                uri.clone(),
                Document {
                    content: content.clone(),
                    version,
                },
            );

            self.analyze(&uri, &content).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().await.remove(&params.text_document.uri);
        // Clear diagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        if let Some(doc) = docs.get(uri) {
            return Ok(self.get_hover(&doc.content, position));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.read().await;
        if let Some(doc) = docs.get(uri) {
            let items = self.get_completions(&doc.content, position);
            return Ok(Some(CompletionResponse::Array(items)));
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let docs = self.documents.read().await;
        if let Some(doc) = docs.get(uri) {
            let symbols = self.get_symbols(&doc.content);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        if let Some(doc) = docs.get(uri) {
            if let Some(mut location) = self.get_definition(&doc.content, position) {
                // Replace the placeholder URI with the actual document URI
                location.uri = uri.clone();
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }

        Ok(None)
    }
}

// Helper functions

fn extract_location(message: &str) -> Option<(usize, usize)> {
    // Parse "line X:Y" format
    if let Some(line_pos) = message.find("line ") {
        let rest = &message[line_pos + 5..];
        let parts: Vec<&str> = rest.split(':').take(2).collect();
        if parts.len() >= 2 {
            let line = parts[0].trim().parse().ok()?;
            let col = parts[1].split_whitespace().next()?.parse().ok()?;
            return Some((line, col));
        }
    }
    None
}

fn clean_error_message(message: &str) -> String {
    // Extract just the error description
    if let Some(start) = message.find("error: ") {
        let rest = &message[start + 7..];
        if let Some(end) = rest.find('\n') {
            return rest[..end].to_string();
        }
        return rest.to_string();
    }
    message.to_string()
}

fn position_to_offset(source: &str, position: Position) -> Option<usize> {
    let mut offset = 0;
    for (i, line) in source.lines().enumerate() {
        if i == position.line as usize {
            return Some(offset + position.character as usize);
        }
        offset += line.len() + 1; // +1 for newline
    }
    None
}

fn get_word_at_offset(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    if offset >= bytes.len() {
        return None;
    }

    // Find word boundaries
    let mut start = offset;
    while start > 0 && is_word_char(bytes[start - 1] as char) {
        start -= 1;
    }

    let mut end = offset;
    while end < bytes.len() && is_word_char(bytes[end] as char) {
        end += 1;
    }

    if start < end {
        Some(source[start..end].to_string())
    } else {
        None
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn get_function_documentation(name: &str) -> Option<String> {
    let doc = match name {
        // Time-series functions
        "ret" => "```sigc\nret(x, periods)\n```\n\nCalculate n-period returns.\n\n**Parameters:**\n- `x`: Price series\n- `periods`: Lookback period\n\n**Returns:** Return series",
        "lag" => "```sigc\nlag(x, n)\n```\n\nShift series by n periods.\n\n**Parameters:**\n- `x`: Input series\n- `n`: Number of periods to shift",
        "rolling_mean" => "```sigc\nrolling_mean(x, window)\n```\n\nCalculate rolling mean (moving average).\n\n**Parameters:**\n- `x`: Input series\n- `window`: Window size",
        "rolling_std" => "```sigc\nrolling_std(x, window)\n```\n\nCalculate rolling standard deviation.\n\n**Parameters:**\n- `x`: Input series\n- `window`: Window size",
        "rolling_sum" => "```sigc\nrolling_sum(x, window)\n```\n\nCalculate rolling sum.\n\n**Parameters:**\n- `x`: Input series\n- `window`: Window size",
        "rolling_min" => "```sigc\nrolling_min(x, window)\n```\n\nCalculate rolling minimum.",
        "rolling_max" => "```sigc\nrolling_max(x, window)\n```\n\nCalculate rolling maximum.",
        "ema" => "```sigc\nema(x, span)\n```\n\nExponential moving average.\n\n**Parameters:**\n- `x`: Input series\n- `span`: EMA span (half-life)",
        "ts_rank" => "```sigc\nts_rank(x, window)\n```\n\nTime-series rank within window.",
        "ts_zscore" => "```sigc\nts_zscore(x, window)\n```\n\nTime-series z-score within window.",

        // Cross-sectional functions
        "zscore" => "```sigc\nzscore(x)\n```\n\nCross-sectional z-score normalization.\n\nStandardizes values to have mean 0 and std 1 across the cross-section.",
        "rank" => "```sigc\nrank(x)\n```\n\nCross-sectional percentile rank [0, 1].",
        "demean" => "```sigc\ndemean(x)\n```\n\nSubtract cross-sectional mean.",
        "scale" => "```sigc\nscale(x)\n```\n\nScale to sum to 1 (for portfolio weights).",
        "winsor" => "```sigc\nwinsor(x, p=0.01)\n```\n\nWinsorize at percentile p.\n\n**Parameters:**\n- `x`: Input series\n- `p`: Percentile (default 0.01 = 1%)",
        "neutralize" => "```sigc\nneutralize(x, by=group)\n```\n\nGroup neutralize (demean within groups).\n\n**Parameters:**\n- `x`: Input series\n- `by`: Grouping variable (e.g., sector)",

        // Arithmetic
        "abs" => "```sigc\nabs(x)\n```\n\nAbsolute value.",
        "sqrt" => "```sigc\nsqrt(x)\n```\n\nSquare root.",
        "log" => "```sigc\nlog(x)\n```\n\nNatural logarithm.",
        "exp" => "```sigc\nexp(x)\n```\n\nExponential function.",
        "clip" => "```sigc\nclip(x, lo, hi)\n```\n\nClip values to range [lo, hi].",
        "min" => "```sigc\nmin(a, b)\n```\n\nElement-wise minimum.",
        "max" => "```sigc\nmax(a, b)\n```\n\nElement-wise maximum.",

        // Data handling
        "fill_nan" => "```sigc\nfill_nan(x, value)\n```\n\nReplace NaN values with specified value.",
        "coalesce" => "```sigc\ncoalesce(a, b)\n```\n\nReturn first non-NaN value.",
        "where" => "```sigc\nwhere(condition, if_true, if_false)\n```\n\nConditional expression.",
        "cumsum" => "```sigc\ncumsum(x)\n```\n\nCumulative sum.",

        // Technical indicators
        "rsi" => "```sigc\nrsi(x, period)\n```\n\nRelative Strength Index.\n\n**Parameters:**\n- `x`: Price series\n- `period`: RSI period (typically 14)",
        "macd" => "```sigc\nmacd(x, fast, slow, signal)\n```\n\nMACD indicator.\n\n**Parameters:**\n- `x`: Price series\n- `fast`: Fast EMA period (typically 12)\n- `slow`: Slow EMA period (typically 26)\n- `signal`: Signal line period (typically 9)",

        // Portfolio
        "long_short" => "```sigc\nrank(x).long_short(top=0.2, bottom=0.2)\n```\n\nCreate long-short portfolio weights.\n\n**Parameters:**\n- `top`: Fraction to go long (e.g., 0.2 = top 20%)\n- `bottom`: Fraction to go short",

        _ => return None,
    };
    Some(doc.to_string())
}

fn get_keyword_documentation(name: &str) -> Option<String> {
    let doc = match name {
        "data" => "**data** section\n\nDeclare data sources.\n\n```sigc\ndata:\n  px: load csv from \"data/prices.csv\"\n  sector: load csv from \"data/sectors.csv\"\n```",
        "params" => "**params** section\n\nDeclare tunable parameters.\n\n```sigc\nparams:\n  lookback = 20\n  threshold = 0.5\n```",
        "signal" => "**signal** block\n\nDefine a trading signal.\n\n```sigc\nsignal momentum:\n  r = ret(px, lookback)\n  emit zscore(r)\n```",
        "portfolio" => "**portfolio** block\n\nDefine portfolio construction and backtest.\n\n```sigc\nportfolio main:\n  weights = rank(signal).long_short(top=0.2, bottom=0.2)\n  backtest from 2020-01-01 to 2024-12-31\n```",
        "emit" => "**emit** statement\n\nOutput the signal value.\n\n```sigc\nemit zscore(score)\n```",
        "fn" => "**fn** - User function definition\n\n```sigc\nfn momentum(x, window=20):\n  x.ret(periods=1).rolling_mean(window=window)\n```",
        "load" => "**load** - Load data from file\n\n```sigc\nload csv from \"path/to/file.csv\"\nload parquet from \"s3://bucket/data.parquet\"\n```",
        "backtest" => "**backtest** - Run backtest\n\n```sigc\nbacktest from 2020-01-01 to 2024-12-31\nbacktest rebal=21d benchmark=SPY from 2020-01-01 to 2024-12-31\n```",
        _ => return None,
    };
    Some(doc.to_string())
}

fn get_function_completions() -> Vec<CompletionItem> {
    let functions = [
        // Time-series
        ("ret", "Return calculation", "ret(${1:x}, ${2:periods})"),
        ("lag", "Lag by periods", "lag(${1:x}, ${2:n})"),
        ("rolling_mean", "Rolling mean", "rolling_mean(${1:x}, ${2:window})"),
        ("rolling_std", "Rolling std dev", "rolling_std(${1:x}, ${2:window})"),
        ("rolling_sum", "Rolling sum", "rolling_sum(${1:x}, ${2:window})"),
        ("rolling_min", "Rolling minimum", "rolling_min(${1:x}, ${2:window})"),
        ("rolling_max", "Rolling maximum", "rolling_max(${1:x}, ${2:window})"),
        ("ema", "Exponential MA", "ema(${1:x}, ${2:span})"),
        ("ts_rank", "Time-series rank", "ts_rank(${1:x}, ${2:window})"),
        ("ts_zscore", "Time-series z-score", "ts_zscore(${1:x}, ${2:window})"),
        ("decay_linear", "Linear decay", "decay_linear(${1:x}, ${2:window})"),

        // Cross-sectional
        ("zscore", "Cross-sectional z-score", "zscore(${1:x})"),
        ("rank", "Cross-sectional rank", "rank(${1:x})"),
        ("demean", "Subtract mean", "demean(${1:x})"),
        ("scale", "Scale to sum=1", "scale(${1:x})"),
        ("winsor", "Winsorize", "winsor(${1:x}, p=${2:0.01})"),
        ("neutralize", "Group neutralize", "neutralize(${1:x}, by=${2:sector})"),
        ("quantile", "Quantile value", "quantile(${1:x}, q=${2:0.5})"),
        ("bucket", "Assign to buckets", "bucket(${1:x}, n=${2:5})"),
        ("median", "Cross-sectional median", "median(${1:x})"),

        // Arithmetic
        ("abs", "Absolute value", "abs(${1:x})"),
        ("sqrt", "Square root", "sqrt(${1:x})"),
        ("log", "Natural log", "log(${1:x})"),
        ("exp", "Exponential", "exp(${1:x})"),
        ("pow", "Power", "pow(${1:x}, ${2:n})"),
        ("clip", "Clip to range", "clip(${1:x}, ${2:lo}, ${3:hi})"),
        ("min", "Minimum", "min(${1:a}, ${2:b})"),
        ("max", "Maximum", "max(${1:a}, ${2:b})"),

        // Data handling
        ("fill_nan", "Fill NaN values", "fill_nan(${1:x}, ${2:0})"),
        ("coalesce", "First non-NaN", "coalesce(${1:a}, ${2:b})"),
        ("where", "Conditional", "where(${1:cond}, ${2:if_true}, ${3:if_false})"),
        ("cumsum", "Cumulative sum", "cumsum(${1:x})"),
        ("cumprod", "Cumulative product", "cumprod(${1:x})"),
        ("cummax", "Cumulative max", "cummax(${1:x})"),
        ("cummin", "Cumulative min", "cummin(${1:x})"),

        // Technical
        ("rsi", "RSI indicator", "rsi(${1:px}, ${2:14})"),
        ("macd", "MACD indicator", "macd(${1:px}, ${2:12}, ${3:26}, ${4:9})"),
    ];

    functions
        .iter()
        .map(|(name, detail, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(detail.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn get_keyword_completions() -> Vec<CompletionItem> {
    let keywords = [
        ("data", "Data section"),
        ("params", "Parameters section"),
        ("signal", "Signal block"),
        ("portfolio", "Portfolio block"),
        ("emit", "Emit signal output"),
        ("fn", "Function definition"),
        ("load", "Load data"),
        ("from", "From source"),
        ("backtest", "Run backtest"),
        ("weights", "Portfolio weights"),
        ("csv", "CSV format"),
        ("parquet", "Parquet format"),
    ];

    keywords
        .iter()
        .map(|(name, detail)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| SigcServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
