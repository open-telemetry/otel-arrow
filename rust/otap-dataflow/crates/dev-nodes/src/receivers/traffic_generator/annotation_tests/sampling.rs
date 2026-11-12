// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Sampler tests: standard normal functions, in-range guarantees for every
//! continuous distribution, and distribution shape.
//!
//! Shared helpers live in the parent module.

use super::*;

// ===========================================================================
// Sampling: standard normal functions
// ===========================================================================

/// Standard normal CDF reference values `(z, Phi(z))`.
///
/// Computed with 40-digit arithmetic (`mpmath`: `erfc(-z / sqrt(2)) / 2`) and
/// rounded to f64; they match published standard normal tables.
const CDF_REFERENCE: [(f64, f64); 15] = [
    (0.0, 0.5),
    (0.5, 0.691_462_461_274_013_1),
    (-0.5, 0.308_537_538_725_986_9),
    (1.0, 0.841_344_746_068_542_9),
    (-1.0, 0.158_655_253_931_457_05),
    (1.96, 0.975_002_104_851_779_6),
    (-1.96, 0.024_997_895_148_220_436),
    (3.0, 0.998_650_101_968_369_9),
    (-3.0, 0.001_349_898_031_630_094_5),
    (5.0, 0.999_999_713_348_428_1),
    (-5.0, 2.866_515_718_791_939e-7),
    (-8.0, 6.220_960_574_271_784e-16),
    (-10.0, 7.619_853_024_160_526e-24),
    (-20.0, 2.753_624_118_606_233_7e-89),
    (-37.0, 5.725_571_222_524_577e-300),
];

/// Standard normal quantile reference values `(p, z)` with `Phi(z) = p`.
///
/// Computed with 60-digit arithmetic (`mpmath`: root of
/// `log(Phi(z)) - log(p)`).
const QUANTILE_REFERENCE: [(f64, f64); 8] = [
    (0.975, 1.959_963_984_540_054_2),
    (0.995, 2.575_829_303_548_901),
    (0.001, -3.090_232_306_167_813_5),
    (1e-10, -6.361_340_902_404_056),
    (1e-20, -9.262_340_089_798_408),
    (1e-100, -21.273_453_560_965_324),
    (1e-300, -37.047_096_299_361_2),
    (0.5, 0.0),
];

fn rel_err(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        (a - b).abs() / a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
    }
}

fn z_grid() -> Vec<f64> {
    (-370..=370).map(|i| f64::from(i) * 0.1).collect()
}

fn p_grid() -> Vec<f64> {
    let mut ps: Vec<f64> = (1..=300).map(|e| 10f64.powi(-e)).collect();
    ps.extend((1..1000).map(|i| f64::from(i) / 1000.0));
    ps.extend((1..=16).map(|e| 1.0 - 10f64.powi(-e)));
    ps.sort_by(f64::total_cmp);
    ps
}

/// Scenario: `normal_cdf` and `normal_sf` are evaluated at tabulated points
/// from the center out to z = -37 (p ~ 1e-300).
/// Guarantees: Both match high-precision reference values to 1e-12 relative
/// error, and `normal_sf` keeps that precision in the upper tail where
/// `1 - cdf` would cancel to zero.
#[test]
fn normal_cdf_and_sf_match_reference_values() {
    for (z, expected) in CDF_REFERENCE {
        let cdf = normal_cdf(z);
        assert!(
            rel_err(cdf, expected) < 1e-12,
            "cdf({z}) = {cdf}, expected {expected}"
        );
        let sf = normal_sf(-z);
        assert!(
            rel_err(sf, expected) < 1e-12,
            "sf({}) = {sf}, expected {expected}",
            -z
        );
    }
}

/// Scenario: `normal_quantile` and `normal_inverse_sf` are evaluated at
/// tabulated probabilities from 0.5 down to 1e-300.
/// Guarantees: Both match high-precision reference quantiles to 1e-12
/// relative error (or 1e-15 absolute at zero), so truncated sampling places
/// values where the exact distribution would.
#[test]
fn normal_quantile_matches_reference_values() {
    for (p, expected) in QUANTILE_REFERENCE {
        let z = normal_quantile(p);
        assert!(
            rel_err(z, expected) < 1e-12 || (z - expected).abs() < 1e-15,
            "quantile({p}) = {z}, expected {expected}"
        );
        let upper = normal_inverse_sf(p);
        assert!(
            rel_err(upper, -expected) < 1e-12 || (upper + expected).abs() < 1e-15,
            "inverse_sf({p}) = {upper}, expected {}",
            -expected
        );
    }
}

/// Scenario: Symmetry identities of the standard normal are checked over a
/// dense grid of z in [-37, 37] and p from 1e-300 to 1 - 1e-16.
/// Guarantees: `cdf(z) + cdf(-z) = 1`, `sf(z) == cdf(-z)` exactly, and
/// `quantile(p) == -inverse_sf(p)` exactly, so lower- and upper-tail
/// sampling paths are consistent.
#[test]
fn normal_functions_are_symmetric() {
    for z in z_grid() {
        assert!(
            (normal_cdf(z) + normal_cdf(-z) - 1.0).abs() < 1e-15,
            "z = {z}"
        );
        assert_eq!(normal_sf(z), normal_cdf(-z), "z = {z}");
    }
    for p in p_grid() {
        assert_eq!(normal_quantile(p), -normal_inverse_sf(p), "p = {p}");
    }
}

/// Scenario: The CDF, survival function, and quantile are evaluated over
/// increasing inputs.
/// Guarantees: CDF and quantile are non-decreasing, the survival function is
/// non-increasing, and every probability lies in [0, 1], so the inverse
/// transform maps ordered uniforms to ordered samples.
#[test]
fn normal_functions_are_monotonic_and_bounded() {
    let zs = z_grid();
    for w in zs.windows(2) {
        assert!(normal_cdf(w[0]) <= normal_cdf(w[1]), "cdf at {}", w[1]);
        assert!(normal_sf(w[0]) >= normal_sf(w[1]), "sf at {}", w[1]);
    }
    for z in zs {
        assert!((0.0..=1.0).contains(&normal_cdf(z)));
        assert!((0.0..=1.0).contains(&normal_sf(z)));
    }
    let ps = p_grid();
    for w in ps.windows(2) {
        assert!(
            normal_quantile(w[0]) <= normal_quantile(w[1]),
            "quantile at {}",
            w[1]
        );
    }
}

/// Scenario: z is round-tripped through `normal_cdf` / `normal_quantile` (z <=
/// 0) and `normal_sf` / `normal_inverse_sf` (z > 0) over [-37, 37].
/// Guarantees: The quantile inverts the CDF to 1e-9 absolute error, which
/// verifies the Halley refinement on top of the rational approximation.
#[test]
fn normal_quantile_inverts_cdf() {
    for z in z_grid() {
        let back = if z <= 0.0 {
            normal_quantile(normal_cdf(z))
        } else {
            normal_inverse_sf(normal_sf(z))
        };
        assert!((back - z).abs() < 1e-9, "round trip of {z} gave {back}");
    }
}

/// Scenario: The derivative of the lower-tail probability (`normal_cdf` for
/// z <= 0, `normal_sf` for z > 0) is estimated by a central difference over z
/// in [-8, 8].
/// Guarantees: It matches the closed-form normal density
/// `exp(-z^2 / 2) / sqrt(2 pi)` to 1e-6 relative error, checking the CDF and
/// survival function against an independent formula.
#[test]
fn normal_cdf_derivative_matches_density() {
    for i in -80..=80 {
        let z = f64::from(i) * 0.1;
        // Differentiate the small tail probability so the difference does not
        // cancel against values near 1; scale the step with z.
        let h = 1e-6 * z.abs().max(1.0);
        let numeric = if z <= 0.0 {
            (normal_cdf(z + h) - normal_cdf(z - h)) / (2.0 * h)
        } else {
            (normal_sf(z - h) - normal_sf(z + h)) / (2.0 * h)
        };
        let density = (-z * z / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
        assert!(
            rel_err(numeric, density) < 1e-6,
            "z = {z}: d/dz cdf = {numeric}, density = {density}"
        );
    }
}

/// Scenario: The normal functions are evaluated at their domain edges.
/// Guarantees: Probabilities 0 and 1 map to infinite quantiles, infinite z
/// maps to exactly 0 or 1, and no probability in [0, 1] yields NaN.
#[test]
fn normal_functions_handle_edges() {
    assert_eq!(normal_quantile(0.0), f64::NEG_INFINITY);
    assert_eq!(normal_quantile(1.0), f64::INFINITY);
    assert_eq!(normal_inverse_sf(0.0), f64::INFINITY);
    assert_eq!(normal_inverse_sf(1.0), f64::NEG_INFINITY);
    assert_eq!(normal_cdf(f64::NEG_INFINITY), 0.0);
    assert_eq!(normal_cdf(f64::INFINITY), 1.0);
    assert_eq!(normal_sf(f64::NEG_INFINITY), 1.0);
    assert_eq!(normal_sf(f64::INFINITY), 0.0);
    for p in std::iter::once(0.0)
        .chain(p_grid())
        .chain(std::iter::once(1.0))
    {
        assert!(!normal_quantile(p).is_nan(), "quantile({p}) is NaN");
        assert!(!normal_inverse_sf(p).is_nan(), "inverse_sf({p}) is NaN");
    }
}

/// Scenario: Lognormal value moments are converted to the underlying normal
/// parameters for several (mean, stddev) pairs.
/// Guarantees: The closed-form lognormal moments of the returned (mu, sigma)
/// reproduce the requested mean and stddev, so annotations in value units
/// produce the intended distribution.
#[test]
fn lognormal_params_reproduce_moments() {
    for (mean_v, sd_v) in [
        (1.0, 1.0),
        (0.15, 0.3),
        (2048.0, 8192.0),
        (100.0, 1.0),
        (1e-3, 5e-3),
        (1.0, 1e-9),
    ] {
        let (mu, sigma) = lognormal_params(mean_v, sd_v);
        let s2 = sigma * sigma;
        let mean = (mu + s2 / 2.0).exp();
        // Use exp_m1 so exp(s2) - 1 keeps precision for tiny s2; otherwise the
        // reconstructed stddev for small variance ratios would itself round to
        // zero and the assertion would spuriously fail.
        let sd = (s2.exp_m1() * (2.0 * mu + s2).exp()).sqrt();
        assert!(
            rel_err(mean, mean_v) < 1e-12,
            "mean for ({mean_v}, {sd_v}): {mean}"
        );
        assert!(
            rel_err(sd, sd_v) < 1e-10,
            "stddev for ({mean_v}, {sd_v}): {sd}"
        );
    }
}

/// Scenario: A lognormal annotation requests a tiny but nonzero stddev
/// (mean 1, stddev 1e-9) whose variance ratio, added to 1.0, rounds back to
/// 1.0 under naive arithmetic.
/// Guarantees: The underlying sigma stays strictly positive and sampling over
/// the valid range does not collapse every draw to the mean, so small
/// requested variances are preserved rather than silently dropped.
#[test]
fn lognormal_tiny_variance_is_preserved() {
    let (_, sigma) = lognormal_params(1.0, 1e-9);
    assert!(sigma > 0.0, "sigma collapsed to {sigma}");

    let spec = parse_scalar(
        "range: { min: 0, max: 2 }\n\
         distribution: { type: lognormal, mean: 1, stddev: 1e-9 }",
        ScalarKind::Double,
    )
    .expect("tiny-variance lognormal is valid");
    let sampler = ValueSampler::new(&spec, false);
    let mut r = rng();
    let mut distinct = false;
    for _ in 0..1_000 {
        let x = sampler.sample_f64(&mut r);
        assert!((0.0..=2.0).contains(&x), "out of range: {x}");
        if (x - 1.0).abs() > 0.0 {
            distinct = true;
        }
    }
    assert!(distinct, "all draws collapsed to the mean");
}

/// Scenario: A lognormal annotation uses an extreme scale (mean and stddev
/// both 1e200) where squaring mean and stddev independently would overflow to
/// infinity and yield inf/inf == NaN.
/// Guarantees: The spec compiles into a finite sigma and is accepted by range
/// validation instead of being rejected with a spurious "~0 probability"
/// error.
#[test]
fn lognormal_extreme_scale_is_accepted() {
    let (mu, sigma) = lognormal_params(1e200, 1e200);
    assert!(mu.is_finite() && sigma.is_finite(), "mu {mu} sigma {sigma}");

    let _ = parse_scalar(
        "range: { min: 0, max: 1e300 }\n\
         distribution: { type: lognormal, mean: 1e200, stddev: 1e200 }",
        ScalarKind::Double,
    )
    .expect("extreme-scale lognormal is valid");
}

// ===========================================================================
// Sampling: range guarantees for every non-uniform method
// ===========================================================================

/// Scenario: Every non-uniform continuous distribution is sampled as doubles
/// over ranges that are centered, wide, in either tail, negative, or with the
/// mean outside the range.
/// Guarantees: All samples are finite and lie inside [min, max].
#[test]
fn continuous_samples_stay_in_range_doubles() {
    for (name, d, (lo, hi)) in non_uniform_cases() {
        let xs = draw(&number_sampler((lo, hi), d, false), 20_000);
        for x in &xs {
            assert!(x.is_finite(), "{name}: non-finite {x}");
            assert!((lo..=hi).contains(x), "{name}: {x} outside [{lo}, {hi}]");
        }
    }
}

/// Scenario: Every non-uniform continuous distribution is sampled as integers.
/// Guarantees: All samples are integral and lie inside [min, max].
#[test]
fn continuous_samples_stay_in_range_ints() {
    for (name, d, (lo, hi)) in non_uniform_cases() {
        let (lo, hi) = (lo.ceil(), hi.floor());
        let xs = draw(&number_sampler((lo, hi), d, true), 20_000);
        for x in &xs {
            assert_eq!(x.fract(), 0.0, "{name}: non-integer {x}");
            assert!((lo..=hi).contains(x), "{name}: {x} outside [{lo}, {hi}]");
        }
    }
}

/// Scenario: Zipf is sampled over several integer ranges and exponents, with
/// both the cumulative table and the rejection-inversion sampler.
/// Guarantees: Every sample is an integer in [min, max] and rank 1 (min) is
/// the most frequent value.
#[test]
fn zipf_samples_stay_in_range_and_favor_min() {
    for (lo, hi, s) in [
        (1.0, 10.0, 1.0),
        (1.0, 1000.0, 1.1),
        (100.0, 200.0, 2.0),
        (-5.0, 5.0, 0.5),
    ] {
        let range = Range { min: lo, max: hi };
        let table = NumberSampler::new(range, Distribution::Zipf { exponent: s }, true);
        let rejection = NumberSampler::new_zipf_rejection(range, s);
        for (name, sampler) in [("table", table), ("rejection", rejection)] {
            let xs = draw(&sampler, 20_000);
            let mut counts: HashMap<i64, usize> = HashMap::new();
            for x in &xs {
                assert_eq!(x.fract(), 0.0, "{name}: non-integer {x}");
                assert!((lo..=hi).contains(x), "{name}: {x} outside [{lo}, {hi}]");
                *counts.entry(*x as i64).or_default() += 1;
            }
            let (mode, _) = counts.iter().max_by_key(|(_, c)| **c).expect("non-empty");
            assert_eq!(*mode, lo as i64, "{name} zipf({s}) on [{lo}, {hi}]");
        }
    }
}

/// Scenario: A zipf range larger than `ZIPF_TABLE_CAP` is compiled and sampled.
/// Guarantees: The sampler falls back from the precomputed cumulative table to
/// the rejection-inversion sampler (so no multi-megabyte table is built) and
/// still yields integers inside the range with rank 1 (min) most frequent.
#[test]
fn zipf_above_table_cap_uses_rejection_and_stays_in_range() {
    let lo = 1.0;
    let hi = (ZIPF_TABLE_CAP + 1_000) as f64;
    let range = Range { min: lo, max: hi };
    let sampler = NumberSampler::new(range, Distribution::Zipf { exponent: 1.2 }, true);
    let xs = draw(&sampler, 20_000);
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for x in &xs {
        assert_eq!(x.fract(), 0.0, "non-integer {x}");
        assert!((lo..=hi).contains(x), "{x} outside [{lo}, {hi}]");
        *counts.entry(*x as i64).or_default() += 1;
    }
    let (mode, _) = counts.iter().max_by_key(|(_, c)| **c).expect("non-empty");
    assert_eq!(*mode, lo as i64, "min should be the most frequent rank");
}

/// Scenario: The integer sampler's widened inverse transform is used on a
/// small range.
/// Guarantees: Both endpoints are reachable and every integer in the range
/// appears, so rounding does not starve the bounds.
#[test]
fn integer_endpoints_are_reachable() {
    for d in [
        Distribution::Uniform,
        Distribution::Normal {
            mean: 3.0,
            stddev: 1.5,
        },
        Distribution::LogNormal {
            mean: 3.0,
            stddev: 2.0,
        },
        Distribution::Exponential { mean: 2.0 },
        Distribution::Zipf { exponent: 0.5 },
    ] {
        let xs = draw(&number_sampler((1.0, 5.0), d, true), 20_000);
        let seen: HashSet<i64> = xs.iter().map(|x| *x as i64).collect();
        assert_eq!(seen, (1..=5).collect(), "{d:?}");
    }
}

/// Scenario: Integer sampling is applied to a range with fractional bounds
/// (as can happen for nested specs constructed in code).
/// Guarantees: Samples are snapped inside the integer sub-range.
#[test]
fn integer_sampler_snaps_fractional_bounds() {
    let xs = draw(
        &number_sampler(
            (0.5, 3.5),
            Distribution::Normal {
                mean: 2.0,
                stddev: 5.0,
            },
            true,
        ),
        5_000,
    );
    assert!(xs.iter().all(|x| (1.0..=3.0).contains(x)));
}

/// Scenario: A degenerate range (min == max) is sampled with every
/// distribution type.
/// Guarantees: The constant value is always returned.
#[test]
fn degenerate_range_is_constant() {
    for d in [
        Distribution::Uniform,
        Distribution::Normal {
            mean: 0.0,
            stddev: 1.0,
        },
        Distribution::LogNormal {
            mean: 1.0,
            stddev: 1.0,
        },
        Distribution::Exponential { mean: 1.0 },
        Distribution::Zipf { exponent: 1.0 },
    ] {
        for integer in [false, true] {
            let xs = draw(&number_sampler((7.0, 7.0), d, integer), 100);
            assert!(xs.iter().all(|x| *x == 7.0), "{d:?} integer={integer}");
        }
    }
}

/// Scenario: A normal distribution is truncated to a range far in each tail.
/// Guarantees: Inverse transform keeps samples in range and spread across it
/// (no collapse onto a bound), with the mass piled toward the side nearest
/// the mean.
#[test]
fn far_tail_normal_is_spread_not_clamped() {
    let d = Distribution::Normal {
        mean: 0.0,
        stddev: 1.0,
    };
    let right = draw(&number_sampler((8.0, 10.0), d, false), 20_000);
    assert!(right.iter().all(|x| (8.0..=10.0).contains(x)));
    assert!(
        quantile(&right, 0.5) < 8.2,
        "median should hug the near bound"
    );
    assert!(
        quantile(&right, 0.99) > 8.3,
        "upper tail should not collapse"
    );
    let distinct: HashSet<u64> = right.iter().map(|x| x.to_bits()).collect();
    assert!(
        distinct.len() > 19_000,
        "values must not pile up on a bound"
    );

    let left = draw(&number_sampler((-10.0, -8.0), d, false), 20_000);
    assert!(left.iter().all(|x| (-10.0..=-8.0).contains(x)));
    assert!(quantile(&left, 0.5) > -8.2);
}

// ===========================================================================
// Sampling: distribution shape
// ===========================================================================

/// Scenario: Uniform doubles are sampled over [10, 20].
/// Guarantees: Mean and quartiles match the uniform distribution.
#[test]
fn uniform_shape() {
    let xs = draw(
        &number_sampler((10.0, 20.0), Distribution::Uniform, false),
        50_000,
    );
    assert!((mean(&xs) - 15.0).abs() < 0.05);
    assert!((quantile(&xs, 0.25) - 12.5).abs() < 0.1);
    assert!((quantile(&xs, 0.75) - 17.5).abs() < 0.1);
}

/// Scenario: A normal distribution is sampled over a range that contains
/// essentially all of its mass.
/// Guarantees: Sample mean, stddev and median match the parameters.
#[test]
fn normal_shape_untruncated() {
    let xs = draw(
        &number_sampler(
            (0.0, 100.0),
            Distribution::Normal {
                mean: 50.0,
                stddev: 10.0,
            },
            false,
        ),
        50_000,
    );
    let m = mean(&xs);
    let sd = (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64).sqrt();
    assert!((m - 50.0).abs() < 0.2, "mean {m}");
    assert!((sd - 10.0).abs() < 0.2, "stddev {sd}");
    assert!((quantile(&xs, 0.5) - 50.0).abs() < 0.3);
    assert!(
        (quantile(&xs, 0.8413) - 60.0).abs() < 0.4,
        "one sigma quantile"
    );
}

/// Scenario: A normal distribution is truncated to [mean, +inf-ish).
/// Guarantees: The sample mean equals the half-normal mean
/// (mean + stddev * sqrt(2/pi)), confirming truncation is exact.
#[test]
fn normal_shape_half_truncated() {
    let xs = draw(
        &number_sampler(
            (0.0, 100.0),
            Distribution::Normal {
                mean: 0.0,
                stddev: 10.0,
            },
            false,
        ),
        50_000,
    );
    let expected = 10.0 * (2.0 / std::f64::consts::PI).sqrt();
    assert!((mean(&xs) - expected).abs() < 0.1, "mean {}", mean(&xs));
}

/// Scenario: A lognormal distribution is sampled over a wide range.
/// Guarantees: Mean and median match the value-space parameters
/// (median = exp(mu)).
#[test]
fn lognormal_shape() {
    let (mean_v, sd_v) = (100.0, 50.0);
    let xs = draw(
        &number_sampler(
            (0.0, 1e7),
            Distribution::LogNormal {
                mean: mean_v,
                stddev: sd_v,
            },
            false,
        ),
        50_000,
    );
    let (mu, _) = lognormal_params(mean_v, sd_v);
    assert!(
        (mean(&xs) - mean_v).abs() / mean_v < 0.02,
        "mean {}",
        mean(&xs)
    );
    assert!((quantile(&xs, 0.5) - mu.exp()).abs() / mu.exp() < 0.02);
}

/// Scenario: An exponential distribution is sampled untruncated and
/// truncated.
/// Guarantees: The untruncated mean matches; the truncated distribution keeps
/// the memoryless shape (min + mean when shifted far into the tail).
#[test]
fn exponential_shape() {
    let xs = draw(
        &number_sampler((0.0, 1e6), Distribution::Exponential { mean: 5.0 }, false),
        50_000,
    );
    assert!((mean(&xs) - 5.0).abs() < 0.1, "mean {}", mean(&xs));
    assert!((quantile(&xs, 0.5) - 5.0 * 2f64.ln()).abs() < 0.1);

    let tail = draw(
        &number_sampler((100.0, 1e6), Distribution::Exponential { mean: 5.0 }, false),
        50_000,
    );
    assert!(
        (mean(&tail) - 105.0).abs() < 0.15,
        "tail mean {}",
        mean(&tail)
    );
}

/// Scenario: Zipf is sampled over [1, 10] with exponent 1.
/// Guarantees: Rank frequencies follow 1/k normalized by the harmonic number.
#[test]
fn zipf_shape() {
    let xs = draw(
        &number_sampler((1.0, 10.0), Distribution::Zipf { exponent: 1.0 }, true),
        100_000,
    );
    let h: f64 = (1..=10).map(|k| 1.0 / f64::from(k)).sum();
    for k in 1..=10 {
        let observed = xs.iter().filter(|x| **x == f64::from(k)).count() as f64 / xs.len() as f64;
        let expected = 1.0 / (f64::from(k) * h);
        assert!(
            (observed - expected).abs() < 0.01,
            "rank {k}: {observed} vs {expected}"
        );
    }
}

/// Scenario: Weighted and unweighted discrete specs are sampled.
/// Guarantees: Proportions match the weights (uniform when omitted) and
/// zero-weight values are never chosen.
#[test]
fn discrete_proportions() {
    let mut r = rng();
    let weighted = IndexSampler::new(4, Some(&[70.0, 20.0, 10.0, 0.0]));
    let mut counts = [0usize; 4];
    for _ in 0..100_000 {
        counts[weighted.sample(&mut r)] += 1;
    }
    for (i, w) in [0.7, 0.2, 0.1, 0.0].iter().enumerate() {
        let p = counts[i] as f64 / 100_000.0;
        assert!((p - w).abs() < 0.01, "index {i}: {p} vs {w}");
    }
    assert_eq!(counts[3], 0, "zero weight must never be selected");

    let uniform = IndexSampler::new(5, None);
    let mut counts = [0usize; 5];
    for _ in 0..100_000 {
        counts[uniform.sample(&mut r)] += 1;
    }
    for c in counts {
        assert!((c as f64 / 100_000.0 - 0.2).abs() < 0.01);
    }
}

/// Scenario: Two samplers are driven by identically seeded RNGs.
/// Guarantees: Sampling is reproducible for a given seed.
#[test]
fn sampling_is_reproducible_for_a_seed() {
    let s = number_sampler(
        (0.0, 10.0),
        Distribution::LogNormal {
            mean: 2.0,
            stddev: 1.0,
        },
        false,
    );
    assert_eq!(draw(&s, 1_000), draw(&s, 1_000));
}

/// Scenario: `ValueSampler::sample_value` and `ValueSampler::sample_count`
/// are called on samplers compiled from continuous and discrete specs.
/// Guarantees: Continuous int samplers yield `Scalar::Int`, double samplers
/// yield `Scalar::Double`, discrete values are returned as written, and
/// `sample_count` never goes negative.
#[test]
fn sample_value_output_types() {
    let spec = ValueSpec::Continuous {
        range: Range { min: 0.0, max: 9.0 },
        distribution: Distribution::Uniform,
    };
    let mut r = rng();
    assert!(matches!(
        ValueSampler::new(&spec, true).sample_value(&mut r),
        Scalar::Int(_)
    ));
    assert!(matches!(
        ValueSampler::new(&spec, false).sample_value(&mut r),
        Scalar::Double(_)
    ));
    let neg = ValueSampler::new(
        &ValueSpec::Discrete {
            values: vec![Scalar::Int(-3)],
            weights: None,
        },
        true,
    );
    assert_eq!(neg.sample_value(&mut r), Scalar::Int(-3));
    assert_eq!(neg.sample_count(&mut r), 0);
}

// ===========================================================================
