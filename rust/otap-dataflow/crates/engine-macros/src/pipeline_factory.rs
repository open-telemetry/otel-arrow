// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the `#[pipeline_factory]` proc macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemStatic, Token, Type,
    parse::{Parse, ParseStream},
};

/// Arguments for the pipeline_factory macro.
pub(crate) struct PipelineFactoryArgs {
    /// Prefix for generated static variables.
    pub prefix: Ident,
    /// Data type for the pipeline factory.
    pub pdata_type: Type,
}

impl Parse for PipelineFactoryArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let prefix = input.parse::<Ident>()?;
        let _comma: Token![,] = input.parse()?;
        let pdata_type = input.parse::<Type>()?;
        Ok(PipelineFactoryArgs { prefix, pdata_type })
    }
}

/// Generate the full output for a `#[pipeline_factory(...)]` annotation.
pub(crate) fn expand_pipeline_factory(
    args: PipelineFactoryArgs,
    registry_static: ItemStatic,
) -> TokenStream {
    let prefix = &args.prefix;
    let pdata_type = &args.pdata_type;
    let registry_name = &registry_static.ident;
    let registry_vis = &registry_static.vis;

    // Generate prefixed identifiers
    let receiver_factories_name = quote::format_ident!("{}_RECEIVER_FACTORIES", prefix);
    let processor_factories_name = quote::format_ident!("{}_PROCESSOR_FACTORIES", prefix);
    let exporter_factories_name = quote::format_ident!("{}_EXPORTER_FACTORIES", prefix);
    let extension_factories_name = quote::format_ident!("{}_EXTENSION_FACTORIES", prefix);
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
    let get_extension_factory_map_name = quote::format_ident!(
        "get_{}_extension_factory_map",
        prefix.to_string().to_lowercase()
    );

    quote! {
        /// A slice of receiver factories.
        #[::otap_df_engine::distributed_slice]
        pub static #receiver_factories_name: [::otap_df_engine::ReceiverFactory<#pdata_type>] = [..];

        /// A slice of processor factories.
        #[::otap_df_engine::distributed_slice]
        pub static #processor_factories_name: [::otap_df_engine::ProcessorFactory<#pdata_type>] = [..];

        /// A slice of exporter factories.
        #[::otap_df_engine::distributed_slice]
        pub static #exporter_factories_name: [::otap_df_engine::ExporterFactory<#pdata_type>] = [..];

        /// A slice of extension factories.
        #[::otap_df_engine::distributed_slice]
        pub static #extension_factories_name: [::otap_df_engine::ExtensionFactory] = [..];

        /// The factory registry instance.
        #registry_vis static #registry_name: std::sync::LazyLock<PipelineFactory<#pdata_type>> = std::sync::LazyLock::new(|| {
            // Reference build_registry to avoid unused import warning, even though we don't call it
            let _ = build_factory::<#pdata_type>;
            PipelineFactory::new(
                &#receiver_factories_name,
                &#processor_factories_name,
                &#exporter_factories_name,
                &#extension_factories_name,
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

        /// Gets the extension factory map, initializing it if necessary.
        pub fn #get_extension_factory_map_name() -> &'static std::collections::HashMap<&'static str, ::otap_df_engine::ExtensionFactory> {
            #registry_name.get_extension_factory_map()
        }
    }
}
