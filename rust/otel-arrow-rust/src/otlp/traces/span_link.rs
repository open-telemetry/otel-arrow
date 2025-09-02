// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::trace::v1::span;

pub struct SpanLink {
    pub parent_id: u16,
    pub link: span::Link,
}
