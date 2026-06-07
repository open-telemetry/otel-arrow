// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Testing helpers for the telemetry SDK.

use crate::attributes::{AttributeSetHandler, AttributeValue};
use crate::descriptor::AttributesDescriptor;

/// The empty attribute set descriptor.
static EMPTY_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
    name: "empty_metrics",
    fields: &[],
};

/// Empty attribute set for testing.
pub struct EmptyAttributes();

impl AttributeSetHandler for EmptyAttributes {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        &EMPTY_ATTRIBUTES_DESCRIPTOR
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &[]
    }
}
