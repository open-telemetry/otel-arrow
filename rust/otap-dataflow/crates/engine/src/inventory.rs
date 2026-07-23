// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component inventory: link-time metadata for security-relevant components.
//!
//! This module implements the runtime surface of [RFC 0001: Component
//! Inventory][rfc]. Components annotated with the
//! [`#[component_inventory]`][macro] attribute macro (in
//! `otap-df-engine-macros`) emit one [`ComponentMeta`] entry into the
//! [`COMPONENT_INVENTORY`] distributed slice at link time. Offline tooling
//! (`cargo xtask component-inventory`, added in a later phase) reads the slice
//! to detect new/removed components for threat-model drift detection,
//! documentation coverage, and security review.
//!
//! This mirrors the existing `#[capability]` -> `KNOWN_CAPABILITIES` mechanism
//! (`crate::capability`). The data is read only by offline tooling and never at
//! runtime, so the mechanism is zero-cost.
//!
//! [rfc]: https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/rfcs/0001-component-inventory.md
//! [macro]: otap_df_engine_macros::component_inventory

/// Component category.
///
/// The `#[component_inventory]` macro accepts a bare identifier (e.g.
/// `Receiver`) and rejects unknown variants at compile time, preventing
/// misspellings like `Reciever` from silently corrupting the inventory. For
/// factory components the macro also validates the category against the URN's
/// middle segment (e.g. `urn:otel:`**`receiver`**`:otlp`).
///
/// Phase 1 (RFC 0001) ships only the four factory categories. The non-factory
/// categories (`Admin`, `Controller`, `Cli`, `Subsystem`, `Safety`) proposed in
/// the RFC are deferred to Phase 2, when the non-factory components (admin
/// server, controller, `dfctl`, memory limiter) are actually annotated and the
/// synthetic-URN scheme for them is settled with the SIG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Category {
    /// A receiver: ingests telemetry into a pipeline (`urn:...:receiver:...`).
    Receiver,
    /// An exporter: emits telemetry out of a pipeline (`urn:...:exporter:...`).
    Exporter,
    /// A processor: transforms telemetry in a pipeline (`urn:...:processor:...`).
    Processor,
    /// An extension: shared, non-pipeline functionality (`urn:...:extension:...`).
    Extension,
    /// Built-in HTTP/gRPC admin server (`urn:...:admin:...`).
    Admin,
    /// Pipeline controller or OpAMP engine (`urn:...:controller:...`).
    Controller,
    /// Command line tooling (`urn:...:cli:...`).
    Cli,
    /// Core infrastructure subsystem (`urn:...:subsystem:...`).
    Subsystem,
    /// Safety guardrails such as memory limiter (`urn:...:safety:...`).
    Safety,
}

impl Category {
    /// The URN category segment for this variant (e.g. `Receiver` -> `"receiver"`).
    ///
    /// Used to cross-check `category` against a component's URN and by the
    /// inventory tooling.
    #[must_use]
    pub const fn urn_segment(self) -> &'static str {
        match self {
            Category::Receiver => "receiver",
            Category::Exporter => "exporter",
            Category::Processor => "processor",
            Category::Extension => "extension",
            Category::Admin => "admin",
            Category::Controller => "controller",
            Category::Cli => "cli",
            Category::Subsystem => "subsystem",
            Category::Safety => "safety",
        }
    }
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.urn_segment())
    }
}

/// Well-known attribute keys (RFC 0001, "Option A": free-form map + key
/// constants). Contributors are encouraged to use these constants for the
/// security-relevant attributes so keys stay consistent across components.
///
/// The `attributes` map is intentionally free-form (`&[(&str, &str)]`) so any
/// component can express any property; these constants only standardize the
/// common keys. Value validation for security-relevant keys (RFC "Option C")
/// is intentionally not implemented in Phase 1.
//
// TODO(stability): a component "stability" attribute was considered but
// intentionally omitted. Stability is not modeled per-signal: many components
// have no signal type, or handle multiple signal types, so a single per-signal
// stability field does not fit. Revisit with the SIG if a component-level
// stability key is wanted later.
pub mod attrs {
    /// Network port the component listens on or connects to (e.g. `"4317"`).
    pub const PORT: &str = "port";
    /// Wire protocol (e.g. `"gRPC (HTTP/2)"`, `"HTTP"`).
    pub const PROTOCOL: &str = "protocol";
    /// Authentication mechanism (e.g. `"mTLS (opt-in)"`, `"NONE"`).
    pub const AUTH: &str = "auth";
    /// Whether/how the component accesses the local filesystem.
    pub const FILESYSTEM_ACCESS: &str = "filesystem_access";
    /// Cloud API the component talks to, if any.
    pub const CLOUD_API: &str = "cloud_api";
    /// Cargo feature flag gating the component, if any.
    pub const FEATURE_FLAG: &str = "feature_flag";
}

/// Inventory metadata for one security-relevant component.
///
/// Collected at link time via the [`COMPONENT_INVENTORY`] distributed slice;
/// extracted by `cargo xtask component-inventory`. Identity and category are
/// the fixed fields; all domain-specific properties live in the free-form
/// [`attributes`](ComponentMeta::attributes) slice so the struct is not biased
/// toward any one access pattern (network, filesystem, cloud, ...).
///
/// Note: this struct is deliberately **not** `#[non_exhaustive]` -- the
/// `#[component_inventory]` macro constructs it directly with a struct literal
/// from other crates, which `#[non_exhaustive]` would forbid. New fields must
/// therefore be added in lockstep with the macro's emission.
#[derive(Debug, Clone, Copy)]
pub struct ComponentMeta {
    /// Unique identifier. For factory components this is the factory's URN
    /// (its `name` field). For non-factory components it is an explicit,
    /// URN-shaped id supplied on the annotation.
    pub id: &'static str,

    /// Component category (validated against the URN segment when a URN exists).
    pub category: Category,

    /// Short human-readable description.
    pub description: Option<&'static str>,

    /// Source file, auto-populated via [`file!`] by the macro.
    pub file: &'static str,

    /// Source line, auto-populated via [`line!`] by the macro.
    pub line: u32,

    /// Free-form key/value attributes. Well-known keys are provided as
    /// constants in [`attrs`].
    pub attributes: &'static [(&'static str, &'static str)],
}

impl ComponentMeta {
    /// Look up an attribute value by key, if present.
    #[must_use]
    pub fn attribute(&self, key: &str) -> Option<&'static str> {
        self.attributes
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }
}

/// Link-time registry of all components annotated with `#[component_inventory]`
/// compiled into the binary.
///
/// Populated by the `#[component_inventory]` proc macro. Read only by offline
/// tooling (`cargo xtask component-inventory`); never at runtime.
//
// `linkme::distributed_slice` requires a `pub static`; `#[doc(hidden)]`
// excludes it from generated rustdoc so external crates don't see it in the
// public API surface. `#[allow(unsafe_code)]` is required because
// `linkme::distributed_slice` emits a static with `#[link_section = "..."]`,
// which the engine crate's `-D unsafe-code` lint would otherwise reject.
#[doc(hidden)]
#[allow(unsafe_code)]
#[linkme::distributed_slice]
pub static COMPONENT_INVENTORY: [ComponentMeta] = [..];

/// Iterate over every component registered in the [`COMPONENT_INVENTORY`]
/// distributed slice.
///
/// Mirrors the iteration pattern used for `KNOWN_CAPABILITIES`.
#[must_use]
pub fn components() -> &'static [ComponentMeta] {
    &COMPONENT_INVENTORY
}
