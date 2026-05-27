//! Statistical-rigor tools for honest backtest evaluation.
//!
//! A backtest Sharpe ratio, taken at face value, is the single most misleading
//! number in quantitative finance: it ignores non-normality, the *number of
//! trials* that produced it, and the multiple-testing inflation that comes from
//! searching over many strategies. This module makes those corrections
//! first-class so a reported result can be defended:
//!
//! * [`probabilistic_sharpe_ratio`] / [`deflated_sharpe_ratio`] — Bailey &
//!   López de Prado (2014), "The Deflated Sharpe Ratio". The DSR corrects the
//!   Sharpe for skew, kurtosis, sample length, *and* the number of trials.
//! * [`pbo_cscv`] — Bailey, Borwein, López de Prado & Zhu, "The Probability of
//!   Backtest Overfitting" via Combinatorially-Symmetric Cross-Validation.
//! * [`bonferroni_haircut`] / [`holm_adjust`] / [`bhy_adjust`] — Harvey, Liu &
//!   Zhu (2016), "…and the Cross-Section of Expected Returns" multiple-testing
//!   adjustments.
//! * [`permutation_sharpe_pvalue`] — distribution-free Monte-Carlo p-value.
//! * [`min_track_record_length`] — Bailey & López de Prado, the sample length
//!   needed before a Sharpe is statistically distinguishable from a benchmark.
//!
//! All Sharpe ratios here are **per-period** (not annualized) unless stated;
//! annualization is a monotone rescaling and cancels in every ratio below.

use sig_types::{Result, SigcError};

/// Euler–Mascheroni constant (used by the expected-maximum-Sharpe estimator).
const EULER_MASCHERONI: f64 = 0.577_215_664_901_532_9;

// ---------------------------------------------------------------------------
// Normal distribution helpers
// ---------------------------------------------------------------------------

/// Error function (Abramowitz & Stegun 7.1.26; |error| < 1.5e-7).
fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    sign * y
}

/// Standard normal CDF, Φ(x).
pub fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal inverse CDF (probit), Φ⁻¹(p), via Acklam's algorithm.
pub fn norm_ppf(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.024_25;
    let p_high = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

// ---------------------------------------------------------------------------
// Return statistics
// ---------------------------------------------------------------------------

/// Summary moments of a return series, plus the per-period Sharpe ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReturnStats {
    /// Number of observations.
    pub n: usize,
    pub mean: f64,
    /// Population standard deviation.
    pub std: f64,
    /// Skewness (γ₃; 0 for a normal distribution).
    pub skew: f64,
    /// Kurtosis (γ₄; 3 for a normal distribution — *not* excess kurtosis).
    pub kurt: f64,
    /// Per-period Sharpe ratio, `mean / std`.
    pub sharpe: f64,
}

impl ReturnStats {
    /// Compute moments from a return series. All moments are population
    /// (÷N) moments, matching the conventions in Bailey & López de Prado.
    pub fn from_returns(returns: &[f64]) -> ReturnStats {
        let n = returns.len();
        if n == 0 {
            return ReturnStats { n: 0, mean: 0.0, std: 0.0, skew: 0.0, kurt: 3.0, sharpe: 0.0 };
        }
        let nf = n as f64;
        let mean = returns.iter().sum::<f64>() / nf;
        let m2 = returns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nf;
        let std = m2.sqrt();
        let (skew, kurt) = if std > 0.0 {
            let m3 = returns.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / nf;
            let m4 = returns.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / nf;
            (m3 / std.powi(3), m4 / std.powi(4))
        } else {
            (0.0, 3.0)
        };
        let sharpe = if std > 0.0 { mean / std } else { 0.0 };
        ReturnStats { n, mean, std, skew, kurt, sharpe }
    }
}

/// Per-period Sharpe of a slice (population std). Returns 0 if degenerate.
fn sharpe_of(returns: &[f64]) -> f64 {
    ReturnStats::from_returns(returns).sharpe
}

// ---------------------------------------------------------------------------
// Probabilistic / Deflated Sharpe Ratio
// ---------------------------------------------------------------------------

/// Probabilistic Sharpe Ratio: the probability that the *true* per-period
/// Sharpe exceeds `benchmark_sr`, given the observed Sharpe and its
/// non-normality (Bailey & López de Prado, 2012).
///
/// `PSR = Φ( (ŜR − SR*)·√(n−1) / √(1 − γ₃·ŜR + ((γ₄−1)/4)·ŜR²) )`
pub fn probabilistic_sharpe_ratio(stats: &ReturnStats, benchmark_sr: f64) -> f64 {
    if stats.n < 2 {
        return 0.0;
    }
    let sr = stats.sharpe;
    let denom = (1.0 - stats.skew * sr + ((stats.kurt - 1.0) / 4.0) * sr * sr).max(1e-12);
    let z = (sr - benchmark_sr) * ((stats.n as f64 - 1.0).sqrt()) / denom.sqrt();
    norm_cdf(z)
}

/// Expected maximum Sharpe ratio across `num_trials` independent strategies,
/// each with Sharpe variance `variance_of_trial_sr` (Bailey & López de Prado).
///
/// `E[max] ≈ √V · [ (1−γ)·Z⁻¹(1 − 1/T) + γ·Z⁻¹(1 − 1/(T·e)) ]`
pub fn expected_max_sharpe(variance_of_trial_sr: f64, num_trials: usize) -> f64 {
    if num_trials <= 1 || variance_of_trial_sr <= 0.0 {
        return 0.0;
    }
    let t = num_trials as f64;
    let term = (1.0 - EULER_MASCHERONI) * norm_ppf(1.0 - 1.0 / t)
        + EULER_MASCHERONI * norm_ppf(1.0 - 1.0 / (t * std::f64::consts::E));
    variance_of_trial_sr.sqrt() * term
}

/// Deflated Sharpe Ratio: the PSR evaluated against the *expected maximum*
/// Sharpe that `num_trials` random trials would produce by chance. A DSR near
/// 1 survives the multiple-trials deflation; a DSR near 0.5 or below does not.
///
/// `variance_of_trial_sr` is the variance of the (per-period) Sharpe ratios
/// across the trials that were searched.
pub fn deflated_sharpe_ratio(
    stats: &ReturnStats,
    num_trials: usize,
    variance_of_trial_sr: f64,
) -> f64 {
    let sr_star = expected_max_sharpe(variance_of_trial_sr, num_trials);
    probabilistic_sharpe_ratio(stats, sr_star)
}

/// Minimum Track Record Length: the number of observations required before the
/// Probabilistic Sharpe Ratio against `benchmark_sr` reaches `confidence`
/// (e.g. 0.95). Returns `None` if the observed Sharpe does not exceed the
/// benchmark (in which case no sample length suffices).
pub fn min_track_record_length(
    stats: &ReturnStats,
    benchmark_sr: f64,
    confidence: f64,
) -> Option<f64> {
    let sr = stats.sharpe;
    if sr <= benchmark_sr {
        return None;
    }
    let z = norm_ppf(confidence);
    let factor = 1.0 - stats.skew * sr + ((stats.kurt - 1.0) / 4.0) * sr * sr;
    Some(1.0 + factor * (z / (sr - benchmark_sr)).powi(2))
}

// ---------------------------------------------------------------------------
// Multiple-testing haircuts (Harvey–Liu–Zhu)
// ---------------------------------------------------------------------------

/// Result of a multiple-testing haircut applied to a single Sharpe ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Haircut {
    /// Two-sided p-value treating the strategy as the only test.
    pub p_single: f64,
    /// Multiple-testing-adjusted p-value.
    pub p_adjusted: f64,
    /// The Sharpe after the haircut (same units as the input Sharpe).
    pub haircut_sharpe: f64,
    /// Fraction of the original Sharpe removed, in `[0, 1]`.
    pub haircut_pct: f64,
}

/// Bonferroni haircut: deflate an observed Sharpe for having run `num_tests`
/// independent tests. The t-statistic is `ŜR·√n`; its two-sided p-value is
/// multiplied by `num_tests`, then converted back to a haircut Sharpe.
pub fn bonferroni_haircut(stats: &ReturnStats, num_tests: usize) -> Haircut {
    let sr = stats.sharpe;
    let n = stats.n.max(1) as f64;
    let t = sr * n.sqrt();
    let p_single = 2.0 * (1.0 - norm_cdf(t.abs()));
    let p_adjusted = (p_single * num_tests.max(1) as f64).min(1.0);
    // Convert the adjusted p-value back to a t-stat, then to a Sharpe. Clamp to
    // the original |t|: a haircut can only shrink the Sharpe, never grow it.
    // (Without the cap, an underflowed p_single = 0 would give norm_ppf(1) = ∞.)
    let t_adj = norm_ppf(1.0 - p_adjusted / 2.0).clamp(0.0, t.abs());
    let haircut_sharpe = (t_adj / n.sqrt()) * sr.signum();
    let haircut_pct = if sr.abs() > 0.0 {
        (1.0 - haircut_sharpe.abs() / sr.abs()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Haircut { p_single, p_adjusted, haircut_sharpe, haircut_pct }
}

/// Holm step-down adjusted p-values (controls family-wise error rate).
pub fn holm_adjust(pvalues: &[f64]) -> Vec<f64> {
    let m = pvalues.len();
    if m == 0 {
        return Vec::new();
    }
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|&a, &b| pvalues[a].partial_cmp(&pvalues[b]).unwrap());

    let mut adjusted = vec![0.0; m];
    let mut running: f64 = 0.0;
    for (rank, &idx) in order.iter().enumerate() {
        let factor = (m - rank) as f64;
        running = running.max((factor * pvalues[idx]).min(1.0));
        adjusted[idx] = running;
    }
    adjusted
}

/// Benjamini–Hochberg–Yekutieli adjusted p-values (controls the false
/// discovery rate under arbitrary dependence — the variant used by HLZ).
pub fn bhy_adjust(pvalues: &[f64]) -> Vec<f64> {
    let m = pvalues.len();
    if m == 0 {
        return Vec::new();
    }
    // c(M) = Σ_{i=1}^M 1/i (the Yekutieli dependence correction).
    let c_m: f64 = (1..=m).map(|i| 1.0 / i as f64).sum();

    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|&a, &b| pvalues[a].partial_cmp(&pvalues[b]).unwrap());

    let mut adjusted = vec![0.0; m];
    let mut running = f64::INFINITY;
    // Step up from the largest p-value to enforce monotonicity.
    for rank in (0..m).rev() {
        let idx = order[rank];
        let val = (c_m * m as f64 / (rank as f64 + 1.0)) * pvalues[idx];
        running = running.min(val).min(1.0);
        adjusted[idx] = running;
    }
    adjusted
}

// ---------------------------------------------------------------------------
// Permutation test
// ---------------------------------------------------------------------------

/// Distribution-free Monte-Carlo p-value for an observed Sharpe ratio under the
/// null that the sign of each return is random (a zero-edge strategy). Returns
/// the fraction of sign-flipped resamples whose Sharpe meets or exceeds the
/// observed Sharpe. Deterministic given `seed`.
pub fn permutation_sharpe_pvalue(returns: &[f64], num_perms: usize, seed: u64) -> f64 {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    if returns.len() < 2 || num_perms == 0 {
        return 1.0;
    }
    let observed = sharpe_of(returns);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut at_least = 0usize;
    let mut buf = vec![0.0; returns.len()];
    for _ in 0..num_perms {
        for (b, &r) in buf.iter_mut().zip(returns.iter()) {
            *b = if rng.gen::<bool>() { r } else { -r };
        }
        if sharpe_of(&buf) >= observed {
            at_least += 1;
        }
    }
    // +1 smoothing avoids a p-value of exactly zero.
    (at_least as f64 + 1.0) / (num_perms as f64 + 1.0)
}

// ---------------------------------------------------------------------------
// Probability of Backtest Overfitting (CSCV)
// ---------------------------------------------------------------------------

/// Result of a CSCV probability-of-backtest-overfitting estimate.
#[derive(Debug, Clone)]
pub struct PboResult {
    /// Probability of backtest overfitting in `[0, 1]`.
    pub pbo: f64,
    /// Number of in-sample/out-of-sample combinations evaluated.
    pub n_combinations: usize,
    /// Logit of the OOS rank of the IS-best config, one per combination.
    pub logits: Vec<f64>,
}

/// Probability of Backtest Overfitting via Combinatorially-Symmetric
/// Cross-Validation (Bailey et al., 2014).
///
/// `matrix[c][t]` is the period-`t` return of configuration `c`. The series is
/// split into `n_splits` (even) contiguous partitions; for every way of
/// choosing half the partitions as in-sample, the config with the best IS
/// Sharpe is found and its *rank* among all configs out-of-sample is computed.
/// If the IS winner systematically lands below the OOS median, the search is
/// overfitting. PBO is the fraction of combinations whose OOS logit ≤ 0.
pub fn pbo_cscv(matrix: &[Vec<f64>], n_splits: usize) -> Result<PboResult> {
    let n_configs = matrix.len();
    if n_configs < 2 {
        return Err(SigcError::Runtime(
            "PBO requires at least 2 configurations".into(),
        ));
    }
    if n_splits < 2 || n_splits % 2 != 0 {
        return Err(SigcError::Runtime(
            "PBO requires an even n_splits >= 2".into(),
        ));
    }
    let t_len = matrix[0].len();
    if matrix.iter().any(|r| r.len() != t_len) {
        return Err(SigcError::Runtime(
            "PBO: all configurations must have equal length".into(),
        ));
    }
    if t_len < n_splits {
        return Err(SigcError::Runtime(
            "PBO: fewer observations than splits".into(),
        ));
    }

    // Contiguous, near-even partition boundaries over [0, t_len).
    let bounds: Vec<(usize, usize)> = (0..n_splits)
        .map(|s| (s * t_len / n_splits, (s + 1) * t_len / n_splits))
        .collect();

    let half = n_splits / 2;
    let combos = combinations(n_splits, half);

    let mut logits = Vec::with_capacity(combos.len());
    for is_parts in &combos {
        let is_set: std::collections::HashSet<usize> = is_parts.iter().copied().collect();

        // Sharpe of each config in-sample and out-of-sample.
        let mut is_sharpe = vec![0.0; n_configs];
        let mut oos_sharpe = vec![0.0; n_configs];
        for (c, series) in matrix.iter().enumerate() {
            let mut is_slice = Vec::new();
            let mut oos_slice = Vec::new();
            for (s, &(lo, hi)) in bounds.iter().enumerate() {
                if is_set.contains(&s) {
                    is_slice.extend_from_slice(&series[lo..hi]);
                } else {
                    oos_slice.extend_from_slice(&series[lo..hi]);
                }
            }
            is_sharpe[c] = sharpe_of(&is_slice);
            oos_sharpe[c] = sharpe_of(&oos_slice);
        }

        // Best IS config, then its relative rank OOS.
        let best = (0..n_configs)
            .max_by(|&a, &b| is_sharpe[a].partial_cmp(&is_sharpe[b]).unwrap())
            .unwrap();
        let worse_oos = oos_sharpe
            .iter()
            .filter(|&&s| s < oos_sharpe[best])
            .count();
        // Relative rank ω ∈ (0,1): fraction of configs the winner beats OOS.
        let omega = (worse_oos as f64 + 0.5) / n_configs as f64;
        let omega = omega.clamp(1e-6, 1.0 - 1e-6);
        logits.push((omega / (1.0 - omega)).ln());
    }

    let n_combinations = logits.len();
    let pbo = if n_combinations > 0 {
        logits.iter().filter(|&&l| l <= 0.0).count() as f64 / n_combinations as f64
    } else {
        0.0
    };

    Ok(PboResult { pbo, n_combinations, logits })
}

/// All size-`k` subsets of `0..n`, as sorted index vectors.
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..k).collect();
    if k == 0 || k > n {
        return out;
    }
    loop {
        out.push(idx.clone());
        // Advance to the next combination in lexicographic order.
        let mut i = k;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] != i + n - k {
                break;
            }
        }
        idx[i] += 1;
        for j in (i + 1)..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Aggregate report
// ---------------------------------------------------------------------------

/// A bundled rigor assessment of a single strategy's return series.
#[derive(Debug, Clone)]
pub struct RigorReport {
    pub stats: ReturnStats,
    /// Annualized Sharpe (per-period × √periods_per_year), for display.
    pub annualized_sharpe: f64,
    /// PSR against a zero benchmark.
    pub psr: f64,
    /// Deflated Sharpe given the number of trials searched.
    pub deflated_sharpe: f64,
    /// Bonferroni multiple-testing haircut.
    pub haircut: Haircut,
    /// Permutation p-value of the observed Sharpe.
    pub permutation_pvalue: f64,
    /// Minimum track-record length (observations) vs a zero benchmark.
    pub min_track_record: Option<f64>,
}

impl RigorReport {
    /// Compute a full rigor report.
    ///
    /// * `num_trials` — how many strategy/parameter configurations were
    ///   searched to arrive at this one (1 if none).
    /// * `variance_of_trial_sr` — variance of the per-period Sharpe across
    ///   those trials (0 collapses the deflation to the plain PSR).
    /// * `periods_per_year` — for the displayed annualized Sharpe (e.g. 252).
    pub fn compute(
        returns: &[f64],
        num_trials: usize,
        variance_of_trial_sr: f64,
        periods_per_year: f64,
    ) -> RigorReport {
        let stats = ReturnStats::from_returns(returns);
        RigorReport {
            annualized_sharpe: stats.sharpe * periods_per_year.sqrt(),
            psr: probabilistic_sharpe_ratio(&stats, 0.0),
            deflated_sharpe: deflated_sharpe_ratio(&stats, num_trials, variance_of_trial_sr),
            haircut: bonferroni_haircut(&stats, num_trials),
            permutation_pvalue: permutation_sharpe_pvalue(returns, 1000, 0xC0FFEE),
            min_track_record: min_track_record_length(&stats, 0.0, 0.95),
            stats,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn normal_cdf_and_ppf_are_inverses() {
        assert!(approx(norm_cdf(0.0), 0.5, 1e-6));
        assert!(approx(norm_cdf(1.96), 0.975, 1e-3));
        assert!(approx(norm_ppf(0.975), 1.96, 1e-3));
        for &p in &[0.01, 0.1, 0.5, 0.9, 0.99] {
            assert!(approx(norm_cdf(norm_ppf(p)), p, 1e-4), "roundtrip p={p}");
        }
    }

    #[test]
    fn return_stats_on_normalish_series() {
        // Symmetric series -> skew ~ 0.
        let r = vec![-0.02, -0.01, 0.0, 0.01, 0.02];
        let s = ReturnStats::from_returns(&r);
        assert_eq!(s.n, 5);
        assert!(approx(s.mean, 0.0, 1e-9));
        assert!(approx(s.skew, 0.0, 1e-9));
    }

    #[test]
    fn psr_increases_with_sharpe() {
        let weak = ReturnStats::from_returns(&vec![0.001, -0.0005, 0.0008, -0.0003, 0.0006]);
        let strong = ReturnStats::from_returns(&vec![0.01, 0.009, 0.011, 0.008, 0.012]);
        assert!(probabilistic_sharpe_ratio(&strong, 0.0) > probabilistic_sharpe_ratio(&weak, 0.0));
        // A consistently positive series should be highly significant.
        assert!(probabilistic_sharpe_ratio(&strong, 0.0) > 0.99);
    }

    #[test]
    fn deflation_reduces_confidence_as_trials_grow() {
        let stats = ReturnStats::from_returns(&vec![0.01, 0.005, 0.012, 0.003, 0.009, 0.007, 0.011]);
        let dsr_few = deflated_sharpe_ratio(&stats, 2, 0.5);
        let dsr_many = deflated_sharpe_ratio(&stats, 1000, 0.5);
        assert!(dsr_many <= dsr_few, "more trials must not increase the DSR");
    }

    #[test]
    fn bonferroni_haircut_shrinks_sharpe() {
        let stats = ReturnStats::from_returns(&vec![0.01, 0.008, 0.012, 0.009, 0.011, 0.007]);
        let single = bonferroni_haircut(&stats, 1);
        let many = bonferroni_haircut(&stats, 100);
        assert!(many.haircut_sharpe.abs() <= single.haircut_sharpe.abs());
        assert!(many.haircut_pct >= single.haircut_pct);
        assert!(many.p_adjusted >= single.p_adjusted);
    }

    #[test]
    fn haircut_sharpe_is_finite_for_extreme_significance() {
        // A near-constant positive series has an enormous Sharpe, so its
        // p-value underflows to 0. The haircut must stay finite and never
        // exceed the original Sharpe (regression for the norm_ppf(1)=∞ case).
        let near_constant: Vec<f64> = (0..252).map(|i| 0.01 + 1e-6 * (i % 3) as f64).collect();
        let stats = ReturnStats::from_returns(&near_constant);
        let h = bonferroni_haircut(&stats, 100);
        assert!(h.haircut_sharpe.is_finite(), "haircut Sharpe must be finite");
        assert!(h.haircut_sharpe.abs() <= stats.sharpe.abs() + 1e-9);
        assert!((0.0..=1.0).contains(&h.haircut_pct));
    }

    #[test]
    fn holm_and_bhy_are_monotone_and_bounded() {
        let p = vec![0.001, 0.01, 0.03, 0.2, 0.5];
        for adj in [holm_adjust(&p), bhy_adjust(&p)] {
            assert_eq!(adj.len(), p.len());
            assert!(adj.iter().all(|&x| (0.0..=1.0).contains(&x)));
            // Adjusted p-values are >= raw p-values.
            assert!(adj.iter().zip(&p).all(|(&a, &raw)| a + 1e-12 >= raw));
        }
    }

    #[test]
    fn permutation_pvalue_small_for_strong_signal() {
        let r: Vec<f64> = (0..100).map(|i| 0.01 + 0.001 * (i % 3) as f64).collect();
        let p = permutation_sharpe_pvalue(&r, 2000, 42);
        assert!(p < 0.01, "strong positive series should be significant, got {p}");
    }

    #[test]
    fn permutation_pvalue_is_deterministic() {
        let r = vec![0.01, -0.02, 0.03, -0.01, 0.02, -0.015, 0.025];
        let a = permutation_sharpe_pvalue(&r, 500, 7);
        let b = permutation_sharpe_pvalue(&r, 500, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn combinations_count_is_correct() {
        // C(6,3) = 20
        assert_eq!(combinations(6, 3).len(), 20);
        // C(10,5) = 252
        assert_eq!(combinations(10, 5).len(), 252);
    }

    #[test]
    fn pbo_high_for_pure_noise_configs() {
        // Build many configs of i.i.d.-ish noise; no config has true edge,
        // so the IS winner should land near the OOS median => PBO near 0.5+.
        let n_configs = 20;
        let t = 240;
        let mut matrix = Vec::new();
        for c in 0..n_configs {
            let mut seed = (c as u64).wrapping_mul(2654435761).wrapping_add(1);
            let series: Vec<f64> = (0..t)
                .map(|_| {
                    // cheap deterministic LCG in [-0.5, 0.5)
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5
                })
                .collect();
            matrix.push(series);
        }
        let res = pbo_cscv(&matrix, 10).unwrap();
        assert_eq!(res.n_combinations, 252);
        assert!(res.pbo > 0.3, "noise configs should overfit substantially, got {}", res.pbo);
    }

    #[test]
    fn min_track_record_none_when_below_benchmark() {
        let stats = ReturnStats::from_returns(&vec![-0.01, -0.005, 0.001, -0.002]);
        assert!(min_track_record_length(&stats, 0.0, 0.95).is_none());
    }

    #[test]
    fn rigor_report_smoke() {
        let r: Vec<f64> = (0..252).map(|i| 0.0005 + 0.002 * ((i as f64 * 0.3).sin())).collect();
        let rep = RigorReport::compute(&r, 50, 0.25, 252.0);
        assert!(rep.psr >= 0.0 && rep.psr <= 1.0);
        assert!(rep.deflated_sharpe >= 0.0 && rep.deflated_sharpe <= 1.0);
        assert!(rep.haircut.haircut_pct >= 0.0);
    }
}
