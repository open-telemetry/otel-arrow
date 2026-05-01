use std::collections::HashMap;

use crate::clickhouse_exporter::SUPPORTED_ARROW_PAYLOAD_TYPES;
use crate::clickhouse_exporter::config::Config;
use crate::clickhouse_exporter::transform::transform_plan::TransformationPlan;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

mod transform_attributes;
pub(crate) mod transform_batch;
mod transform_column;
pub(crate) mod transform_plan;

/// Build a map of supported arrow payload types to static Transformation Plans according to configuration.
pub fn build_payload_transform_map(
    config: &Config,
) -> HashMap<ArrowPayloadType, TransformationPlan> {
    SUPPORTED_ARROW_PAYLOAD_TYPES
        .iter()
        .copied()
        .map(|pt| (pt, TransformationPlan::from_config(&pt, config)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn build_payload_transform_map_has_all_supported_keys_and_expected_values() {
        let config = Config::default();

        let m = build_payload_transform_map(&config);

        // 1) keys match SUPPORTED_ARROW_PAYLOAD_TYPES exactly
        assert_eq!(m.len(), SUPPORTED_ARROW_PAYLOAD_TYPES.len());

        let expected: HashSet<_> = SUPPORTED_ARROW_PAYLOAD_TYPES.iter().copied().collect();
        let actual: HashSet<_> = m.keys().copied().collect();
        assert_eq!(actual, expected);

        // 2) each plan matches TransformationPlan::from_config
        for &pt in SUPPORTED_ARROW_PAYLOAD_TYPES.iter() {
            let expected_plan = TransformationPlan::from_config(&pt, &config);
            let actual_plan = m.get(&pt).expect("missing payload type");
            assert_eq!(actual_plan, &expected_plan);
        }
    }
}
