//! Look-ahead-bias (point-in-time) type-system tests.
//!
//! These verify that sigc statically rejects signals that read future data,
//! and that the temporal `peek` is propagated correctly through the IR.

use sig_compiler::Compiler;

/// Compile a source string and return the error message, if any.
fn compile_err(source: &str) -> Option<String> {
    Compiler::new().compile(source).err().map(|e| e.to_string())
}

/// Look up the temporal peek of the (single) emitted output node.
fn output_peek(source: &str) -> i64 {
    let ir = Compiler::new()
        .compile(source)
        .expect("expected source to compile");
    let out_id = *ir.outputs.last().expect("no outputs");
    ir.nodes
        .iter()
        .find(|n| n.id == out_id)
        .expect("output node missing")
        .type_info
        .temporal
        .peek
}

// ---------------------------------------------------------------------------
// Corpus files
// ---------------------------------------------------------------------------

fn read(rel: &str) -> String {
    std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn leaky_future_return_is_rejected() {
    let err = compile_err(&read("../../examples/leaky/future_return.sig"))
        .expect("future_return.sig must fail to compile");
    assert!(
        err.contains("look-ahead"),
        "expected a look-ahead-bias error, got: {err}"
    );
}

#[test]
fn leaky_lead_peek_is_rejected() {
    let err = compile_err(&read("../../examples/leaky/lead_peek.sig"))
        .expect("lead_peek.sig must fail to compile");
    assert!(err.contains("look-ahead"), "got: {err}");
    // It peeks five bars ahead — the message should say so.
    assert!(err.contains('5'), "expected the 5-bar horizon in: {err}");
}

#[test]
fn safe_momentum_compiles() {
    let src = read("../../examples/leaky/safe_momentum.sig");
    assert!(
        compile_err(&src).is_none(),
        "safe_momentum.sig should compile, got: {:?}",
        compile_err(&src)
    );
    assert!(
        output_peek(&src) <= 0,
        "safe signal must be point-in-time (peek <= 0)"
    );
}

// ---------------------------------------------------------------------------
// Inline cases pinning the propagation rules
// ---------------------------------------------------------------------------

const DATA: &str = "data:\n  px: load parquet from \"prices.parquet\"\n\n";

#[test]
fn backward_operators_are_point_in_time() {
    // ret over a positive window + trailing rolling stat: peek must be 0.
    let src = format!("{DATA}signal s:\n  emit zscore(ret(px, periods=10))\n");
    assert_eq!(output_peek(&src), 0);
}

#[test]
fn negative_period_lag_peeks_forward() {
    // lag(px, periods=-3) reads px[t+3]; the error must report 3 bars.
    let src = format!("{DATA}signal s:\n  emit lag(px, periods=-3)\n");
    let err = compile_err(&src).expect("negative lag must be rejected");
    assert!(err.contains("look-ahead"), "got: {err}");
    assert!(err.contains('3'), "expected 3-bar horizon in: {err}");
}

#[test]
fn future_leak_survives_arithmetic_composition() {
    // A forward read combined with safe data still taints the result:
    // max(peek) propagates through the binary op.
    let src = format!(
        "{DATA}signal s:\n  bad = ret(px, periods=-2)\n  good = rolling_mean(px, window=5)\n  emit zscore(bad + good)\n"
    );
    let err = compile_err(&src).expect("composition with a future read must be rejected");
    assert!(err.contains("look-ahead"), "got: {err}");
}
