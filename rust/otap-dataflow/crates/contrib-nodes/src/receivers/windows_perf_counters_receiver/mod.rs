// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows performance-counter receiver.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = runtime::WINDOWS_PERF_COUNTERS_RECEIVER_URN,
    target = "otel.receiver.windows_perf_counters",
);

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod config;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod metrics;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod otap_builder;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod pdh;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod runtime;
#[cfg(test)]
mod test_utils {
    use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_pdata_codec::OtapPayload;
    use prost::Message;

    pub(super) fn decode(records: OtapArrowRecords) -> ExportMetricsServiceRequest {
        let bytes: OtlpProtoBytes = OtapPayload::from_otap(records)
            .try_into_with_default()
            .unwrap();
        ExportMetricsServiceRequest::decode(bytes.as_bytes()).unwrap()
    }
}
