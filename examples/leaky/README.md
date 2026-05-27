# Look-ahead bias corpus

These `.sig` files exercise sigc's **point-in-time temporal type system**, which
makes look-ahead bias a *compile error* instead of a silently-passing backtest.

Every value in the IR carries a temporal `peek` — the maximum number of bars
into the future it reads (see `sig_types::Temporal`). The lowering pass
propagates it as `out_peek = max(input peeks) + operator.temporal_shift()`, and
an emitted signal is rejected if `peek > 0`.

| File | Outcome | Why |
|------|---------|-----|
| `future_return.sig` | **compile error** | `ret(px, periods=-1)` reads `px[t+1]` (peek = 1) |
| `lead_peek.sig` | **compile error** | `lag(px, periods=-5)` reads `px[t+5]` (peek = 5) |
| `safe_momentum.sig` | compiles | only backward-looking ops (peek = 0) |

Try them:

```bash
sigc compile examples/leaky/future_return.sig   # error: look-ahead bias ...
sigc compile examples/leaky/safe_momentum.sig    # ok
```

Regression-tested in `crates/sigc/tests/leakage.rs`.
