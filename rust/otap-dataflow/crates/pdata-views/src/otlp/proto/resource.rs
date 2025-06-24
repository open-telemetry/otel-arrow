// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for proto message structs
//! from otlp resources.proto.

use crate::otlp::proto::common::{KeyValueIter, ObjKeyValue};
use crate::views::resource::ResourceView;

use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;

/* ───────────────────────────── VIEW WRAPPERS (zero-alloc) ────────────── */

/// Lightweight wrapper around `Resource` that implements `ResourceView`
#[derive(Clone, Copy)]
pub struct ObjResource<'a> {
    inner: &'a Resource,
}

impl<'a> ObjResource<'a> {
    /// Construct a new instance of `ObjResource`
    #[must_use]
    pub fn new(inner: &'a Resource) -> Self {
        Self { inner }
    }
}

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl ResourceView for ObjResource<'_> {
    type Attribute<'b>
        = ObjKeyValue<'b>
    where
        Self: 'b;

    type AttributesIter<'b>
        = KeyValueIter<'b>
    where
        Self: 'b;

    fn attributes(&self) -> Self::AttributesIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn dropped_attributes_count(&self) -> u32 {
        self.inner.dropped_attributes_count
    }
}
