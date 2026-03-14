# Changelog

All notable changes to sigc are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2026-01-15

### Added

- **Type System Enhancements**
    - Type inference for all expressions
    - Operator signatures with input/output type requirements
    - Shape-aware operations with semantic categories
    - Comprehensive error messages with suggestions

- **Macro System**
    - 8 built-in macros: `momentum`, `mean_reversion`, `vol_adj_momentum`, `trend`, `rsi_signal`, `breakout`, `cs_momentum`, `quality`
    - User-defined macros with typed parameters
    - `let` statements for intermediate computations
    - `emit` statements for macro output

- **External Integrations**
    - Yahoo Finance data provider
    - Alpaca broker integration (paper and live trading)
    - WebSocket streaming client

### Changed

- Improved error messages with source locations and suggestions
- Enhanced LSP with operator hover documentation

### Fixed

- Fixed memory leak in long-running daemon mode
- Fixed corporate action adjustment for dividends

## [0.9.0] - 2026-01-01

### Added

- **Language Enhancements**
    - Custom functions with `fn` keyword
    - Default parameter values
    - Function composition and chaining

- **VS Code Extension**
    - Language Server Protocol (LSP) support
    - Real-time error diagnostics
    - Hover documentation for 50+ operators
    - Code completion with snippets
    - Go-to-definition for signals, functions, macros
    - Document outline

### Changed

- Refactored compiler for better error recovery
- Improved parser performance by 40%

## [0.8.0] - 2025-12-15

### Added

- **Deployment & Safety**
    - Circuit breakers with configurable thresholds
    - Position limit enforcement
    - Order validation and rejection
    - Global kill switch
    - Rate limiting

- **Docker Support**
    - Dockerfile for containerized deployment
    - docker-compose.yml for production setup
    - Health check endpoints

### Changed

- Safety checks are now enabled by default
- Improved daemon stability

## [0.7.0] - 2025-12-01

### Added

- **Advanced Analytics**
    - Fama-French factor models
    - Barra risk models
    - VaR, CVaR, and stress testing
    - Hidden Markov Model regime detection
    - Clustering-based regime identification
    - Mean-variance optimization
    - Risk parity portfolio construction
    - Black-Litterman model
    - Hierarchical Risk Parity (HRP)

### Changed

- Portfolio optimization now uses CVXPY backend
- Improved risk model estimation

## [0.6.0] - 2025-11-15

### Added

- **Education & Documentation**
    - 11-chapter Quant Guide
    - 23 example strategies across 6 categories
    - Comprehensive operator documentation
    - Tutorial series

### Fixed

- Fixed walk-forward optimization window alignment
- Fixed benchmark-relative metrics calculation

## [0.5.0] - 2025-11-01

### Added

- **Integration & Polish**
    - TOML-based configuration
    - Colored CLI output
    - Test suite with 330 tests
    - Performance benchmarks

### Changed

- CLI now uses clap v4 for argument parsing
- Improved error formatting

## [0.4.0] - 2025-10-15

### Added

- **Performance & Scale**
    - SIMD-optimized kernels for rolling statistics
    - Memory-mapped data loading
    - Incremental computation for streaming
    - Parallel execution with Rayon

### Changed

- Default to SIMD kernels when available
- Reduced memory usage by 60% for large datasets

## [0.3.0] - 2025-10-01

### Added

- **Production Features**
    - Async PostgreSQL connector with connection pooling
    - Corporate action adjustments (splits, dividends, mergers)
    - Alert system (console, Slack, email)
    - Job scheduling (cron-based)
    - Audit logging for compliance

### Changed

- Database connector now uses sqlx for async operations
- Alert routing based on severity

## [0.2.0] - 2025-09-15

### Added

- **Backtesting Features**
    - Transaction cost models (linear, square-root, Almgren-Chriss)
    - Walk-forward optimization
    - Position constraints (max weight, sector, turnover)
    - Benchmark-relative analysis (alpha, beta, information ratio)
    - Brinson attribution

### Changed

- Backtest now includes all metrics by default
- Improved cost model accuracy

## [0.1.0] - 2025-09-01

### Added

- Initial release
- DSL compiler with type checking
- Runtime with Polars/Arrow backend
- Basic backtester with Sharpe, drawdown, turnover
- CLI with compile, run, explain, diff commands
- Daemon mode with nng RPC
- Content-addressed caching with sled + blake3
- Data loading from CSV, Parquet, S3
- 50+ operators (time-series, cross-sectional, technical)

---

## Version History Summary

| Version | Date | Highlights |
|---------|------|------------|
| 0.10.0 | 2026-01-15 | Type system, macros, Yahoo/Alpaca integrations |
| 0.9.0 | 2026-01-01 | Custom functions, VS Code LSP |
| 0.8.0 | 2025-12-15 | Safety systems, Docker |
| 0.7.0 | 2025-12-01 | Advanced analytics, portfolio optimization |
| 0.6.0 | 2025-11-15 | Quant Guide, 23 strategies |
| 0.5.0 | 2025-11-01 | Polish, 330 tests |
| 0.4.0 | 2025-10-15 | SIMD, memory mapping |
| 0.3.0 | 2025-10-01 | Production features, alerts |
| 0.2.0 | 2025-09-15 | Cost models, constraints |
| 0.1.0 | 2025-09-01 | Initial release |
