// SPDX-License-Identifier: Apache-2.0

//! Proc macros for the async pipeline engine
//! 
//! This crate provides procedural macros that help generate boilerplate code
//! for factory registries and distributed slices in the pipeline engine.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStatic, Type};

/// Attribute macro to generate distributed slices and helper functions for a factory registry.
/// 
/// This macro generates distributed slices for factories and helper functions to access them.
/// 
/// # Usage
/// 
/// First, import the necessary types and macro:
/// ```rust,ignore
/// use otap_df_engine::{FactoryRegistry};
/// use otap_df_engine_macros::factory_registry;
/// 
/// // Define your data type (this would be defined elsewhere)
/// struct MyData;
/// 
/// // Apply the macro to generate distributed slices and helpers
/// #[factory_registry(MyData)]
/// static FACTORY_REGISTRY: FactoryRegistry<MyData> = FactoryRegistry::new();
/// ```
/// 
/// This generates:
/// - Distributed slices for receiver, processor, and exporter factories
/// - Helper functions to access factory maps
/// - A create_runtime_pipeline function
#[proc_macro_attribute]
pub fn factory_registry(args: TokenStream, input: TokenStream) -> TokenStream {
    let pdata_type = parse_macro_input!(args as Type);
    let registry_static = parse_macro_input!(input as ItemStatic);
    
    let registry_name = &registry_static.ident;
    
    let output = quote! {
        /// A slice of receiver factories.
        #[::otap_df_engine::distributed_slice]
        pub static RECEIVER_FACTORIES: [::otap_df_engine::ReceiverFactory<#pdata_type>] = [..];

        /// A slice of processor factories.
        #[::otap_df_engine::distributed_slice]
        pub static PROCESSOR_FACTORIES: [::otap_df_engine::ProcessorFactory<#pdata_type>] = [..];

        /// A slice of exporter factories.
        #[::otap_df_engine::distributed_slice]
        pub static EXPORTER_FACTORIES: [::otap_df_engine::ExporterFactory<#pdata_type>] = [..];

        /// The factory registry instance.
        #registry_static

        /// Gets the receiver factory map, initializing it if necessary.
        pub fn get_receiver_factory_map() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ReceiverFactory<#pdata_type>> {
            #registry_name.get_receiver_factory_map(&RECEIVER_FACTORIES)
        }

        /// Gets the processor factory map, initializing it if necessary.
        pub fn get_processor_factory_map() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ProcessorFactory<#pdata_type>> {
            #registry_name.get_processor_factory_map(&PROCESSOR_FACTORIES)
        }

        /// Gets the exporter factory map, initializing it if necessary.
        pub fn get_exporter_factory_map() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ExporterFactory<#pdata_type>> {
            #registry_name.get_exporter_factory_map(&EXPORTER_FACTORIES)
        }

        /// Creates a runtime pipeline from the given pipeline configuration.
        pub fn create_runtime_pipeline(
            config: ::otap_df_config::pipeline::PipelineConfig,
        ) -> Result<::otap_df_engine::runtime_config::RuntimePipeline<#pdata_type>, ::otap_df_engine::error::Error<#pdata_type>> {
            #registry_name.create_runtime_pipeline(
                config,
                &RECEIVER_FACTORIES,
                &PROCESSOR_FACTORIES,
                &EXPORTER_FACTORIES,
            )
        }
    };

    output.into()
}
