// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains backend-agnostic, zero-copy view traits for common OTLP messages from
//! OTLP resources.proto

use crate::views::common::AttributeView;

/// View for Resource
pub trait ResourceView {
    /// The `AttributeView` trait associated with this impl of the `ResourceView` trait.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att
    type AttributesIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// Access this resource's attributes
    fn attributes(&self) -> Self::AttributesIter<'_>;

    /// Access this resource's dropped attributes.The value is 0 when no attributes were dropped.
    fn dropped_attributes_count(&self) -> u32;
}
