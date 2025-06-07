// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use std::{collections::HashMap, sync::OnceLock};

use serde_json::Value;

use crate::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    error::Error,
    exporter::ExporterWrapper,
    processor::ProcessorWrapper,
    receiver::ReceiverWrapper,
    runtime_config::{RuntimeNode, RuntimePipeline},
};

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

pub mod config;
mod effect_handler;
pub mod local;
pub mod pipeline;
pub mod runtime_config;
pub mod shared;

pub mod testing;

pub use linkme::distributed_slice;

/// Trait for factory types that expose a name.
/// 
/// This trait is used to define a common interface for different types of factories
/// that create instances of receivers, processors, or exporters.
pub trait NamedFactory {
    /// Returns the name of the node factory.
    fn name(&self) -> &'static str;
}

/// A factory for creating receivers.
pub struct ReceiverFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new receiver instance.
    pub create: fn(config: &Value, receiver_config: &ReceiverConfig) -> ReceiverWrapper<PData>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ReceiverFactory<PData> {
    fn clone(&self) -> Self {
        ReceiverFactory {
            name: self.name,
            create: self.create,
        }
    }
}

impl<PData> NamedFactory for ReceiverFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A factory for creating processors.
pub struct ProcessorFactory<PData> {
    /// The name of the processor.
    pub name: &'static str,
    /// A function that creates a new processor instance.
    pub create: fn(config: &Value, processor_config: &ProcessorConfig) -> ProcessorWrapper<PData>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ProcessorFactory<PData> {
    fn clone(&self) -> Self {
        ProcessorFactory {
            name: self.name,
            create: self.create,
        }
    }
}

impl<PData> NamedFactory for ProcessorFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A factory for creating exporter.
pub struct ExporterFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new exporter instance.
    pub create: fn(config: &Value, exporter_config: &ExporterConfig) -> ExporterWrapper<PData>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ExporterFactory<PData> {
    fn clone(&self) -> Self {
        ExporterFactory {
            name: self.name,
            create: self.create,
        }
    }
}

impl<PData> NamedFactory for ExporterFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// Returns a map of factory names to factory instances.
pub fn get_factory_map<T>(
    factory_map: &'static OnceLock<HashMap<&'static str, T>>,
    factory_slice: &'static [T]
) -> &'static HashMap<&'static str, T>
where
    T: NamedFactory + Clone,
{
    factory_map.get_or_init(|| {
        factory_slice
            .iter()
            .map(|f| (f.name(), f.clone()))
            .collect::<HashMap<&'static str, T>>()
    })
}

/// Generic factory registry that encapsulates all factory maps for a given pdata type.
pub struct FactoryRegistry<PData> {
    receiver_factory_map: OnceLock<HashMap<&'static str, ReceiverFactory<PData>>>,
    processor_factory_map: OnceLock<HashMap<&'static str, ProcessorFactory<PData>>>,
    exporter_factory_map: OnceLock<HashMap<&'static str, ExporterFactory<PData>>>,
}

impl<PData> FactoryRegistry<PData> {
    /// Creates a new factory registry.
    pub const fn new() -> Self {
        Self {
            receiver_factory_map: OnceLock::new(),
            processor_factory_map: OnceLock::new(),
            exporter_factory_map: OnceLock::new(),
        }
    }

    /// Gets the receiver factory map, initializing it if necessary.
    pub fn get_receiver_factory_map(
        &'static self,
        factory_slice: &'static [ReceiverFactory<PData>],
    ) -> &'static HashMap<&'static str, ReceiverFactory<PData>> {
        get_factory_map(&self.receiver_factory_map, factory_slice)
    }

    /// Gets the processor factory map, initializing it if necessary.
    pub fn get_processor_factory_map(
        &'static self,
        factory_slice: &'static [ProcessorFactory<PData>],
    ) -> &'static HashMap<&'static str, ProcessorFactory<PData>> {
        get_factory_map(&self.processor_factory_map, factory_slice)
    }

    /// Gets the exporter factory map, initializing it if necessary.
    pub fn get_exporter_factory_map(
        &'static self,
        factory_slice: &'static [ExporterFactory<PData>],
    ) -> &'static HashMap<&'static str, ExporterFactory<PData>> {
        get_factory_map(&self.exporter_factory_map, factory_slice)
    }

    /// Creates a runtime pipeline from the given pipeline configuration.
    pub fn create_runtime_pipeline(
        &'static self,
        config: otap_df_config::pipeline::PipelineConfig,
        receiver_factories: &'static [ReceiverFactory<PData>],
        processor_factories: &'static [ProcessorFactory<PData>],
        exporter_factories: &'static [ExporterFactory<PData>],
    ) -> Result<RuntimePipeline<PData>, Error<PData>> {
        let receiver_factory_map = self.get_receiver_factory_map(receiver_factories);
        let processor_factory_map = self.get_processor_factory_map(processor_factories);
        let exporter_factory_map = self.get_exporter_factory_map(exporter_factories);
        let mut nodes = Vec::new();

        // Generate all the errors.
        for (node_id, node_config) in config.node_iter() {
            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => {
                    let factory = receiver_factory_map
                        .get(node_config.plugin_urn.as_ref())
                        .ok_or_else(|| Error::UnknownReceiver{plugin_urn: node_config.plugin_urn.clone()})?;
                    let receiver_config = ReceiverConfig::new(node_id.clone());
                    let create = factory.create;
                    nodes.push(RuntimeNode::Receiver {
                        config: node_config.clone(),
                        instance: create(
                            &node_config.config,
                            &receiver_config,
                        )
                    });
                }
                otap_df_config::node::NodeKind::Processor => {
                    let factory = processor_factory_map
                        .get(node_config.plugin_urn.as_ref())
                        .ok_or_else(|| Error::UnknownProcessor{plugin_urn: node_config.plugin_urn.clone()})?;
                    let processor_config = ProcessorConfig::new(node_id.clone());
                    let create = factory.create;
                    nodes.push(RuntimeNode::Processor {
                        config: node_config.clone(),
                        instance: create(
                            &node_config.config,
                            &processor_config,
                        )
                    });
                }
                otap_df_config::node::NodeKind::Exporter => {
                    let factory = exporter_factory_map
                        .get(node_config.plugin_urn.as_ref())
                        .ok_or_else(|| Error::UnknownExporter{plugin_urn: node_config.plugin_urn.clone()})?;
                    let exporter_config = ExporterConfig::new(node_id.clone());
                    let create = factory.create;
                    nodes.push(RuntimeNode::Exporter {
                        config: node_config.clone(),
                        instance: create(
                            &node_config.config,
                            &exporter_config,
                        )
                    });
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            }
        }

        Ok(RuntimePipeline::new(config, nodes))
    }
}

/// Macro to create a factory registry with distributed slices for a specific data type.
/// This macro is hygienic and doesn't require any additional imports.
#[macro_export]
macro_rules! create_factory_registry {
    ($pdata_type:ty, $registry_name:ident) => {
        /// A slice of receiver factories.
        #[$crate::distributed_slice]
        pub static RECEIVER_FACTORIES: [$crate::ReceiverFactory<$pdata_type>] = [..];

        /// A slice of processor factories.
        #[$crate::distributed_slice]
        pub static PROCESSOR_FACTORIES: [$crate::ProcessorFactory<$pdata_type>] = [..];

        /// A slice of exporter factories.
        #[$crate::distributed_slice]
        pub static EXPORTER_FACTORIES: [$crate::ExporterFactory<$pdata_type>] = [..];

        /// Factory registry instance with distributed slices.
        static FACTORY_REGISTRY: $crate::FactoryRegistry<$pdata_type> = $crate::FactoryRegistry::new();

        /// Factory registry accessor with simplified methods.
        pub struct $registry_name;

        impl $registry_name {
            /// Gets the receiver factory map, initializing it if necessary.
            pub fn get_receiver_factory_map() -> &'static std::collections::HashMap<&'static str, $crate::ReceiverFactory<$pdata_type>> {
                FACTORY_REGISTRY.get_receiver_factory_map(&RECEIVER_FACTORIES)
            }

            /// Gets the processor factory map, initializing it if necessary.
            pub fn get_processor_factory_map() -> &'static std::collections::HashMap<&'static str, $crate::ProcessorFactory<$pdata_type>> {
                FACTORY_REGISTRY.get_processor_factory_map(&PROCESSOR_FACTORIES)
            }

            /// Gets the exporter factory map, initializing it if necessary.
            pub fn get_exporter_factory_map() -> &'static std::collections::HashMap<&'static str, $crate::ExporterFactory<$pdata_type>> {
                FACTORY_REGISTRY.get_exporter_factory_map(&EXPORTER_FACTORIES)
            }
        }

        /// Creates a runtime pipeline from the given pipeline configuration.
        pub fn create_runtime_pipeline(
            config: otap_df_config::pipeline::PipelineConfig,
        ) -> Result<$crate::runtime_config::RuntimePipeline<$pdata_type>, $crate::error::Error<$pdata_type>> {
            FACTORY_REGISTRY.create_runtime_pipeline(
                config,
                &RECEIVER_FACTORIES,
                &PROCESSOR_FACTORIES,
                &EXPORTER_FACTORIES,
            )
        }
    };
}
