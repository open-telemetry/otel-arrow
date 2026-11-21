// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run the same invalid-input contract against every linked production factory.

use otel_arrow_dfe_contrib_extensions as _;
use otel_arrow_dfe_contrib_nodes as _;
use otel_arrow_dfe_controller::CONTROLLER_EXTENSION_FACTORIES;
use otel_arrow_dfe_core_nodes as _;
use otel_arrow_dfe_dev_nodes as _;
use otel_arrow_dfe_otap::OTAP_PIPELINE_FACTORY;
use serde_json::json;
use std::collections::BTreeMap;

/// Each entry is an actual call to the linked factory's validator.
pub fn observations() -> BTreeMap<&'static str, bool> {
    let invalid = json!("invalid scalar configuration");
    let mut results = BTreeMap::new();
    let factory = &OTAP_PIPELINE_FACTORY;
    macro_rules! observe {
        ($registry:expr) => {
            for component in $registry {
                let _ = results.insert(
                    component.name,
                    (component.validate_config)(&invalid).is_err(),
                );
            }
        };
    }
    observe!(factory.get_receiver_factory_map().values());
    observe!(factory.get_processor_factory_map().values());
    observe!(factory.get_exporter_factory_map().values());
    observe!(factory.get_extension_factory_map().values());
    observe!(CONTROLLER_EXTENSION_FACTORIES.iter());
    results
}
