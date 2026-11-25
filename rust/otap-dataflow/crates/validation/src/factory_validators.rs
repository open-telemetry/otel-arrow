// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run the same invalid-input contract against every linked production factory.

use otel_arrow_dfe_controller::CONTROLLER_EXTENSION_FACTORIES;
use otel_arrow_dfe_core_nodes as _;
use otel_arrow_dfe_dev_nodes as _;
use otel_arrow_dfe_otap::OTAP_PIPELINE_FACTORY;
use serde_json::json;
use std::collections::BTreeMap;

/// Result from invoking one linked production factory.
pub struct Observation {
    /// Component kind in the factory registry.
    pub role: &'static str,
    /// Whether the factory rejected the invalid configuration.
    pub rejected: bool,
}

/// Each entry is an actual call to the linked factory's validator.
pub fn observations(include: impl Fn(&str) -> bool) -> BTreeMap<&'static str, Observation> {
    // The validation library contributes its own test-only factories to linkme.
    let selected = |urn: &str| {
        !matches!(
            urn,
            crate::fanout_processor::FANOUT_PROCESSOR_URN
                | crate::validation_exporter::VALIDATION_EXPORTER_URN
        ) && include(urn)
    };
    let invalid = json!("invalid scalar configuration");
    let mut results = BTreeMap::new();
    let factory = &OTAP_PIPELINE_FACTORY;
    macro_rules! observe {
        ($registry:expr, $role:literal) => {
            for component in $registry {
                if !selected(component.name) {
                    continue;
                }
                let _ = results.insert(
                    component.name,
                    Observation {
                        role: $role,
                        rejected: (component.validate_config)(&invalid).is_err(),
                    },
                );
            }
        };
    }
    observe!(factory.get_receiver_factory_map().values(), "receiver");
    observe!(factory.get_processor_factory_map().values(), "processor");
    observe!(factory.get_exporter_factory_map().values(), "exporter");
    observe!(factory.get_extension_factory_map().values(), "extension");
    observe!(
        CONTROLLER_EXTENSION_FACTORIES.iter(),
        "controller-extension"
    );
    results
}
