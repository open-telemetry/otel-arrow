// SPDX-License-Identifier: Apache-2.0

//! Proc macros for the async pipeline engine
//!
//! This crate provides procedural macros that help generate boilerplate code
//! for factory registries and distributed slices in the pipeline engine.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStatic, Type, parse_macro_input};

/// Attribute macro to generate distributed slices and initialize a factory registry.
///
/// This macro generates distributed slices for factories and initializes the annotated
/// FACTORY_REGISTRY static variable.
///
/// # Usage
///
/// Simply declare a FACTORY_REGISTRY static and annotate it with the data type:
/// ```rust,ignore
/// use otap_df_engine::{FactoryRegistry, build_registry};
/// use otap_df_engine_macros::factory_registry;
///
/// // Define your data type (this would be defined elsewhere)
/// struct MyData;
///
/// // Declare and initialize the factory registry
/// #[factory_registry(MyData)]
/// static FACTORY_REGISTRY: FactoryRegistry<MyData> = build_registry();
/// ```
///
/// Note: You need to import both `FactoryRegistry` and `build_registry`. The 
/// `build_registry()` call is a placeholder that gets replaced by the macro, but
/// importing it explicitly makes the API more natural and clear.
/// The individual factory types are imported internally by the macro.
///
/// This generates:
/// - Distributed slices for receiver, processor, and exporter factories
/// - Proper initialization of the FACTORY_REGISTRY with lazy loading
/// - Helper functions to access factory maps
#[proc_macro_attribute]
pub fn factory_registry(args: TokenStream, input: TokenStream) -> TokenStream {
    let pdata_type = parse_macro_input!(args as Type);
    let registry_static = parse_macro_input!(input as ItemStatic);

    let registry_name = &registry_static.ident;
    let registry_vis = &registry_static.vis;

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
        #registry_vis static #registry_name: std::sync::LazyLock<FactoryRegistry<#pdata_type>> = std::sync::LazyLock::new(|| {
            // Reference build_registry to avoid unused import warning, even though we don't call it
            let _ = build_registry::<#pdata_type>;
            FactoryRegistry::new(
                &RECEIVER_FACTORIES,
                &PROCESSOR_FACTORIES,
                &EXPORTER_FACTORIES,
            )
        });

        /// Gets the receiver factory map, initializing it if necessary.
        pub fn get_receiver_factory_map() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ReceiverFactory<#pdata_type>> {
            #registry_name.get_receiver_factory_map()
        }

        /// Gets the processor factory map, initializing it if necessary.
        pub fn get_processor_factory_map() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ProcessorFactory<#pdata_type>> {
            #registry_name.get_processor_factory_map()
        }

        /// Gets the exporter factory map, initializing it if necessary.
        pub fn get_exporter_factory_map() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ExporterFactory<#pdata_type>> {
            #registry_name.get_exporter_factory_map()
        }
    };

    output.into()
}
