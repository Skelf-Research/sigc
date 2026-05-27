#!/usr/bin/env bash
# Fetch a fixed, reproducible universe of daily bars into data/prices.parquet
# using `sigc fetch` (free Yahoo Finance data, no API key). Re-running with the
# same args and date range reproduces the same panel.
#
# Usage: scripts/fetch_data.sh
set -euo pipefail
cd "$(dirname "$0")/.."

# A fixed 10-name large-cap universe (plain tickers for Yahoo). Extend as needed.
SYMBOLS="${SYMBOLS:-AAPL,MSFT,GOOG,AMZN,META,JPM,XOM,JNJ,PG,WMT}"
SOURCE="${SOURCE:-yahoo}"
START="${START:-20150101}"
END="${END:-20241231}"
OUT="${OUT:-data/prices.parquet}"

echo "Fetching ${SYMBOLS} from ${SOURCE} (${START}..${END}) -> ${OUT}"
cargo run --release -p sigc -- fetch \
  --source "$SOURCE" --symbols "$SYMBOLS" --start "$START" --end "$END" --out "$OUT"

# --- Ken French factors (manual; needs `unzip`) -----------------------------
# The factor files ship as zips, so fetch+unzip here, then parse in Rust via
# sig_runtime::parse_ken_french (already unit-tested):
#
#   base="https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp"
#   curl -sL "$base/F-F_Research_Data_Factors_daily_CSV.zip" -o /tmp/ff.zip
#   unzip -p /tmp/ff.zip > data/ff_factors.csv
echo "Done. (Ken French factors: see commented steps in this script.)"
