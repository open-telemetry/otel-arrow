// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Samplers compiled from `traffic-generator` value specs.
//!
//! Continuous specs are sampled with inverse transform sampling over the
//! truncated distribution: for a CDF `F` restricted to `[a, b]`,
//! `x = F^-1(F(a) + u * (F(b) - F(a)))` with `u ~ Uniform(0, 1)`. Every sample
//! therefore lies in the range with exactly one uniform draw, no rejection loop
//! and no clamping spike at the bounds. When the range lies in the right tail
//! the survival function `1 - F` is used instead so the tail keeps precision.
//!
//! The standard normal functions ([`normal_cdf`], [`normal_sf`],
//! [`normal_quantile`], [`normal_inverse_sf`]) are written on top of
//! `libm::erfc`, with Acklam's rational approximation for the quantile refined
//! by one Halley step. Lognormal sampling is derived from them; exponential
//! sampling uses closed forms.
//!
//! [`normal_cdf`]: crate::receivers::traffic_generator::sampler::normal_cdf
//! [`normal_sf`]: crate::receivers::traffic_generator::sampler::normal_sf
//! [`normal_quantile`]: crate::receivers::traffic_generator::sampler::normal_quantile
//! [`normal_inverse_sf`]: crate::receivers::traffic_generator::sampler::normal_inverse_sf

use rand::{Rng, RngExt};
use rand_distr::Distribution as _;

use super::annotation::{Distribution, Range, Scalar, ValueSpec};

/// Largest integer range for which zipf uses a precomputed cumulative table.
/// Larger ranges fall back to `rand_distr::Zipf` (rejection-inversion).
pub const ZIPF_TABLE_CAP: u64 = 1 << 20;

// ---------------------------------------------------------------------------
// Standard normal functions
// ---------------------------------------------------------------------------

const SQRT_2: f64 = std::f64::consts::SQRT_2;
const SQRT_2PI: f64 = 2.506_628_274_631_000_5;

/// `P[Z <= z]` for a standard normal `Z`.
#[must_use]
pub fn normal_cdf(z: f64) -> f64 {
    0.5 * libm::erfc(-z / SQRT_2)
}

/// `P[Z > z]` for a standard normal `Z`, computed directly (not `1 - cdf`) so
/// the upper tail keeps full precision.
#[must_use]
pub fn normal_sf(z: f64) -> f64 {
    0.5 * libm::erfc(z / SQRT_2)
}

/// Inverse of [`normal_cdf`]: the `z` with `P[Z <= z] = p`.
///
/// Returns `-inf` for `p <= 0` and `+inf` for `p >= 1`.
#[must_use]
pub fn normal_quantile(p: f64) -> f64 {
    if p >= 1.0 {
        f64::INFINITY
    } else if p > 0.5 {
        -lower_tail_quantile(1.0 - p)
    } else {
        lower_tail_quantile(p)
    }
}

/// Inverse of [`normal_sf`]: the `z` with `P[Z > z] = q`.
///
/// Returns `+inf` for `q <= 0` and `-inf` for `q >= 1`.
#[must_use]
pub fn normal_inverse_sf(q: f64) -> f64 {
    if q >= 1.0 {
        f64::NEG_INFINITY
    } else if q > 0.5 {
        lower_tail_quantile(1.0 - q)
    } else {
        -lower_tail_quantile(q)
    }
}

/// Quantile for `p` in `(0, 0.5]`, where the lower-tail CDF is precise.
///
/// Acklam's rational approximation (relative error ~1e-9) followed by one
/// Halley refinement step against [`normal_cdf`].
fn lower_tail_quantile(p: f64) -> f64 {
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

    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    let x = if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    };
    // One Halley refinement step against the precise lower-tail CDF.
    let e = normal_cdf(x) - p;
    let u = e * SQRT_2PI * (x * x / 2.0).exp();
    if u.is_finite() {
        x - u / (1.0 + x * u / 2.0)
    } else {
        x
    }
}

// ---------------------------------------------------------------------------
// Truncated continuous sampling
// ---------------------------------------------------------------------------

/// Precomputed truncation of a standard normal to `[za, zb]`.
#[derive(Debug, Clone, Copy)]
struct NormalTruncation {
    /// Sample through the survival function (range in the right half).
    upper: bool,
    /// `cdf(za)` or `sf(za)` depending on `upper`.
    pa: f64,
    /// `cdf(zb)` or `sf(zb)` depending on `upper`.
    pb: f64,
}

impl NormalTruncation {
    fn new(za: f64, zb: f64) -> Self {
        // When the range starts above the median, work in survival space so
        // probabilities near 1 do not lose precision.
        if za > 0.0 {
            Self {
                upper: true,
                pa: normal_sf(za),
                pb: normal_sf(zb),
            }
        } else {
            Self {
                upper: false,
                pa: normal_cdf(za),
                pb: normal_cdf(zb),
            }
        }
    }

    fn mass(&self) -> f64 {
        (self.pb - self.pa).abs()
    }

    fn sample_z(&self, u: f64) -> f64 {
        let p = self.pa + u * (self.pb - self.pa);
        if self.upper {
            normal_inverse_sf(p)
        } else {
            normal_quantile(p)
        }
    }
}

/// Compiled continuous distribution over an inclusive range.
#[derive(Debug, Clone)]
enum Continuous {
    Uniform,
    Normal {
        mean: f64,
        stddev: f64,
        t: NormalTruncation,
    },
    LogNormal {
        mu: f64,
        sigma: f64,
        t: NormalTruncation,
    },
    Exponential {
        mean: f64,
        /// Lower bound clipped to the support (`>= 0`).
        lo: f64,
        /// `expm1(-(hi - lo) / mean)`, negative.
        span: f64,
    },
    ZipfTable {
        cumulative: Vec<f64>,
    },
    ZipfRejection {
        zipf: rand_distr::Zipf<f64>,
    },
}

/// Convert lognormal value moments to the underlying normal parameters.
#[must_use]
pub fn lognormal_params(mean: f64, stddev: f64) -> (f64, f64) {
    let sigma2 = (1.0 + (stddev * stddev) / (mean * mean)).ln();
    let mu = mean.ln() - sigma2 / 2.0;
    (mu, sigma2.sqrt())
}

/// Sampler for a continuous value spec.
#[derive(Debug, Clone)]
pub struct NumberSampler {
    lo: f64,
    hi: f64,
    /// Original inclusive range (for clamping and integer endpoints).
    range: Range,
    integer: bool,
    dist: Continuous,
}

impl NumberSampler {
    /// Compile a sampler for `range` and `distribution`.
    ///
    /// For integer output the continuous distribution is sampled over
    /// `[min - 0.5, max + 0.5]` and rounded, so each integer `k` receives the
    /// mass of `[k - 0.5, k + 0.5]` and both endpoints are reachable.
    #[must_use]
    pub fn new(range: Range, distribution: Distribution, integer: bool) -> Self {
        // Integer output can only take integer values inside the range.
        let range = if integer {
            let (lo, hi) = (range.min.ceil(), range.max.floor());
            if lo <= hi {
                Range { min: lo, max: hi }
            } else {
                let mid = range.min.round();
                Range { min: mid, max: mid }
            }
        } else {
            range
        };
        let widen = integer
            && !matches!(
                distribution,
                Distribution::Uniform | Distribution::Zipf { .. }
            );
        let (lo, hi) = if widen {
            (range.min - 0.5, range.max + 0.5)
        } else {
            (range.min, range.max)
        };
        let distribution = if range.min == range.max {
            // A degenerate range is a constant regardless of distribution.
            Distribution::Uniform
        } else {
            distribution
        };
        let dist = match distribution {
            Distribution::Uniform => Continuous::Uniform,
            Distribution::Normal { mean, stddev } => Continuous::Normal {
                mean,
                stddev,
                t: NormalTruncation::new((lo - mean) / stddev, (hi - mean) / stddev),
            },
            Distribution::LogNormal { mean, stddev } => {
                let (mu, sigma) = lognormal_params(mean, stddev);
                let za = if lo <= 0.0 {
                    f64::NEG_INFINITY
                } else {
                    (lo.ln() - mu) / sigma
                };
                let zb = (hi.ln() - mu) / sigma;
                Continuous::LogNormal {
                    mu,
                    sigma,
                    t: NormalTruncation::new(za, zb),
                }
            }
            Distribution::Exponential { mean } => {
                let lo_s = lo.max(0.0);
                Continuous::Exponential {
                    mean,
                    lo: lo_s,
                    span: (-(hi - lo_s) / mean).exp_m1(),
                }
            }
            Distribution::Zipf { exponent } => {
                // Int range bounds are validated to +/-2^53, so the rank
                // count fits in u64 without overflow.
                let n = zipf_rank_count(&range);
                if n <= ZIPF_TABLE_CAP {
                    Continuous::ZipfTable {
                        cumulative: zipf_table(n, exponent),
                    }
                } else {
                    Continuous::ZipfRejection {
                        zipf: rand_distr::Zipf::new(n as f64, exponent)
                            .expect("zipf parameters validated at parse time"),
                    }
                }
            }
        };
        Self {
            lo,
            hi,
            range,
            integer,
            dist,
        }
    }

    /// Force the rejection-inversion zipf sampler regardless of range size.
    #[cfg(test)]
    pub(crate) fn new_zipf_rejection(range: Range, exponent: f64) -> Self {
        let n = zipf_rank_count(&range);
        Self {
            lo: range.min,
            hi: range.max,
            range,
            integer: true,
            dist: Continuous::ZipfRejection {
                zipf: rand_distr::Zipf::new(n as f64, exponent).expect("valid zipf"),
            },
        }
    }

    /// Whether samples are integral.
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        self.integer
    }

    /// Probability mass of the (possibly widened) range under the distribution.
    #[must_use]
    pub fn mass(&self) -> f64 {
        match &self.dist {
            Continuous::Normal { t, .. } | Continuous::LogNormal { t, .. } => t.mass(),
            // The memoryless inverse transform is exact anywhere in the tail,
            // so report the conditional mass (never underflows for hi > lo).
            Continuous::Exponential { span, .. } => -span,
            _ => 1.0,
        }
    }

    /// Draw a value as `f64` (integral when the sampler is integer).
    pub fn sample_f64<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let x = match &self.dist {
            Continuous::Uniform => {
                if self.integer {
                    return rng.random_range(self.range.min as i64..=self.range.max as i64) as f64;
                }
                if self.lo == self.hi {
                    self.lo
                } else {
                    // Interpolate instead of `lo + u * (hi - lo)`: the width
                    // overflows to infinity for ranges wider than f64::MAX.
                    let u = rng.random::<f64>();
                    self.lo * (1.0 - u) + self.hi * u
                }
            }
            Continuous::Normal { mean, stddev, t } => {
                mean + stddev * t.sample_z(rng.random::<f64>())
            }
            Continuous::LogNormal { mu, sigma, t } => {
                (mu + sigma * t.sample_z(rng.random::<f64>())).exp()
            }
            Continuous::Exponential { mean, lo, span } => {
                // Memoryless inverse transform of exp truncated to [lo, hi].
                lo - mean * (rng.random::<f64>() * span).ln_1p()
            }
            Continuous::ZipfTable { cumulative } => {
                let total = *cumulative.last().unwrap_or(&1.0);
                let target = rng.random::<f64>() * total;
                let idx = cumulative.partition_point(|c| *c <= target);
                return self.range.min + idx.min(cumulative.len() - 1) as f64;
            }
            Continuous::ZipfRejection { zipf } => {
                return self.range.min + zipf.sample(rng) - 1.0;
            }
        };
        let x = if x.is_nan() { self.lo } else { x };
        if self.integer {
            x.round().clamp(self.range.min, self.range.max)
        } else {
            x.clamp(self.range.min, self.range.max)
        }
    }
}

/// Number of integer ranks in an inclusive int range.
fn zipf_rank_count(range: &Range) -> u64 {
    debug_assert!(range.min <= range.max);
    ((range.max - range.min) as u64).saturating_add(1)
}

fn zipf_table(n: u64, exponent: f64) -> Vec<f64> {
    let mut acc = 0.0;
    (1..=n)
        .map(|k| {
            acc += (k as f64).powf(-exponent);
            acc
        })
        .collect()
}

/// Probability mass carried by `range` under `distribution` (1 for uniform and zipf).
///
/// Used at parse time to reject ranges that are effectively unreachable.
#[must_use]
pub fn range_mass(range: &Range, distribution: &Distribution, integer: bool) -> f64 {
    match distribution {
        Distribution::Uniform | Distribution::Zipf { .. } => 1.0,
        _ if range.min == range.max => 1.0,
        _ => NumberSampler::new(*range, *distribution, integer).mass(),
    }
}

// ---------------------------------------------------------------------------
// Discrete sampling
// ---------------------------------------------------------------------------

/// Sampler choosing an index into a list, uniformly or by weight.
#[derive(Debug, Clone)]
pub struct IndexSampler {
    len: usize,
    weighted: Option<Weighted>,
}

/// Cumulative weights and the last index with a positive weight.
#[derive(Debug, Clone)]
struct Weighted {
    cumulative: Vec<f64>,
    last_positive: usize,
}

impl IndexSampler {
    /// Build a sampler over `len` items with optional relative weights.
    ///
    /// Weights are validated at parse time: non-negative, finite sum, and
    /// at least one positive weight.
    #[must_use]
    pub fn new(len: usize, weights: Option<&[f64]>) -> Self {
        let weighted = weights.map(|w| {
            let mut acc = 0.0;
            let cumulative = w
                .iter()
                .map(|x| {
                    acc += x;
                    acc
                })
                .collect();
            let last_positive = w.iter().rposition(|x| *x > 0.0).unwrap_or(0);
            Weighted {
                cumulative,
                last_positive,
            }
        });
        Self { len, weighted }
    }

    /// Draw an index in `0..len`. Zero-weight items are never selected.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        match &self.weighted {
            None => rng.random_range(0..self.len),
            Some(w) => {
                let total = w.cumulative.last().copied().unwrap_or(0.0);
                let target = rng.random::<f64>() * total;
                // First index whose cumulative weight exceeds target. Rounding
                // can leave target at or above the last cumulative value, so
                // clamp to the last positive weight rather than the last item
                // (which may have weight zero).
                w.cumulative
                    .partition_point(|x| *x <= target)
                    .min(w.last_positive)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Value sampling
// ---------------------------------------------------------------------------

/// A value spec compiled once into a reusable sampler.
///
/// Building a sampler can be expensive (truncated normal / lognormal bounds
/// evaluate `erfc`, zipf builds a cumulative table over the whole range), so
/// the generator compiles every spec once when the registry is loaded and
/// draws from the compiled form for each value.
#[derive(Debug, Clone)]
pub enum ValueSampler {
    /// Pick one of `values` (weighted when the spec has `weights`).
    Discrete {
        /// Candidate values.
        values: Vec<Scalar>,
        /// Index sampler over `values`.
        index: IndexSampler,
    },
    /// Sample a number from a compiled continuous distribution.
    Continuous(NumberSampler),
}

impl ValueSampler {
    /// Compile `spec`; `integer` selects integer output for continuous specs.
    #[must_use]
    pub fn new(spec: &ValueSpec, integer: bool) -> Self {
        match spec {
            ValueSpec::Discrete { values, weights } => ValueSampler::Discrete {
                values: values.clone(),
                index: IndexSampler::new(values.len(), weights.as_deref()),
            },
            ValueSpec::Continuous {
                range,
                distribution,
            } => ValueSampler::Continuous(NumberSampler::new(*range, *distribution, integer)),
        }
    }

    /// Draw one value.
    ///
    /// Discrete samplers pick one of `values` (weighted when `weights` is
    /// present, uniform otherwise). Continuous samplers sample the range with
    /// inverse transform sampling.
    #[cfg(test)]
    pub(crate) fn sample_value<R: Rng + ?Sized>(&self, rng: &mut R) -> Scalar {
        match self {
            ValueSampler::Discrete { values, index } => values[index.sample(rng)].clone(),
            ValueSampler::Continuous(sampler) => {
                let x = sampler.sample_f64(rng);
                if sampler.is_integer() {
                    Scalar::Int(x as i64)
                } else {
                    Scalar::Double(x)
                }
            }
        }
    }

    /// Draw a number (strings map to 0, booleans to 0 / 1).
    pub fn sample_f64<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match self {
            ValueSampler::Discrete { values, index } => scalar_f64(&values[index.sample(rng)]),
            // Integer samplers already return integral values.
            ValueSampler::Continuous(sampler) => sampler.sample_f64(rng),
        }
    }

    /// Draw a non-negative count.
    pub fn sample_count<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let v = self.sample_f64(rng).round();
        if v <= 0.0 { 0 } else { v as usize }
    }
}

fn scalar_f64(value: &Scalar) -> f64 {
    match value {
        Scalar::Int(i) => *i as f64,
        Scalar::Double(d) => *d,
        Scalar::Bool(b) => f64::from(u8::from(*b)),
        Scalar::String(_) => 0.0,
    }
}

/// Sampler that maps each discrete value of a spec to a precomputed output.
///
/// Used where the generator converts a sampled value into something else
/// (status codes, severity numbers, attribute `AnyValue`s, template key
/// names), so the conversion happens once at compile time instead of on
/// every draw.
#[derive(Debug, Clone)]
pub struct MappedSampler<T> {
    outputs: Vec<T>,
    index: IndexSampler,
}

impl<T> MappedSampler<T> {
    /// Compile `spec`, mapping each discrete value with `map`.
    ///
    /// Returns `None` for continuous specs or when `map` rejects a value.
    pub fn new(spec: &ValueSpec, map: impl FnMut(&Scalar) -> Option<T>) -> Option<Self> {
        let ValueSpec::Discrete { values, weights } = spec else {
            return None;
        };
        let outputs = values.iter().map(map).collect::<Option<Vec<_>>>()?;
        Some(Self {
            index: IndexSampler::new(outputs.len(), weights.as_deref()),
            outputs,
        })
    }

    /// Draw one precomputed output.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> &T {
        &self.outputs[self.index.sample(rng)]
    }
}
