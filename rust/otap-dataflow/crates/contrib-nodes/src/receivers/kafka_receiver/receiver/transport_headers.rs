// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Arrival context formation.
//!
//! Streams Kafka message headers into the compiled [`ArrivalBinding`], which
//! captures the policy-selected headers and forms the [`OtapPdata`] context.
//! This is independent of the `resource_attrs_from_headers` mechanism that
//! injects headers into resource attributes.

use otel_arrow_dfe_engine::pdata_context::{Arrival, ArrivalBinding, ArrivalHeaders, ContextError};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use rdkafka::Message as _;
use rdkafka::message::{BorrowedMessage, Headers};
use std::borrow::Cow;

/// Forms the context of `pdata` from the Kafka message headers.
///
/// Headers without a value are skipped. This is independent of the
/// `resource_attrs_from_headers` mechanism which injects headers into resource
/// attributes.
pub(super) fn form_arrival_context(
    kafka_message: &BorrowedMessage<'_>,
    arrival: &ArrivalBinding,
    pdata: &mut OtapPdata,
) -> Result<(), ContextError> {
    let mut headers = kafka_message
        .headers()
        .into_iter()
        .flat_map(|headers| headers.iter())
        .filter_map(|header| header.value.map(|value| (header.key, Cow::Borrowed(value))));
    let (metadata, stats) = arrival.form(Arrival {
        headers: ArrivalHeaders::Iter(&mut headers),
        identity: None,
        peer: None,
    })?;
    if let Some(stats) = stats {
        otel_error!(
            "kafka.capture_policy.limits_exceeded",
            stats = %stats,
        );
    }
    *pdata.context_mut() = metadata.into();
    Ok(())
}
