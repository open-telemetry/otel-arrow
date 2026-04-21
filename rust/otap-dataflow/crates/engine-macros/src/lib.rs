// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Proc macros for the async pipeline engine
//!
//! This crate provides procedural macros that help generate boilerplate code
//! for factory registries and distributed slices in the pipeline engine.

use proc_macro::TokenStream;
use syn::{ItemStatic, ItemTrait, parse_macro_input};

mod capability;
mod pipeline_factory;

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
    let args = parse_macro_input!(args as pipeline_factory::PipelineFactoryArgs);
    let registry_static = parse_macro_input!(input as ItemStatic);
    pipeline_factory::expand_pipeline_factory(args, registry_static).into()
}

/// Attribute macro for defining a capability trait.
///
/// Annotate a trait definition to generate the full capability infrastructure.
/// The original trait is consumed and replaced with the generated items.
///
/// # Input
///
/// ```rust,ignore
/// #[capability(name = "bearer_token_provider", description = "Provides bearer tokens")]
/// pub trait BearerTokenProvider {
///     async fn get_token(&self) -> Result<BearerToken, Error>;
///     fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>>;
/// }
/// ```
///
/// # Generated output
///
/// Given the input above, the macro generates (conceptually):
///
/// ```rust,ignore
/// // 1. Local trait — !Send, used by local pipeline nodes.
/// pub mod local {
///     #[async_trait(?Send)]
///     pub trait BearerTokenProvider {
///         async fn get_token(&self) -> Result<BearerToken, Error>;
///         fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>>;
///     }
/// }
///
/// // 2. Shared trait — Send, used by shared pipeline nodes.
/// pub mod shared {
///     #[async_trait]
///     pub trait BearerTokenProvider: Send {
///         async fn get_token(&self) -> Result<BearerToken, Error>;
///         fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>>;
///     }
/// }
///
/// // 3. SharedAsLocal adapter — delegates local::Trait to a shared impl.
/// //    Used internally by the engine for shared-only extensions that need
/// //    to serve local consumers.
/// struct SharedAsLocalBearerTokenProvider(Box<dyn shared::BearerTokenProvider>);
/// impl local::BearerTokenProvider for SharedAsLocalBearerTokenProvider { /* delegates */ }
///
/// // 4. Zero-sized registration struct — used as the type parameter in
/// //    require_local::<BearerTokenProvider>() / require_shared::<BearerTokenProvider>().
/// pub struct BearerTokenProvider;
///
/// // 5. Sealed trait impls — prevents external crates from adding capabilities.
/// impl CapabilitySealed for BearerTokenProvider {}
/// impl ExtensionCapability for BearerTokenProvider {
///     const NAME: &'static str = "bearer_token_provider";
///     type Local = dyn local::BearerTokenProvider;
///     type Shared = dyn shared::BearerTokenProvider;
///     fn adapt_shared_to_local(...) -> Option<Rc<dyn Any>> { /* wraps in adapter */ }
/// }
///
/// // 6. KNOWN_CAPABILITIES entry — link-time registration for config validation.
/// #[distributed_slice(KNOWN_CAPABILITIES)]
/// static _KNOWN_CAP_BEARER_TOKEN_PROVIDER: KnownCapability = KnownCapability {
///     name: "bearer_token_provider",
///     description: "Provides bearer tokens",
///     type_id: || TypeId::of::<BearerTokenProvider>(),
///     adapt_shared_to_local: ...,
/// };
/// ```
///
/// # Consuming capabilities
///
/// Node factories receive `&Capabilities` and resolve capabilities by the
/// registration struct:
///
/// ```rust,ignore
/// // Local consumer — returns Rc<dyn local::BearerTokenProvider>
/// let auth = capabilities.require_local::<BearerTokenProvider>()?;
///
/// // Shared consumer — returns Box<dyn shared::BearerTokenProvider>
/// let auth = capabilities.require_shared::<BearerTokenProvider>()?;
/// ```
///
/// # Important
///
/// Each capability must be defined in its own file under `capability/` to
/// avoid `mod local` / `mod shared` name collisions. The macro generates
/// `crate::capability::*` paths, so it can only be invoked from within the
/// `otap-df-engine` crate.
#[proc_macro_attribute]
pub fn capability(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as capability::CapabilityArgs);
    let trait_item = parse_macro_input!(input as ItemTrait);
    capability::expand_capability(args, trait_item).into()
}
