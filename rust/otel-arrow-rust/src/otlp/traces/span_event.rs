// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::trace::v1::span;

pub struct SpanEvent {
    pub parent_id: u16,
    pub event: span::Event,
}
