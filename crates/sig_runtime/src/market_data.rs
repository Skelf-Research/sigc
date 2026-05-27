//! Real market-data ingestion: free, reproducible public sources.
//!
//! Backtests need real data to make a credible claim; the bundled toy prices
//! do not. This module fetches daily bars from [Stooq](https://stooq.com) and
//! parses [Ken French's data library](https://mba.tuck.dartmouth.edu) factor
//! files, both freely redistributable, into Polars frames that the runtime
//! consumes.
//!
//! The design separates **pure parsing** (unit-tested offline) from **network
//! fetching** (thin wrappers around `reqwest::blocking`), so the data-shaping
//! logic is verifiable without a network connection.

use polars::prelude::*;
use sig_types::{Result, SigcError};

/// Daily close bars for a single symbol — the pure intermediate produced by
/// parsing, before assembling a wide price panel.
#[derive(Debug, Clone, PartialEq)]
pub struct Bars {
    pub symbol: String,
    pub dates: Vec<String>,
    pub close: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Pure parsing (offline-testable)
// ---------------------------------------------------------------------------

/// Parse a Stooq daily CSV (`Date,Open,High,Low,Close,Volume`) into close bars.
/// Unknown column orderings are handled by reading the header; malformed rows
/// are skipped.
pub fn parse_stooq_csv(csv: &str, symbol: &str) -> Result<Bars> {
    let mut lines = csv.lines();
    let header = lines
        .next()
        .ok_or_else(|| SigcError::Runtime("empty Stooq CSV".into()))?;
    let cols: Vec<&str> = header.split(',').map(|c| c.trim()).collect();
    let date_idx = cols
        .iter()
        .position(|c| c.eq_ignore_ascii_case("date"))
        .ok_or_else(|| SigcError::Runtime("Stooq CSV missing Date column".into()))?;
    let close_idx = cols
        .iter()
        .position(|c| c.eq_ignore_ascii_case("close"))
        .ok_or_else(|| SigcError::Runtime("Stooq CSV missing Close column".into()))?;

    let mut dates = Vec::new();
    let mut close = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= date_idx.max(close_idx) {
            continue;
        }
        let (Some(d), Ok(c)) = (
            fields.get(date_idx).map(|s| s.trim().to_string()),
            fields[close_idx].trim().parse::<f64>(),
        ) else {
            continue;
        };
        if d.is_empty() {
            continue;
        }
        dates.push(d);
        close.push(c);
    }
    if dates.is_empty() {
        return Err(SigcError::Runtime(format!(
            "no parseable rows in Stooq CSV for {symbol}"
        )));
    }
    Ok(Bars { symbol: symbol.to_string(), dates, close })
}

/// Assemble per-symbol bars into a wide price panel: a `date` column followed
/// by one close-price column per symbol, aligned on the sorted union of dates
/// (missing observations become NaN). Date-sorted ascending.
pub fn build_price_panel(bars: &[Bars]) -> Result<DataFrame> {
    if bars.is_empty() {
        return Err(SigcError::Runtime("no symbols to build a panel from".into()));
    }
    // Sorted union of all dates (lexicographic == chronological for ISO dates).
    let mut all_dates: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for b in bars {
        all_dates.extend(b.dates.iter().cloned());
    }
    let dates: Vec<String> = all_dates.into_iter().collect();
    let index: std::collections::HashMap<&str, usize> =
        dates.iter().enumerate().map(|(i, d)| (d.as_str(), i)).collect();

    let mut columns: Vec<Column> = Vec::with_capacity(bars.len() + 1);
    columns.push(Column::new("date".into(), dates.clone()));

    for b in bars {
        let mut col = vec![f64::NAN; dates.len()];
        for (d, &c) in b.dates.iter().zip(b.close.iter()) {
            if let Some(&i) = index.get(d.as_str()) {
                col[i] = c;
            }
        }
        columns.push(Column::new(b.symbol.as_str().into(), col));
    }

    DataFrame::new(columns)
        .map_err(|e| SigcError::Runtime(format!("failed to build price panel: {e}")))
}

/// Parse a Ken French factor CSV (e.g. `F-F_Research_Data_Factors_daily`).
/// These files carry a free-text preamble, then a header row beginning with an
/// empty cell followed by factor names (`Mkt-RF,SMB,HML,RF`), then `YYYYMMDD`
/// data rows, then trailing annual blocks. We read the first daily block.
pub fn parse_ken_french(csv: &str) -> Result<DataFrame> {
    let mut lines = csv.lines().peekable();

    // Find the header row: contains the factor names, first cell empty.
    let mut factor_names: Vec<String> = Vec::new();
    while let Some(line) = lines.next() {
        if line.contains("Mkt-RF") || line.to_ascii_uppercase().contains("MKT-RF") {
            factor_names = line
                .split(',')
                .skip(1)
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
            break;
        }
    }
    if factor_names.is_empty() {
        return Err(SigcError::Runtime("Ken French: factor header not found".into()));
    }

    let mut dates: Vec<String> = Vec::new();
    let mut cols: Vec<Vec<f64>> = vec![Vec::new(); factor_names.len()];
    for line in lines {
        let fields: Vec<&str> = line.split(',').map(|c| c.trim()).collect();
        let date_tok = fields[0];
        // A daily data row starts with an 8-digit date; stop at the first row
        // that doesn't (blank line or the "Annual Factors" section).
        if date_tok.len() != 8 || !date_tok.bytes().all(|b| b.is_ascii_digit()) {
            if dates.is_empty() {
                continue; // tolerate blank lines between header and data
            }
            break;
        }
        if fields.len() < factor_names.len() + 1 {
            continue;
        }
        dates.push(date_tok.to_string());
        for (j, _) in factor_names.iter().enumerate() {
            cols[j].push(fields[j + 1].parse::<f64>().unwrap_or(f64::NAN));
        }
    }
    if dates.is_empty() {
        return Err(SigcError::Runtime("Ken French: no daily rows parsed".into()));
    }

    let mut columns: Vec<Column> = Vec::with_capacity(factor_names.len() + 1);
    columns.push(Column::new("date".into(), dates));
    for (name, data) in factor_names.iter().zip(cols.into_iter()) {
        columns.push(Column::new(name.as_str().into(), data));
    }
    DataFrame::new(columns)
        .map_err(|e| SigcError::Runtime(format!("failed to build factor frame: {e}")))
}

/// Convert a `YYYYMMDD` string to a UTC Unix timestamp (seconds), for the
/// Yahoo chart endpoint's `period1`/`period2` parameters.
pub fn yyyymmdd_to_unix(s: &str) -> Result<i64> {
    let d = chrono::NaiveDate::parse_from_str(s.trim(), "%Y%m%d")
        .map_err(|e| SigcError::Runtime(format!("bad date '{s}' (want YYYYMMDD): {e}")))?;
    Ok(d.and_hms_opt(0, 0, 0)
        .expect("midnight is valid")
        .and_utc()
        .timestamp())
}

/// Parse Yahoo Finance's v8 chart JSON into close bars. The payload nests the
/// series under `chart.result[0]` with parallel `timestamp` and
/// `indicators.quote[0].close` arrays; null closes (non-trading days) are
/// skipped.
pub fn parse_yahoo_chart_json(json: &str, symbol: &str) -> Result<Bars> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| SigcError::Runtime(format!("Yahoo JSON parse for {symbol}: {e}")))?;
    let result = &v["chart"]["result"][0];
    if result.is_null() {
        return Err(SigcError::Runtime(format!(
            "Yahoo returned no data for {symbol}: {}",
            v["chart"]["error"]
        )));
    }
    let timestamps = result["timestamp"]
        .as_array()
        .ok_or_else(|| SigcError::Runtime(format!("Yahoo: no timestamps for {symbol}")))?;
    let closes = result["indicators"]["quote"][0]["close"]
        .as_array()
        .ok_or_else(|| SigcError::Runtime(format!("Yahoo: no close series for {symbol}")))?;

    let mut dates = Vec::new();
    let mut close = Vec::new();
    for (ts, c) in timestamps.iter().zip(closes.iter()) {
        let (Some(ts), Some(cv)) = (ts.as_i64(), c.as_f64()) else {
            continue; // skip nulls / non-numeric
        };
        let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) else {
            continue;
        };
        dates.push(dt.format("%Y-%m-%d").to_string());
        close.push(cv);
    }
    if dates.is_empty() {
        return Err(SigcError::Runtime(format!(
            "Yahoo: no valid close rows for {symbol}"
        )));
    }
    Ok(Bars { symbol: symbol.to_string(), dates, close })
}

/// Write a frame to parquet (used by `sigc fetch` to pin the data artifact).
pub fn write_parquet(df: &mut DataFrame, path: &str) -> Result<()> {
    let file = std::fs::File::create(path)
        .map_err(|e| SigcError::Runtime(format!("create {path}: {e}")))?;
    ParquetWriter::new(file)
        .finish(df)
        .map_err(|e| SigcError::Runtime(format!("write parquet {path}: {e}")))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Network fetching (thin; verified by integration, not unit tests)
// ---------------------------------------------------------------------------

/// Blocking HTTP GET (with a browser-like UA, which Yahoo requires) returning
/// the body as text.
fn fetch_url(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; sigc/0.1; +https://github.com/skelf-Research/sigc)")
        .build()
        .map_err(|e| SigcError::Runtime(format!("http client: {e}")))?;
    let resp = client
        .get(url)
        .send()
        .map_err(|e| SigcError::Runtime(format!("GET {url}: {e}")))?;
    if !resp.status().is_success() {
        return Err(SigcError::Runtime(format!("GET {url}: HTTP {}", resp.status())));
    }
    resp.text()
        .map_err(|e| SigcError::Runtime(format!("reading {url}: {e}")))
}

/// Fetch one symbol's daily bars from Yahoo Finance (v8 chart endpoint, no API
/// key). `start`/`end` are `YYYYMMDD`; symbols are plain tickers (e.g. `AAPL`).
pub fn fetch_yahoo(symbol: &str, start: &str, end: &str) -> Result<Bars> {
    let p1 = yyyymmdd_to_unix(start)?;
    let p2 = yyyymmdd_to_unix(end)? + 86_400; // make the end day inclusive
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}\
         ?period1={p1}&period2={p2}&interval=1d"
    );
    let json = fetch_url(&url)?;
    parse_yahoo_chart_json(&json, symbol)
}

/// Fetch one symbol's daily bars from Stooq. Stooq now gates anonymous CSV
/// downloads behind a (captcha-issued) API key; pass it via `apikey`.
/// `start`/`end` are `YYYYMMDD`; symbols use the Stooq suffix (e.g. `aapl.us`).
pub fn fetch_stooq(symbol: &str, start: &str, end: &str, apikey: Option<&str>) -> Result<Bars> {
    let mut url = format!("https://stooq.com/q/d/l/?s={symbol}&d1={start}&d2={end}&i=d");
    if let Some(k) = apikey {
        url.push_str(&format!("&apikey={k}"));
    }
    let csv = fetch_url(&url)?;
    parse_stooq_csv(&csv, symbol)
}

/// Build a wide price panel by fetching each symbol with `fetch`.
fn fetch_panel_with<F>(symbols: &[String], fetch: F) -> Result<DataFrame>
where
    F: Fn(&str) -> Result<Bars>,
{
    let mut bars = Vec::with_capacity(symbols.len());
    for s in symbols {
        tracing::info!("fetching {s}");
        bars.push(fetch(s)?);
    }
    build_price_panel(&bars)
}

/// Fetch a multi-symbol daily price panel from Yahoo Finance.
pub fn fetch_yahoo_panel(symbols: &[String], start: &str, end: &str) -> Result<DataFrame> {
    fetch_panel_with(symbols, |s| fetch_yahoo(s, start, end))
}

/// Fetch a multi-symbol daily price panel from Stooq (requires `apikey`).
pub fn fetch_stooq_panel(
    symbols: &[String],
    start: &str,
    end: &str,
    apikey: Option<&str>,
) -> Result<DataFrame> {
    fetch_panel_with(symbols, |s| fetch_stooq(s, start, end, apikey))
}

#[cfg(test)]
mod tests {
    use super::*;

    const STOOQ: &str = "Date,Open,High,Low,Close,Volume\n\
        2024-01-02,100.0,101.0,99.5,100.5,1000\n\
        2024-01-03,100.5,102.0,100.0,101.8,1200\n\
        \n\
        2024-01-04,101.8,103.0,101.0,102.9,1100\n";

    #[test]
    fn parse_stooq_reads_close_by_header() {
        let bars = parse_stooq_csv(STOOQ, "aapl.us").unwrap();
        assert_eq!(bars.symbol, "aapl.us");
        assert_eq!(bars.dates, vec!["2024-01-02", "2024-01-03", "2024-01-04"]);
        assert_eq!(bars.close, vec![100.5, 101.8, 102.9]);
    }

    #[test]
    fn parse_stooq_handles_reordered_columns() {
        let csv = "Close,Date\n10.0,2024-01-02\n11.0,2024-01-03\n";
        let bars = parse_stooq_csv(csv, "x").unwrap();
        assert_eq!(bars.close, vec![10.0, 11.0]);
        assert_eq!(bars.dates, vec!["2024-01-02", "2024-01-03"]);
    }

    #[test]
    fn build_panel_aligns_and_fills_gaps() {
        let a = Bars { symbol: "a".into(), dates: vec!["d1".into(), "d2".into()], close: vec![1.0, 2.0] };
        let b = Bars { symbol: "b".into(), dates: vec!["d2".into(), "d3".into()], close: vec![20.0, 30.0] };
        let df = build_price_panel(&[a, b]).unwrap();
        assert_eq!(df.height(), 3); // union {d1,d2,d3}
        assert_eq!(df.get_column_names(), &["date", "a", "b"]);

        let a_col: Vec<f64> = df.column("a").unwrap().f64().unwrap().into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
        let b_col: Vec<f64> = df.column("b").unwrap().f64().unwrap().into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
        assert_eq!(a_col[0], 1.0);
        assert_eq!(a_col[1], 2.0);
        assert!(a_col[2].is_nan(), "a has no d3 -> NaN");
        assert!(b_col[0].is_nan(), "b has no d1 -> NaN");
        assert_eq!(b_col[1], 20.0);
        assert_eq!(b_col[2], 30.0);
    }

    #[test]
    fn yyyymmdd_to_unix_roundtrips() {
        assert_eq!(yyyymmdd_to_unix("19700101").unwrap(), 0);
        let ts = yyyymmdd_to_unix("20240102").unwrap();
        let s = chrono::DateTime::from_timestamp(ts, 0).unwrap().format("%Y-%m-%d").to_string();
        assert_eq!(s, "2024-01-02");
        assert!(yyyymmdd_to_unix("nonsense").is_err());
    }

    #[test]
    fn parse_yahoo_chart_skips_null_closes() {
        // 1704153600 = 2024-01-02 UTC; second close is null (holiday/missing).
        let json = r#"{"chart":{"result":[{"timestamp":[1704153600,1704240000],
            "indicators":{"quote":[{"close":[185.64,null]}]}}],"error":null}}"#;
        let bars = parse_yahoo_chart_json(json, "AAPL").unwrap();
        assert_eq!(bars.symbol, "AAPL");
        assert_eq!(bars.close, vec![185.64]); // null dropped
        assert_eq!(bars.dates, vec!["2024-01-02"]);
    }

    #[test]
    fn parse_yahoo_errors_on_empty_result() {
        let json = r#"{"chart":{"result":[],"error":{"code":"Not Found"}}}"#;
        assert!(parse_yahoo_chart_json(json, "BOGUS").is_err());
    }

    #[test]
    fn parse_ken_french_reads_daily_block() {
        let csv = "This file was created using ...\n\
            \n\
            ,Mkt-RF,SMB,HML,RF\n\
            19260701, 0.10,-0.24,-0.28, 0.009\n\
            19260702, 0.45,-0.32,-0.08, 0.009\n\
            \n\
            Annual Factors: January-December\n\
            1926, 0.10, 0.20, 0.30, 0.40\n";
        let df = parse_ken_french(csv).unwrap();
        assert_eq!(df.height(), 2, "should read only the 2 daily rows");
        assert_eq!(df.get_column_names(), &["date", "Mkt-RF", "SMB", "HML", "RF"]);
        let mktrf: Vec<f64> = df.column("Mkt-RF").unwrap().f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(mktrf, vec![0.10, 0.45]);
    }
}
