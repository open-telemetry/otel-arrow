// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport header capture.
//!
//! Applies the configured [`HeaderCapturePolicy`] to copy Kafka message headers
//! into [`TransportHeaders`] on the [`OtapPdata`] context. This is independent
//! of the `resource_attrs_from_headers` mechanism that injects headers into
//! resource attributes.

use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use otel_arrow_dfe_config::transport_headers_policy::HeaderCapturePolicy;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use rdkafka::Message as _;
use rdkafka::message::{BorrowedMessage, Headers};

/// Apply the capture policy (if configured) to extract Kafka message headers
/// into [`TransportHeaders`] on the [`OtapPdata`] context.
///
/// This is independent of the `resource_attrs_from_headers` mechanism which injects
/// headers into resource attributes.
pub(super) fn capture_transport_headers(
    kafka_message: &BorrowedMessage<'_>,
    capture_policy: Option<&HeaderCapturePolicy>,
    pdata: &mut OtapPdata,
) {
    if let Some(policy) = capture_policy
        && let Some(headers) = kafka_message.headers()
    {
        let pairs = headers.iter().filter_map(|h| h.value.map(|v| (h.key, v)));
        let mut transport_headers = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs, &mut transport_headers);
        if let Some(stats) = stats {
            otel_error!(
                "kafka.capture_policy.limits_exceeded",
                stats = %stats,
            );
        }
        if !transport_headers.is_empty() {
            pdata.set_transport_headers(transport_headers);
        }
    }
}
