# sigc

`sigc` is a Rust-first research platform that lets quantitative teams prototype, validate, and operationalize alpha ideas with the speed of an interactive DSL and the determinism of a compiler/runtime pair. The end-state goal is a **single, self-contained binary** that ships the compiler, runtime, and orchestration surface so researchers get a turnkey “compile + run” loop without juggling services.

## Why it exists

Quant shops lose hours every week to ad hoc Python notebooks: mismatched calendars, fragile joins, non-deterministic backtests, and duplicated factor code. Every major institutional desk that scaled past a handful of researchers eventually built a typed, reproducible signal language. `sigc` is our open implementation of that idea — a composable, shape-aware DSL backed by a columnar runtime and deterministic backtester.

## What the system looks like

- **Single binary (`sigc`)** bundles compiler, runtime, daemon, and CLI personas so deployments stay simple while still exposing subcommands (`sigc compile`, `sigc run`, `sigc daemon`, etc.).
- **Compiler module** parses the DSL into a typed IR, optimizes it, and produces executable plans that can be cached and reused.
- **Runtime module** executes plans against columnar data (Arrow/Polars), parallelized with Rayon and SIMD kernels for heavy factor math.
- **Daemon mode** serves `Compile+Run` requests over nng for clients that prefer a long-lived service.
- **CLI/REPL mode** offers DSL editing, hot reload, helpful errors, and artifact inspection.
- **Caching** with sled + blake3 keeps both plan compilation and materialized panels fast and reproducible.
- **Adapters** let BacktestPlans execute on the builtin engine, PyO3 bridges, C ABI stubs, or RPC clients — without touching the core.

## Getting started (planned)

1. **Install Rust nightly + toolchain** (exact instructions TBD once crates land).
2. `cargo build --bin sigc` to produce the single binary.
3. `sigc run examples/momentum.sig` to compile and backtest a sample factor (see `examples/`).
4. Inspect cached artifacts with `sigc explain <run-id>` or compare runs using `sigc diff A B`.

> Until code lands, follow the engineering plan in `docs/` for the implementation roadmap.

## Project status

- Architecture and business motivations are captured in [`specs.md`](specs.md).
- Implementation is pre-alpha; we are currently drafting the compiler/runtime scaffolding and unifying binaries.
- Community feedback is welcome — especially from quant researchers or infra teams who have fought similar battles.

## Documentation

- High-level specs: [`specs.md`](specs.md)
- Build plan and milestones: [`docs/build-roadmap.md`](docs/build-roadmap.md)
- Sample strategies: [`examples/momentum.sig`](examples/momentum.sig), [`examples/meanreversion.sig`](examples/meanreversion.sig), [`examples/combo.sig`](examples/combo.sig)
- Python front door (planned): thin PyO3 API for notebook workflows, see roadmap milestone M7
- OSS contribution guidelines: (planned)

## Contributing

This repository is in active design. Please open an issue or discussion with your context before submitting large PRs so we can keep the roadmap coherent.

## License

License TBD (likely Apache-2.0 or MIT once legal clears). For now, treat the repo as all-rights-reserved until we formalize the terms.
