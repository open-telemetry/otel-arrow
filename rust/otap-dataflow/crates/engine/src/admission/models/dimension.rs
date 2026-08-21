// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Weight dimensions that an admission gate can meter.

/// A single weight dimension measured at an ingress admission point.
///
/// A dimension names *what* the `units` argument of
/// `admit` counts. It is deliberately
/// transport-neutral: `Bytes` means "payload bytes as the component defines
/// them" (for OTLP HTTP, decompressed request-body bytes; for OTLP gRPC, the
/// raw encoded message length), and `Messages` means "one framed application
/// message".
///
/// New dimensions are added only when a component can actually measure them at
/// its admission point. Speculative dimensions are omitted so that
/// [`AdmissionDimensionSet`] stays an honest description of provider support.
///
/// Derives `AttributeEnum` so any provider -- built-in or third-party -- can use
/// the dimension directly as a fixed-cardinality telemetry attribute instead of
/// inventing a parallel vocabulary that would drift from this one.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    otel_arrow_dfe_telemetry_macros::AttributeEnum,
)]
pub enum AdmissionDimension {
    /// Payload bytes, as measured by the component at its admission point.
    Bytes,
    /// Framed application messages, one unit per message.
    Messages,
}

impl AdmissionDimension {
    /// All dimensions, in declaration order. Used by diagnostics and tests.
    pub const ALL: &'static [AdmissionDimension] =
        &[AdmissionDimension::Bytes, AdmissionDimension::Messages];

    /// Returns the stable, user-facing spelling of this dimension.
    ///
    /// Used in startup diagnostics and telemetry attributes, so it must stay
    /// stable across releases.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Messages => "messages",
        }
    }

    /// Bit position of this dimension inside an [`AdmissionDimensionSet`].
    const fn bit(self) -> u32 {
        match self {
            Self::Bytes => 1 << 0,
            Self::Messages => 1 << 1,
        }
    }
}

impl std::fmt::Display for AdmissionDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A compact set of [`AdmissionDimension`]s.
///
/// Admission implementations advertise the dimensions they can meter. A set (rather than a
/// single `dimension()` accessor) is required so that one provider can serve a
/// byte-metered OTLP receiver and a message-metered Syslog receiver in the same
/// pipeline; a singular accessor would make multidimensional providers
/// impossible to express.
///
/// The set is a `u32` bitset: it is `Copy`, allocation-free, and cheap to
/// return from a `dyn` trait method.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdmissionDimensionSet(u32);

impl AdmissionDimensionSet {
    /// The empty set. A provider advertising this can bind no gate at all.
    pub const EMPTY: Self = Self(0);

    /// Creates a set containing exactly one dimension.
    #[must_use]
    pub const fn single(dimension: AdmissionDimension) -> Self {
        Self(dimension.bit())
    }

    /// Returns a set containing everything in `self` plus `dimension`.
    #[must_use]
    pub const fn with(self, dimension: AdmissionDimension) -> Self {
        Self(self.0 | dimension.bit())
    }

    /// Returns true when `dimension` is a member of this set.
    #[must_use]
    pub const fn contains(self, dimension: AdmissionDimension) -> bool {
        self.0 & dimension.bit() != 0
    }

    /// Returns true when the set has no members.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Renders the set for startup diagnostics, e.g. `[bytes, messages]`.
    #[must_use]
    pub fn to_display_string(self) -> String {
        let members: Vec<&'static str> = AdmissionDimension::ALL
            .iter()
            .filter(|dimension| self.contains(**dimension))
            .map(|dimension| dimension.as_str())
            .collect();
        format!("[{}]", members.join(", "))
    }
}

impl FromIterator<AdmissionDimension> for AdmissionDimensionSet {
    fn from_iter<I: IntoIterator<Item = AdmissionDimension>>(iter: I) -> Self {
        iter.into_iter()
            .fold(Self::EMPTY, |set, dimension| set.with(dimension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a provider advertises both byte and message metering.
    /// Guarantees: membership queries answer true for every advertised dimension and
    /// false for none of them, so a multidimensional provider is expressible.
    #[test]
    fn dimension_set_reports_every_advertised_member() {
        let set = AdmissionDimensionSet::single(AdmissionDimension::Bytes)
            .with(AdmissionDimension::Messages);

        assert!(set.contains(AdmissionDimension::Bytes));
        assert!(set.contains(AdmissionDimension::Messages));
        assert!(!set.is_empty());
    }

    /// Scenario: a provider advertises only message metering.
    /// Guarantees: an unadvertised dimension is reported absent, which is the signal
    /// startup binding uses to reject a mismatched component.
    #[test]
    fn dimension_set_excludes_unadvertised_members() {
        let set = AdmissionDimensionSet::single(AdmissionDimension::Messages);

        assert!(!set.contains(AdmissionDimension::Bytes));
        assert!(set.contains(AdmissionDimension::Messages));
    }

    /// Scenario: a provider advertises no dimension at all.
    /// Guarantees: the empty set contains nothing, so no gate can be bound from it.
    #[test]
    fn empty_dimension_set_contains_nothing() {
        let set = AdmissionDimensionSet::EMPTY;

        assert!(set.is_empty());
        for dimension in AdmissionDimension::ALL {
            assert!(!set.contains(*dimension));
        }
    }

    /// Scenario: a startup binding error renders the provider's supported dimensions.
    /// Guarantees: the rendered text lists members in a stable declaration order so
    /// diagnostics do not vary between runs.
    #[test]
    fn dimension_set_display_is_stable_and_ordered() {
        let set: AdmissionDimensionSet = [AdmissionDimension::Messages, AdmissionDimension::Bytes]
            .into_iter()
            .collect();

        assert_eq!(set.to_display_string(), "[bytes, messages]");
        assert_eq!(
            AdmissionDimensionSet::EMPTY.to_display_string(),
            "[]".to_owned()
        );
    }
}
