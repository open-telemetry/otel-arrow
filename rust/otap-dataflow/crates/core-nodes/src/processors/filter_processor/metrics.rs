// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP FilterProcessor node.
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP FilterProcessor
#[metric_set(name = "processor.filter.pdata")]
#[derive(Debug, Default, Clone)]
pub struct FilterPdataMetrics {
    /// Number of log batches received by the filter processor.
    ///
    /// Counted on receipt, before conversion/decode, so batches that later
    /// fail conversion or decode are still included.
    #[metric(unit = "{batch}")]
    pub log_batches_seen: Counter<u64>,
    /// Number of metric batches received by the filter processor.
    ///
    /// Counted on receipt, before conversion/decode, so batches that later
    /// fail conversion or decode are still included.
    #[metric(unit = "{batch}")]
    pub metric_batches_seen: Counter<u64>,
    /// Number of span batches received by the filter processor.
    ///
    /// Counted on receipt, before conversion/decode, so batches that later
    /// fail conversion or decode are still included.
    #[metric(unit = "{batch}")]
    pub span_batches_seen: Counter<u64>,

    /// Number of log batches received while an include filter was configured.
    ///
    /// Counted on receipt alongside `log_batches_seen`, before conversion/decode
    /// and filtering. It does not imply filtering ran successfully or that any
    /// records matched the include rule.
    #[metric(unit = "{batch}")]
    pub log_include_configured_batches: Counter<u64>,
    /// Number of metric batches received while an include filter was configured.
    ///
    /// Counted on receipt alongside `metric_batches_seen`, before conversion/decode
    /// and filtering. It does not imply filtering ran successfully or that any
    /// records matched the include rule.
    #[metric(unit = "{batch}")]
    pub metric_include_configured_batches: Counter<u64>,
    /// Number of span batches received while an include filter was configured.
    ///
    /// Counted on receipt alongside `span_batches_seen`, before conversion/decode
    /// and filtering. It does not imply filtering ran successfully or that any
    /// records matched the include rule.
    #[metric(unit = "{batch}")]
    pub span_include_configured_batches: Counter<u64>,

    /// Number of log batches received while an exclude filter was configured.
    ///
    /// Counted on receipt alongside `log_batches_seen`, before conversion/decode
    /// and filtering. It does not imply filtering ran successfully or that any
    /// records matched the exclude rule.
    #[metric(unit = "{batch}")]
    pub log_exclude_configured_batches: Counter<u64>,
    /// Number of metric batches received while an exclude filter was configured.
    ///
    /// Counted on receipt alongside `metric_batches_seen`, before conversion/decode
    /// and filtering. It does not imply filtering ran successfully or that any
    /// records matched the exclude rule.
    #[metric(unit = "{batch}")]
    pub metric_exclude_configured_batches: Counter<u64>,
    /// Number of span batches received while an exclude filter was configured.
    ///
    /// Counted on receipt alongside `span_batches_seen`, before conversion/decode
    /// and filtering. It does not imply filtering ran successfully or that any
    /// records matched the exclude rule.
    #[metric(unit = "{batch}")]
    pub span_exclude_configured_batches: Counter<u64>,

    /// Number of log signals consumed
    #[metric(unit = "{log}")]
    pub log_signals_consumed: Counter<u64>,
    /// Number of metric signals consumed
    #[metric(unit = "{metric}")]
    pub metric_signals_consumed: Counter<u64>,
    /// Number of span signals consumed
    #[metric(unit = "{span}")]
    pub span_signals_consumed: Counter<u64>,

    /// Number of log signals kept (consumed minus filtered)
    #[metric(unit = "{log}")]
    pub log_signals_kept: Counter<u64>,
    /// Number of metric signals kept (consumed minus filtered)
    #[metric(unit = "{metric}")]
    pub metric_signals_kept: Counter<u64>,
    /// Number of span signals kept (consumed minus filtered)
    #[metric(unit = "{span}")]
    pub span_signals_kept: Counter<u64>,

    /// Number of log signals filtered
    #[metric(unit = "{log}")]
    pub log_signals_filtered: Counter<u64>,
    /// Number of metric signals filtered
    #[metric(unit = "{metric}")]
    pub metric_signals_filtered: Counter<u64>,
    /// Number of span signals filtered
    #[metric(unit = "{span}")]
    pub span_signals_filtered: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_default_is_zero() {
        let m = FilterPdataMetrics::default();
        assert_eq!(m.log_batches_seen.get(), 0);
        assert_eq!(m.log_include_configured_batches.get(), 0);
        assert_eq!(m.log_exclude_configured_batches.get(), 0);
        assert_eq!(m.log_signals_kept.get(), 0);
        assert_eq!(m.log_signals_consumed.get(), 0);
        assert_eq!(m.log_signals_filtered.get(), 0);
    }

    #[test]
    fn test_metrics_add_and_inc() {
        let mut m = FilterPdataMetrics::default();
        m.span_batches_seen.inc();
        m.span_include_configured_batches.inc();
        m.span_signals_consumed.add(10);
        m.span_signals_filtered.add(4);
        m.span_signals_kept.add(6);

        assert_eq!(m.span_batches_seen.get(), 1);
        assert_eq!(m.span_include_configured_batches.get(), 1);
        assert_eq!(m.span_exclude_configured_batches.get(), 0);
        assert_eq!(m.span_signals_consumed.get(), 10);
        assert_eq!(m.span_signals_filtered.get(), 4);
        assert_eq!(m.span_signals_kept.get(), 6);
    }
}
