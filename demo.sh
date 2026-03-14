#!/bin/bash
#
# sigc interactive demo
# https://github.com/skelf-Research/sigc
#
# A colorful, step-by-step demonstration of sigc capabilities
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
BOLD='\033[1m'
DIM='\033[2m'
ITALIC='\033[3m'
NC='\033[0m'

# Box drawing
BOX_TL="╭"
BOX_TR="╮"
BOX_BL="╰"
BOX_BR="╯"
BOX_H="─"
BOX_V="│"

# Symbols
CHECK="${GREEN}✓${NC}"
ARROW="${CYAN}➜${NC}"
STAR="${YELLOW}★${NC}"
ROCKET="${MAGENTA}🚀${NC}"
CHART="${CYAN}📈${NC}"
GEAR="${BLUE}⚙${NC}"
SPARKLE="${YELLOW}✨${NC}"

DEMO_DIR=""
SIGC_CMD=""

# -----------------------------------------------------------------------------
# Helper functions
# -----------------------------------------------------------------------------

print_banner() {
    clear
    echo ""
    echo -e "${CYAN}"
    echo '   ███████╗██╗ ██████╗  ██████╗ '
    echo '   ██╔════╝██║██╔════╝ ██╔════╝ '
    echo '   ███████╗██║██║  ███╗██║      '
    echo '   ╚════██║██║██║   ██║██║      '
    echo '   ███████║██║╚██████╔╝╚██████╗ '
    echo '   ╚══════╝╚═╝ ╚═════╝  ╚═════╝ '
    echo -e "${NC}"
    echo -e "   ${BOLD}The Quant's Compiler${NC}"
    echo -e "   ${DIM}Interactive Demo${NC}"
    echo ""
    echo -e "   ${DIM}────────────────────────────────────${NC}"
    echo ""
}

print_step() {
    local num="$1"
    local title="$2"
    echo ""
    echo -e "${BOX_TL}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_TR}"
    echo -e "${BOX_V}  ${BOLD}${BLUE}STEP ${num}${NC}  ${BOLD}${title}${NC}"
    echo -e "${BOX_BL}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_BR}"
    echo ""
}

print_code() {
    local title="$1"
    local file="$2"

    echo -e "   ${DIM}${title}${NC}"
    echo -e "   ${DIM}┌────────────────────────────────────────────────────┐${NC}"
    while IFS= read -r line; do
        # Syntax highlighting
        line=$(echo "$line" | sed \
            -e "s/\b\(data\|signal\|portfolio\|params\|emit\|backtest\|from\|to\):/\\${MAGENTA}\1:\\${NC}/g" \
            -e "s/\b\(load\|csv\|parquet\)\b/\\${BLUE}\1\\${NC}/g" \
            -e "s/\b\(ret\|zscore\|rank\|winsor\|rolling_mean\|rolling_std\|long_short\|ema\)\b/\\${CYAN}\1\\${NC}/g" \
            -e "s/\"[^\"]*\"/\\${GREEN}&\\${NC}/g" \
            -e "s/\b[0-9.]\+\b/\\${YELLOW}&\\${NC}/g" \
            -e "s|//.*|\\${DIM}&\\${NC}|g")
        echo -e "   ${DIM}│${NC} ${line}"
    done < "$file"
    echo -e "   ${DIM}└────────────────────────────────────────────────────┘${NC}"
    echo ""
}

print_command() {
    local cmd="$1"
    echo -e "   ${ARROW} ${DIM}\$${NC} ${CYAN}${cmd}${NC}"
    echo ""
}

print_result() {
    echo -e "   ${DIM}┌─ Results ─────────────────────────────────────────┐${NC}"
    while IFS= read -r line; do
        # Highlight metrics
        line=$(echo "$line" | sed \
            -e "s/\(Sharpe Ratio\)/\\${GREEN}\1\\${NC}/g" \
            -e "s/\(Total Return\)/\\${CYAN}\1\\${NC}/g" \
            -e "s/\(Max Drawdown\)/\\${YELLOW}\1\\${NC}/g" \
            -e "s/\([0-9.-]\+%\)/\\${BOLD}\1\\${NC}/g")
        echo -e "   ${DIM}│${NC} ${line}"
    done
    echo -e "   ${DIM}└────────────────────────────────────────────────────┘${NC}"
}

pause() {
    echo ""
    echo -e "   ${DIM}Press Enter to continue...${NC}"
    read -r
}

success() {
    echo -e "   ${CHECK} $1"
}

info() {
    echo -e "   ${ARROW} $1"
}

# -----------------------------------------------------------------------------
# Data generation
# -----------------------------------------------------------------------------

generate_sample_data() {
    local output_file="$1"
    local num_days=504  # ~2 years of trading days
    local start_date="2023-01-03"

    # Stock tickers and starting prices
    local tickers=("AAPL" "MSFT" "GOOG" "AMZN" "META" "NVDA" "TSLA" "JPM" "V" "JNJ"
                   "WMT" "PG" "MA" "UNH" "HD" "DIS" "PYPL" "NFLX" "ADBE" "CRM"
                   "INTC" "CSCO" "PEP" "TMO" "ABT" "COST" "AVGO" "ACN" "MRK" "NKE")
    local prices=(175 380 140 170 350 480 250 150 230 160
                  155 150 380 520 320 95 75 450 380 210
                  35 50 175 550 105 520 850 320 110 105)

    # Header
    echo -n "date" > "$output_file"
    for ticker in "${tickers[@]}"; do
        echo -n ",$ticker" >> "$output_file"
    done
    echo "" >> "$output_file"

    # Generate prices with realistic movements
    local current_date
    if [[ "$OSTYPE" == "darwin"* ]]; then
        current_date=$(date -j -f "%Y-%m-%d" "$start_date" "+%s")
    else
        current_date=$(date -d "$start_date" "+%s")
    fi

    for ((day=0; day<num_days; day++)); do
        # Skip weekends
        local dow
        if [[ "$OSTYPE" == "darwin"* ]]; then
            dow=$(date -j -f "%s" "$current_date" "+%u")
        else
            dow=$(date -d "@$current_date" "+%u")
        fi

        if [ "$dow" -le 5 ]; then
            # Format date
            local date_str
            if [[ "$OSTYPE" == "darwin"* ]]; then
                date_str=$(date -j -f "%s" "$current_date" "+%Y-%m-%d")
            else
                date_str=$(date -d "@$current_date" "+%Y-%m-%d")
            fi

            echo -n "$date_str" >> "$output_file"

            # Update each stock price
            for i in "${!tickers[@]}"; do
                # Random daily return: slight upward drift + volatility
                local drift=$(awk "BEGIN {printf \"%.6f\", 0.0003 + (rand()-0.5)*0.0001}")
                local vol=$(awk "BEGIN {printf \"%.6f\", (rand()-0.5)*0.03}")
                local ret=$(awk "BEGIN {printf \"%.6f\", $drift + $vol}")

                prices[$i]=$(awk "BEGIN {printf \"%.2f\", ${prices[$i]} * (1 + $ret)}")
                echo -n ",${prices[$i]}" >> "$output_file"
            done
            echo "" >> "$output_file"
        fi

        # Next day
        current_date=$((current_date + 86400))
    done
}

# -----------------------------------------------------------------------------
# Demo strategies
# -----------------------------------------------------------------------------

create_momentum_strategy() {
    cat > "$DEMO_DIR/momentum.sig" << 'EOF'
// Momentum Strategy
// Classic 20-day price momentum with cross-sectional ranking

data:
  prices: load csv from "prices.csv"

params:
  lookback = 20
  top_pct = 0.2

signal momentum:
  returns = ret(prices, lookback)
  score = zscore(returns)
  emit winsor(score, p=0.01)

portfolio main:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)
  backtest from 2023-01-01 to 2024-12-31
EOF
}

create_meanrev_strategy() {
    cat > "$DEMO_DIR/meanrev.sig" << 'EOF'
// Mean Reversion Strategy
// Buy oversold, sell overbought using Bollinger-style z-scores

data:
  prices: load csv from "prices.csv"

params:
  window = 20
  threshold = 2.0

signal mean_reversion:
  ma = rolling_mean(prices, window)
  std = rolling_std(prices, window)
  z = (prices - ma) / std
  emit -zscore(z)

portfolio main:
  weights = rank(mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2023-01-01 to 2024-12-31
EOF
}

create_multifactor_strategy() {
    cat > "$DEMO_DIR/multifactor.sig" << 'EOF'
// Multi-Factor Strategy
// Combines momentum, mean reversion, and volatility factors

data:
  prices: load csv from "prices.csv"

params:
  mom_lookback = 60
  vol_lookback = 20

signal momentum:
  emit zscore(ret(prices, mom_lookback))

signal low_vol:
  vol = rolling_std(ret(prices, 1), vol_lookback)
  emit -zscore(vol)

signal mean_rev:
  fast = ret(prices, 5)
  slow = ret(prices, 20)
  emit -zscore(fast - slow)

signal combined:
  emit 0.4 * momentum + 0.3 * low_vol + 0.3 * mean_rev

portfolio main:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2023-01-01 to 2024-12-31
EOF
}

# -----------------------------------------------------------------------------
# Main demo flow
# -----------------------------------------------------------------------------

main() {
    print_banner

    echo -e "   ${SPARKLE} Welcome to the ${BOLD}sigc${NC} interactive demo!"
    echo ""
    echo -e "   This demo will walk you through:"
    echo -e "   ${ARROW} Creating trading strategies with the sigc DSL"
    echo -e "   ${ARROW} Running backtests on sample data"
    echo -e "   ${ARROW} Analyzing performance metrics"
    echo ""

    pause

    # -------------------------------------------------------------------------
    # Step 1: Check sigc installation
    # -------------------------------------------------------------------------
    print_step "1/5" "Checking Installation"

    if command -v sigc &> /dev/null; then
        SIGC_CMD="sigc"
    elif [ -x "./target/release/sigc" ]; then
        SIGC_CMD="./target/release/sigc"
    elif [ -x "$HOME/.local/bin/sigc" ]; then
        SIGC_CMD="$HOME/.local/bin/sigc"
    else
        echo -e "   ${RED}✗${NC} sigc not found!"
        echo ""
        echo -e "   ${ARROW} Install with: ${CYAN}./install.sh${NC}"
        echo -e "   ${ARROW} Or build:     ${CYAN}cargo build --release${NC}"
        exit 1
    fi

    local version=$($SIGC_CMD --version 2>/dev/null | head -1 || echo "sigc")
    success "Found: ${BOLD}${version}${NC}"

    pause

    # -------------------------------------------------------------------------
    # Step 2: Create demo workspace
    # -------------------------------------------------------------------------
    print_step "2/5" "Creating Demo Workspace"

    DEMO_DIR=$(mktemp -d)
    info "Creating temporary workspace: ${DIM}${DEMO_DIR}${NC}"
    echo ""

    info "Generating sample market data (30 stocks, 2 years)..."
    echo -e "   ${DIM}This simulates realistic price movements...${NC}"
    echo ""

    generate_sample_data "$DEMO_DIR/prices.csv"

    local num_rows=$(wc -l < "$DEMO_DIR/prices.csv")
    local num_cols=$(head -1 "$DEMO_DIR/prices.csv" | tr ',' '\n' | wc -l)

    success "Generated ${BOLD}${num_rows}${NC} rows × ${BOLD}${num_cols}${NC} columns"

    # Show sample of data
    echo ""
    echo -e "   ${DIM}Sample data (first 5 rows, 6 columns):${NC}"
    echo -e "   ${DIM}┌────────────────────────────────────────────────────┐${NC}"
    head -6 "$DEMO_DIR/prices.csv" | cut -d',' -f1-6 | while IFS= read -r line; do
        echo -e "   ${DIM}│${NC} ${line}"
    done
    echo -e "   ${DIM}└────────────────────────────────────────────────────┘${NC}"

    pause

    # -------------------------------------------------------------------------
    # Step 3: Momentum Strategy
    # -------------------------------------------------------------------------
    print_step "3/5" "Demo 1: Momentum Strategy ${ROCKET}"

    echo -e "   ${CHART} ${BOLD}Price Momentum${NC}"
    echo -e "   ${DIM}Buy recent winners, sell recent losers${NC}"
    echo ""

    create_momentum_strategy
    print_code "momentum.sig" "$DEMO_DIR/momentum.sig"

    print_command "$SIGC_CMD run momentum.sig"

    cd "$DEMO_DIR"
    $SIGC_CMD run momentum.sig 2>&1 | print_result

    pause

    # -------------------------------------------------------------------------
    # Step 4: Mean Reversion Strategy
    # -------------------------------------------------------------------------
    print_step "4/5" "Demo 2: Mean Reversion Strategy ${CHART}"

    echo -e "   ${CHART} ${BOLD}Statistical Mean Reversion${NC}"
    echo -e "   ${DIM}Buy oversold stocks, sell overbought stocks${NC}"
    echo ""

    create_meanrev_strategy
    print_code "meanrev.sig" "$DEMO_DIR/meanrev.sig"

    print_command "$SIGC_CMD run meanrev.sig"

    $SIGC_CMD run meanrev.sig 2>&1 | print_result

    pause

    # -------------------------------------------------------------------------
    # Step 5: Multi-Factor Strategy
    # -------------------------------------------------------------------------
    print_step "5/5" "Demo 3: Multi-Factor Strategy ${STAR}"

    echo -e "   ${STAR} ${BOLD}Combined Factor Model${NC}"
    echo -e "   ${DIM}Blend momentum + low volatility + mean reversion${NC}"
    echo ""

    create_multifactor_strategy
    print_code "multifactor.sig" "$DEMO_DIR/multifactor.sig"

    print_command "$SIGC_CMD run multifactor.sig"

    $SIGC_CMD run multifactor.sig 2>&1 | print_result

    echo ""

    # -------------------------------------------------------------------------
    # Summary
    # -------------------------------------------------------------------------
    echo ""
    echo -e "${GREEN}${BOLD}"
    echo "   ╔═══════════════════════════════════════════════════════╗"
    echo "   ║                                                       ║"
    echo "   ║              ${WHITE}Demo Complete! ${SPARKLE}${GREEN}${BOLD}                       ║"
    echo "   ║                                                       ║"
    echo "   ╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    echo -e "   ${BOLD}What you learned:${NC}"
    echo ""
    echo -e "   ${CHECK} Write trading signals with the sigc DSL"
    echo -e "   ${CHECK} Define parameters for strategy tuning"
    echo -e "   ${CHECK} Build portfolios with ranking and position sizing"
    echo -e "   ${CHECK} Run backtests and analyze performance metrics"
    echo ""

    echo -e "   ${BOLD}Next steps:${NC}"
    echo ""
    echo -e "   ${ARROW} Explore strategies:  ${CYAN}ls strategies/${NC}"
    echo -e "   ${ARROW} Read the docs:       ${CYAN}https://docs.skelfresearch.com/sigc${NC}"
    echo -e "   ${ARROW} Try your own data:   ${CYAN}sigc run your_strategy.sig${NC}"
    echo ""

    # Cleanup prompt
    echo -e "   ${DIM}────────────────────────────────────────────────────${NC}"
    echo ""
    read -p "   Keep demo files in $DEMO_DIR? [y/N] " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        info "Demo files saved to: ${CYAN}${DEMO_DIR}${NC}"
    else
        rm -rf "$DEMO_DIR"
        info "Demo files cleaned up"
    fi

    echo ""
    echo -e "   ${SPARKLE} Thanks for trying ${BOLD}sigc${NC}!"
    echo ""
}

main "$@"
