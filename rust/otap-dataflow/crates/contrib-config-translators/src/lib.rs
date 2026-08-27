// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Vendor-specific configuration translators.
//!
//! A *translator* turns a vendor's own configuration document into an
//! [`OtelDataflowSpec`] that the OTAP dataflow engine can run. This mirrors the
//! [`ConfigProvider`](otel_arrow_dfe_config::config_provider::ConfigProvider) abstraction one stage
//! later in the chain: a provider resolves a URI to raw content, whereas a translator converts
//! foreign content into the engine's own pipeline model.
//!
//! ```text
//!   vendor config (JSON)  --ConfigTranslator-->  OtelDataflowSpec  --serialize-->  pipeline YAML
//! ```
//!
//! # Available translators
//!
//! - [`amcs`] -- Azure Monitor Configuration Service (AMCS) third-party (3P) configuration,
//!   as delivered to the Azure Monitor Agent for customer-authored Data Collection Rules.
//!
//! A first-party (1P) Geneva/GigLA translator can be added as a sibling module implementing the
//! same [`ConfigTranslator`] trait.

pub mod amcs;
pub mod error;
mod yaml_style;

use otel_arrow_dfe_config::engine::{EngineConfig, OtelDataflowSpec};

pub use error::Error;

/// Serializes a specification to YAML, omitting the `engine` block when it carries nothing but
/// engine defaults.
///
/// `OtelDataflowSpec::engine` is `#[serde(default)]`, so an absent block deserializes back to
/// `EngineConfig::default()` and the engine behaves identically. Writing the block out anyway
/// would pin whatever the defaults happened to be at generation time -- currently around sixty
/// lines of internal-observability pipeline that dwarf the pipeline we actually generate, and
/// that would silently stop tracking the engine if those defaults ever change.
///
/// Any engine configuration we set deliberately differs from the default and is therefore kept.
fn spec_to_yaml(spec: &OtelDataflowSpec) -> Result<String, Error> {
    let mut value = serde_yaml::to_value(spec).map_err(|e| Error::Serialization {
        details: e.to_string(),
    })?;
    let default_engine =
        serde_yaml::to_value(EngineConfig::default()).map_err(|e| Error::Serialization {
            details: e.to_string(),
        })?;

    if let serde_yaml::Value::Mapping(map) = &mut value {
        let key = serde_yaml::Value::String("engine".to_owned());
        if map.get(&key) == Some(&default_engine) {
            let _ = map.remove(&key);
        }
    }

    yaml_style::sort_nodes(&mut value);

    serde_yaml::to_string(&value)
        .map(|yaml| yaml_style::prettify(&yaml))
        .map_err(|e| Error::Serialization {
            details: e.to_string(),
        })
}

/// Converts a vendor-specific configuration document into an engine pipeline specification.
///
/// Implementations are expected to be pure: given the same input (and the same environment
/// snapshot, where relevant) they must produce the same specification.
pub trait ConfigTranslator {
    /// A short, stable name identifying the configuration dialect this translator understands
    /// (for example `amcs`). Used in diagnostics and for translator selection.
    fn dialect(&self) -> &str;

    /// Translate raw vendor configuration content into a pipeline specification.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the input cannot be parsed, or if the resulting specification
    /// would be rejected by the engine configuration model.
    fn translate(&self, raw: &str) -> Result<OtelDataflowSpec, Error>;

    /// Translate raw vendor configuration content directly to a pipeline YAML document.
    ///
    /// The default implementation calls [`ConfigTranslator::translate`] and serializes the
    /// result, which is what every current translator wants.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if translation fails, or if the specification cannot be serialized.
    fn translate_to_yaml(&self, raw: &str) -> Result<String, Error> {
        let spec = self.translate(raw)?;
        spec_to_yaml(&spec)
    }
}
