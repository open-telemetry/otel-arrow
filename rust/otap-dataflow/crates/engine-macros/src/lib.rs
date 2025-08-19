// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Proc macros for the async pipeline engine
//!
//! This crate provides procedural macros that help generate boilerplate code
//! for factory registries and distributed slices in the pipeline engine.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemStatic, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Arguments for the pipeline_factory macro
struct PipelineFactoryArgs {
    /// Prefix for generated static variables
    prefix: Ident,
    /// Data type for the pipeline factory
    pdata_type: Type,
}

impl Parse for PipelineFactoryArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let prefix = input.parse::<Ident>()?;
        let _comma: Token![,] = input.parse()?;
        let pdata_type = input.parse::<Type>()?;
        Ok(PipelineFactoryArgs { prefix, pdata_type })
    }
}

/// Attribute macro to generate distributed slices and initialize a factory registry.
///
/// This macro generates distributed slices for factories and initializes the annotated
/// XYZ_FACTORY_PIPELINE static variable. It requires a prefix parameter to avoid name
/// conflicts when used multiple times in the same scope.
///
/// # Usage
///
/// Simply declare a XYZ_FACTORY_PIPELINE static and annotate it with a prefix and data type:
/// ```rust,ignore
/// use otap_df_engine::{PipelineFactory, build_factory};
/// use otap_df_engine_macros::pipeline_factory;
///
/// // Define your data type (this would be defined elsewhere)
/// struct MyData;
///
/// // Declare and initialize the factory of pipelines
/// #[pipeline_factory(MY_PREFIX, MyData)]
/// static XYZ_FACTORY_PIPELINE: PipelineFactory<MyData> = build_factory();
/// ```
///
/// Note: You need to import both `PipelineFactory` and `build_factory`. The
/// `build_registry()` call is a placeholder that gets replaced by the macro, but
/// importing it explicitly makes the API more natural and clear.
/// The individual factory types are imported internally by the macro.
///
/// This generates:
/// - Distributed slices for receiver, processor, and exporter factories (prefixed)
/// - Proper initialization of the FACTORY_REGISTRY with lazy loading
/// - Helper functions to access factory maps (prefixed)
#[proc_macro_attribute]
pub fn pipeline_factory(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PipelineFactoryArgs);
    let registry_static = parse_macro_input!(input as ItemStatic);

    let prefix = &args.prefix;
    let pdata_type = &args.pdata_type;
    let registry_name = &registry_static.ident;
    let registry_vis = &registry_static.vis;

    // Generate prefixed identifiers
    let receiver_factories_name = quote::format_ident!("{}_RECEIVER_FACTORIES", prefix);
    let processor_factories_name = quote::format_ident!("{}_PROCESSOR_FACTORIES", prefix);
    let exporter_factories_name = quote::format_ident!("{}_EXPORTER_FACTORIES", prefix);
    let get_receiver_factory_map_name = quote::format_ident!(
        "get_{}_receiver_factory_map",
        prefix.to_string().to_lowercase()
    );
    let get_processor_factory_map_name = quote::format_ident!(
        "get_{}_processor_factory_map",
        prefix.to_string().to_lowercase()
    );
    let get_exporter_factory_map_name = quote::format_ident!(
        "get_{}_exporter_factory_map",
        prefix.to_string().to_lowercase()
    );

    let output = quote! {
        /// A slice of receiver factories.
        #[::otap_df_engine::distributed_slice]
        pub static #receiver_factories_name: [::otap_df_engine::ReceiverFactory<#pdata_type>] = [..];

        /// A slice of processor factories.
        #[::otap_df_engine::distributed_slice]
        pub static #processor_factories_name: [::otap_df_engine::ProcessorFactory<#pdata_type>] = [..];

        /// A slice of exporter factories.
        #[::otap_df_engine::distributed_slice]
        pub static #exporter_factories_name: [::otap_df_engine::ExporterFactory<#pdata_type>] = [..];

        /// The factory registry instance.
        #registry_vis static #registry_name: std::sync::LazyLock<PipelineFactory<#pdata_type>> = std::sync::LazyLock::new(|| {
            // Reference build_registry to avoid unused import warning, even though we don't call it
            let _ = build_factory::<#pdata_type>;
            PipelineFactory::new(
                &#receiver_factories_name,
                &#processor_factories_name,
                &#exporter_factories_name,
            )
        });

        /// Gets the receiver factory map, initializing it if necessary.
        pub fn #get_receiver_factory_map_name() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ReceiverFactory<#pdata_type>> {
            #registry_name.get_receiver_factory_map()
        }

        /// Gets the processor factory map, initializing it if necessary.
        pub fn #get_processor_factory_map_name() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ProcessorFactory<#pdata_type>> {
            #registry_name.get_processor_factory_map()
        }

        /// Gets the exporter factory map, initializing it if necessary.
        pub fn #get_exporter_factory_map_name() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ExporterFactory<#pdata_type>> {
            #registry_name.get_exporter_factory_map()
        }
    };

    output.into()
}
