// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only mock metric/attributes definitions for testing without a
//! pipeline controller.

use crate::{
    attributes::{AttributeSetHandler, AttributeValue},
    descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricsDescriptor,
        MetricsField,
    },
    metrics::MetricSetHandler,
};

static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
    name: "test_metrics",
    metrics: &[
        MetricsField {
            name: "counter1",
            unit: "1",
            brief: "Test counter 1",
            instrument: Instrument::Counter,
        },
        MetricsField {
            name: "counter2",
            unit: "1",
            brief: "Test counter 2",
            instrument: Instrument::Counter,
        },
    ],
};

static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
    name: "test_attributes",
    fields: &[AttributeField {
        key: "test_key",
        r#type: AttributeValueType::String,
        brief: "Test attribute",
    }],
};

#[derive(Debug)]
/// Test metric set with two valuies
pub struct MockMetricSet {
    values: Vec<u64>,
}

impl MockMetricSet {
    /// A test metric with two counters
    #[must_use]
    pub fn new() -> Self {
        Self { values: vec![0, 0] }
    }
}

impl Default for MockMetricSet {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricSetHandler for MockMetricSet {
    fn descriptor(&self) -> &'static MetricsDescriptor {
        &MOCK_METRICS_DESCRIPTOR
    }
    fn snapshot_values(&self) -> Vec<u64> {
        self.values.clone()
    }
    fn clear_values(&mut self) {
        self.values.iter_mut().for_each(|v| *v = 0);
    }
    fn needs_flush(&self) -> bool {
        self.values.iter().any(|&v| v != 0)
    }
}

/// Test metrics attribute set.
#[derive(Debug)]
pub struct MockAttributeSet {
    attribute_values: Vec<AttributeValue>,
}

impl MockAttributeSet {
    /// New test attribute set.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            attribute_values: vec![AttributeValue::String(value.into())],
        }
    }
}

impl AttributeSetHandler for MockAttributeSet {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        &MOCK_ATTRIBUTES_DESCRIPTOR
    }
    fn iter_attributes<'a>(&'a self) -> crate::attributes::AttributeIterator<'a> {
        crate::attributes::AttributeIterator::new(
            MOCK_ATTRIBUTES_DESCRIPTOR.fields,
            &self.attribute_values,
        )
    }
    fn attribute_values(&self) -> &[AttributeValue] {
        &self.attribute_values
    }
}

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
