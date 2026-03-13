# Build Roadmap

This document captures the near-term execution plan for bringing `sigc` from architectural spec to a single self-contained binary that ships the compiler, runtime, and developer tooling.

## 0. Guiding goals

- **Single binary story**: `sigc` should compile into one executable that exposes subcommands (`compile`, `run`, `daemon`, `explain`, etc.). No separate daemons need to be deployed during the first milestones.
- **Deterministic research loop**: every run produces reproducible artifacts (plans, reports, graphs) stored in sled using blake3 keys.
- **Rust-first ergonomics**: keep the critical path in Rust (IR, runtime kernels, caching, adapters) while leaving room for PyO3 and RPC adapters later.
- **Python-like DSL experience**: the surface language is ergonomic but compiles to a typed IR that guarantees shape/dtype safety.

## Phase 1 — Foundations ✅ COMPLETE

| Deliverable | Status | Notes |
| ----------- | ------ | ----- |
| Workspace layout | ✅ | Cargo workspace with crates: `sig_compiler`, `sig_runtime`, `sig_cache`, `sigc` (binary), `pysigc`, plus shared `sig_types`. |
| Core types crate | ✅ | Shapes, dtypes, type annotations, 60+ operators, `BacktestPlan`/`BacktestReport` with `rkyv`. |
| Hashing + caching module | ✅ | sled + blake3, IR serialization/deserialization with `put_ir()`/`get_ir()`. |
| Connector abstraction | ✅ | Traits for `DataSource`, `CalendarProvider`, `SecretsResolver`. S3, CSV, Parquet loaders implemented. |
| Config + logging baseline | ✅ | `tracing` setup, CLI config, feature flags. |

**Exit criteria**: ✅ ACHIEVED

## Phase 2 — Compiler path ✅ COMPLETE

| Deliverable | Status | Notes |
| ----------- | ------ | ----- |
| DSL parser | ✅ | `chumsky` parser with indent-awareness, comments support, loads, params, signal blocks, expressions. |
| Type inference | ✅ | Shape/dtype propagation, rich error messages with line/column numbers and suggestions. |
| Stdlib surface | ✅ | 60+ operators: arithmetic, time-series, cross-sectional, logical, comparison, data handling, technical indicators. |
| IR lowering | ✅ | AST to IR with node/type tables, metadata, rkyv serialization, automatic caching by source hash. |
| CLI `compile` | ✅ | `sigc compile input.sig` validates and caches IR in sled. |

**Exit criteria**: ✅ ACHIEVED - `sigc compile examples/momentum.sig` succeeds and caches IR; repeated runs hit cache.

## Phase 3 — Runtime + Backtester ✅ COMPLETE

| Deliverable | Status | Notes |
| ----------- | ------ | ----- |
| Data ingestion | ✅ | Arrow/Polars loading for parquet/csv with DataLoader, DataManager, DateRange. |
| Connector adapters | ✅ | S3, local FS, CSV, Parquet loaders implemented. SQL connectors pending. |
| Kernel library | ✅ | 60+ kernels including `ret`, `lag`, `zscore`, `rank`, `winsor`, `neutralize`, `long_short`, RSI, MACD, ATR, VWAP. |
| Execution engine | ✅ | IR graph traversal, kernel execution, intermediate caching. |
| Builtin backtester | ✅ | Long/short construction, metrics (Sharpe, return, drawdown, turnover). |
| Reporting hooks | ✅ | `BacktestReport` with metrics, provenance metadata. |
| CLI `run` | ✅ | `sigc run input.sig` compiles and executes, stores artifacts. |
| Panel data | ✅ | `Panel` struct for time × assets with parallel cross-sectional operations. |
| Grid search | ✅ | Parameter optimization with `GridSearch`, sorted by Sharpe/return/drawdown. |
| PyO3 bindings | ✅ | `pysigc` crate with `compile()` and `backtest()` functions. |

**Exit criteria**: ✅ ACHIEVED - Examples run end-to-end, artifacts persisted, 23 integration tests pass.

## Phase 4 — Services + Adapters 🔄 IN PROGRESS

| Deliverable | Status | Notes |
| ----------- | ------ | ----- |
| Embedded daemon mode | ✅ | `sigc daemon` starts REQ/REP loop with nng. |
| CLI client subcommands | ⏳ | `sigc daemon`, basic commands done. `explain`, `diff` pending. |
| PyO3 adapter | ✅ | `pysigc` crate with `compile()`, `backtest()`, `CompiledSignal`, `BacktestResult`. |
| C ABI + RPC adapters | ⏳ | Stubs pending for C++/Lean integrations. |
| Rhai hooks skeleton | ⏳ | Plugin system pending. |
| Observability | ⏳ | Structured logs done. Prometheus/audit log pending. |

**Exit criteria**: Partial - daemon and PyO3 done, other adapters pending.

## Phase 5 — Quality bar + Distribution ⏳ PENDING

| Deliverable | Status | Notes |
| ----------- | ------ | ----- |
| Integration tests | ⏳ | Golden end-to-end runs, cache hit/miss scenarios, deterministic outputs. |
| Benchmark harness | ⏳ | Measure kernel performance vs pure Polars baseline. |
| Reporting/attribution | ⏳ | Factor & sector attribution, P&L breakdown exports (CSV/Parquet/HTML), rolling dashboards ready for BI embedding. |
| Packaging | ⏳ | Release profiles, cross-compilation targets, GitHub Actions for CI + artifact upload. |
| Documentation pass | ⏳ | User manual covering language/reference, data connector guides, governance and contribution docs. |

**Exit criteria**: Ready for an 0.1.0 release with reproducible builds, docs, and basic community guidelines.

## Near-term Priorities

| Priority | Deliverable | Description |
| -------- | ----------- | ----------- |
| 1 | Walk-forward optimization | Rolling window backtests with train/test splits for robustness testing and overfitting detection. |
| 2 | Universe management | Stock universe definitions, membership filtering, index constituents, sector/industry mappings. |
| 3 | Transaction cost models | Slippage estimation, market impact (linear/sqrt), commissions, borrowing costs for shorts. |
| 4 | Data connectors | SQL databases (Postgres, Snowflake), additional cloud sources (GCS, Azure), REST API adapters. |
| 5 | Visualization | Equity curves, drawdown charts, factor exposure plots, turnover analysis, HTML report generation. |
| 6 | Documentation | User manual, language reference, API docs, tutorials, example gallery with 10+ recipes. |

## Strategic milestones

| Milestone | Focus | Key Deliverables |
| --------- | ----- | ---------------- |
| **M0. Bootstrap** | Toolchain + binary skeleton | Cargo workspace, shared types crate, sled-backed cache harness, unified `sigc` binary with subcommand scaffolding, CI smoke build. |
| **M1. Language Core** | Compiler and typing | Chumsky (or equivalent) parser, AST with span metadata, symbol resolver, type/shape inference, minimal stdlib (`lag`, `ret`, `rolling.mean`, `zscore`, `rank`, `neutralize`, `winsor`, `clip`), IR lowering with rkyv serialization. |
| **M2. Execution Platform** | Deterministic compute | Columnar execution engine (Arrow/Polars), SIMD kernel prototypes for heavy transforms, sled-backed plan/materialized cache, CLI `compile`/`run` plumbing with reproducible artifacts, connector drivers (S3, SQL, FS). |
| **M3. Portfolio Construction** | Backtesting fidelity | Builtin backtester covering long/short workflows, beta & sector projections, turnover/cap enforcement, costs/slippage models, CSV/Parquet weight export and HTML/Markdown summary reports, attribution hooks. |
| **M4. Optimization & Data** | Performance + ingestion | Window fusion pass, CSE, Polars multi-core backend, calendar/universe management, NA policy controls, corporate action adjustment APIs. |
| **M5. Diagnostics & QA** | Trust + observability | Dependency graph visualizer, type/shape explain command, property tests vs reference numpy/pandas, reproducibility metadata (git sha, schema hashes), `sigc explain`, `sigc diff`, telemetry/metrics endpoints. |
| **M6. Extensibility** | Ecosystem adapters | Feature-gated PyO3 adapter, C ABI surface, nng RPC client/server modes, Rhai hook system, user-defined kernel sandbox, plugin registry documentation. |
| **M7. Python Front Door** | Notebook-first adoption | Thin PyO3-powered Python API mirroring DSL primitives (`SignalGraph`, decorators), seamless IR sharing between Python and DSL, notebook integration examples, Pandas/Polars result adapters. |
| **M8. Operations & Governance** | Enterprise readiness | Secrets/config profiles, RBAC hooks, audit logging policies, deployment guides (on-prem & cloud), security review checklist, contribution governance model. |
| **M9. Distribution & Adoption** | Productization | Release automation, cross-compilation targets, binary signing, example gallery covering momentum/value/alt data recipes, internal cookbook (10 recipes), contribution guide, licensing clarity. |

## Practitioner parity requirements

| Category | Required capabilities | Notes |
| -------- | --------------------- | ----- |
| Language & UX | Full signal DSL coverage (params, nested scopes, user-defined helpers), comprehensive stdlib for cross-sectional, time-series, portfolio operations, rich error diagnostics with source spans, inline doc/help system. | Should match or exceed ergonomics of internal desk DSLs; enable linting, formatting, and IDE hints. |
| Data & Integration | Connectors for Parquet/Arrow, CSV, SQL warehouses (Snowflake/Redshift), S3/GCS object stores with credential management, calendar/universe management, corporate actions handling, adjustable NA policies. | Practitioners expect one-click access to existing data lakes and corporate action adjustments without bespoke scripts. |
| Analytics & Backtesting | Long/short construction, beta/sector/country/style neutralization, flexible rebalancing schedules, turnover and exposure caps, multi-scenario/backtest sweeps, transaction cost and slippage models (bps, linear impact, square-root), benchmark-relative analytics. | Needs parity with institutional backtesters (AlyxLang, Quill, internal Python frameworks). |
| Reporting & Attribution | Factor/sector attribution, P&L decomposition (alpha/beta/costs), rolling metric dashboards, export to CSV/Parquet/HTML, integration with BI notebooks, human-readable run manifests, run diffing. | Enables investment committees and PMs to consume outputs directly. |
| Observability & Reproducibility | Artifact store with provenance (git sha, schema hashes, parameter dumps), deterministic replays, cache verification/repair tools, audit logs, metrics/telemetry exposed via CLI and optional Prometheus. | Satisfies compliance and model governance requirements. |
| Extensibility & Interop | PyO3 bridge for Python workflows, C ABI and RPC adapters, plugin hooks (Rhai), ability to call external risk models/optimizers, user-defined kernels (with safety sandbox). | Ensures teams can integrate with existing risk/OMS/analytics stacks. |
| Operations & Governance | Role-based access patterns, secrets management, configuration profiles per environment, release cadence with migrations, contribution guidelines, security reviews, licensing clarity. | Critical for production adoption inside regulated firms. |

## Adoption roadmap

| Stage | Target users | Objectives | Feature gates |
| ----- | ------------ | ---------- | ------------- |
| **Alpha (internal research)** | Core dev team + power users | Validate DSL ergonomics, IR correctness, runtime performance on curated datasets, iterate quickly on stdlib gaps. | Enable experimental language features; restrict to local data sources; manual artifact management. |
| **Beta (team-wide)** | Quant researchers within the org | Achieve functional parity with existing Python notebooks/backtesters, provide migration guides, ensure reproducibility and diagnostics meet desk expectations. | Turn on caching, built-in backtester, reporting, authentication hooks for shared data. |
| **Release Candidate** | Broader org + pilot clients | Harden performance, complete adapters (Python front door, RPC), finalize reporting stack, ensure integrations with risk/OMS pipelines, pilot RBAC/secrets profiles. | Feature freeze except bug fixes; start collecting adoption metrics; security review kickoff. |
| **General Availability** | External or multi-team deployment | Deliver signed binaries, full documentation, support SLA, governance process, plugin marketplace guidelines. | Lock down ABI, versioned DSL grammar, long-term support branches, telemetry opt-in/opt-out controls, pass security review checklist. |

## Risk & mitigation

- **Parser/DSL complexity** → Start with a restricted grammar; grow features after runtime stabilizes.
- **Runtime correctness** → Build property tests comparing against NumPy/Polars reference implementations.
- **Cache corruption** → Store versioned metadata with sled keys; provide `sigc cache verify` command for maintenance.
- **Single binary sprawl** → Use feature-gated modules so non-essential adapters can be disabled in `cargo build --no-default-features`.

## Decision log links (future)

- `docs/decisions/0001-cargo-layout.md` *(planned)*
- `docs/decisions/0002-dsl-grammar.md` *(planned)*
