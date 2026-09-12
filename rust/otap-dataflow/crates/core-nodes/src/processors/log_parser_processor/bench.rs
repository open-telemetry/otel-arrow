use super::*;

/// Benchmark-only access to compiled record parsers.
#[cfg(feature = "bench")]
pub struct BenchLogParser(parse_logs::Parser);

#[cfg(feature = "bench")]
impl BenchLogParser {
    /// Compile a parsing configuration before measurement begins.
    #[must_use]
    pub fn new(config: Value) -> Self {
        Self(
            parse_logs::Parser::new(serde_json::from_value(config).expect("benchmark config"))
                .expect("benchmark parser"),
        )
    }

    /// Parse one record and discard its candidates after measurement.
    #[must_use]
    pub fn parse(&self, input: &str) -> bool {
        self.0.parse(input, 0, 3_000_000_000).is_ok()
    }

    /// Decode, stage and sanitize a native OTAP batch, returning preserved-error counts.
    pub fn apply_batch(
        &self,
        mut batch: OtapArrowRecords,
    ) -> Result<(OtapArrowRecords, u64), otel_arrow_dfe_pdata::error::Error> {
        batch.decode_transport_optimized_ids()?;
        let (mut batch, counts) = self.0.apply(batch)?;
        sanitize_otap_batch(&mut batch);
        let errors = counts
            .values()
            .filter(|(reason, _)| *reason != parse_logs::DataError::ObservedFallback)
            .map(|(_, count)| count)
            .sum();
        Ok((batch, errors))
    }

    /// Return the pre-allocation reservation for one input.
    #[must_use]
    pub fn scratch_bound(&self, input: &str) -> usize {
        self.0
            .scratch_bound(input)
            .expect("benchmark scratch bound")
    }
}
