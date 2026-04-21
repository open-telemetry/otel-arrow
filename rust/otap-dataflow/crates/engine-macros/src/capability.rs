// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the `#[capability]` proc macro.
//!
//! Given a trait definition annotated with `#[capability(name = "...", description = "...")]`,
//! generates:
//!
//! - `local::<TraitName>` trait (`#[async_trait(?Send)]`)
//! - `shared::<TraitName>` trait (`#[async_trait]`, `: Send`). `Sync` is not
//!   required by the trait; it is only imposed at the impl site if a method
//!   signature (e.g. `async fn foo(&self)`) forces it.
//! - `SharedAsLocal<TraitName>` adapter struct
//! - Zero-sized `<TraitName>` registration struct
//! - `Sealed` + `ExtensionCapability` impls
//! - `KNOWN_CAPABILITIES` distributed slice entry
//!
//! # Supported
//!
//! - Methods with `&self` receiver (sync and async)
//! - Method-level generics, lifetimes, and where clauses
//! - Default method bodies (preserved in generated local/shared traits)
//! - Doc attributes on the trait (propagated to generated traits)
//! - Visibility modifiers on the trait
//!
//! # Unsupported (rejected with compile-time errors)
//!
//! These limitations are fundamental to the type-erased `HashMap<TypeId, Entry>`
//! registry design and the `SharedAsLocal` adapter delegation pattern:
//!
//! - **Trait-level generics / lifetimes** — `TypeId::of::<T>()` requires a
//!   concrete monomorphized type; a generic registration struct cannot produce
//!   a single `TypeId` for the registry.
//! - **Supertraits** (`trait Foo: Bar`) — the `SharedAsLocal` adapter only
//!   delegates methods defined directly on the `#[capability]` trait. It cannot
//!   auto-implement supertrait methods. Define all methods directly on the
//!   capability trait instead.
//! - **Associated types** — the type-erased `Box<dyn Any>` / downcast pattern
//!   requires knowing the concrete associated type at compile time. Different
//!   implementations could have different associated types, making a single
//!   registry entry impossible.
//! - **Associated constants** — same fundamental issue as associated types.
//!
//! # Generated code paths
//!
//! The macro generates `crate::capability::*` paths, so it must be invoked
//! from within the `otap-df-engine` crate. Each capability should be defined
//! in its own file under `capability/` to avoid `mod local`/`mod shared`
//! name collisions.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Ident, ItemTrait, LitStr, Meta, TraitItem, TraitItemFn,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

/// Parsed arguments from `#[capability(name = "...", description = "...")]`.
pub(crate) struct CapabilityArgs {
    pub name: LitStr,
    pub description: LitStr,
}

impl Parse for CapabilityArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut name: Option<LitStr> = None;
        let mut description: Option<LitStr> = None;

        let metas = Punctuated::<Meta, Comma>::parse_terminated(input)?;
        for meta in metas {
            if let Meta::NameValue(nv) = meta {
                let key = nv
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                match key.as_str() {
                    "name" => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = nv.value
                        {
                            name = Some(s);
                        }
                    }
                    "description" => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = nv.value
                        {
                            description = Some(s);
                        }
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            "unknown capability attribute; expected `name` or `description`",
                        ));
                    }
                }
            }
        }

        let name =
            name.ok_or_else(|| input.error("missing required `name = \"...\"` attribute"))?;
        let description = description
            .ok_or_else(|| input.error("missing required `description = \"...\"` attribute"))?;

        Ok(CapabilityArgs { name, description })
    }
}

/// Validate that the trait definition is within the supported subset.
///
/// Returns a compile error if unsupported features are used.
fn validate_trait(trait_item: &ItemTrait) -> Result<(), TokenStream> {
    // Reject trait-level generics (including lifetimes).
    if !trait_item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &trait_item.generics,
            "#[capability] does not support trait-level generics or lifetimes; \
             use method-level generics instead",
        )
        .to_compile_error());
    }

    // Reject supertraits (the SharedAsLocal adapter cannot auto-delegate supertrait methods).
    if !trait_item.supertraits.is_empty() {
        return Err(syn::Error::new_spanned(
            &trait_item.supertraits,
            "#[capability] does not support supertraits; \
             define all methods directly on the capability trait",
        )
        .to_compile_error());
    }

    // Reject associated types and consts.
    for item in &trait_item.items {
        match item {
            TraitItem::Type(t) => {
                return Err(syn::Error::new_spanned(
                    t,
                    "#[capability] does not support associated types; \
                     use concrete types in method signatures instead",
                )
                .to_compile_error());
            }
            TraitItem::Const(c) => {
                return Err(syn::Error::new_spanned(
                    c,
                    "#[capability] does not support associated constants",
                )
                .to_compile_error());
            }
            _ => {}
        }
    }

    Ok(())
}

/// Extract only the method items from a trait.
fn trait_methods(trait_item: &ItemTrait) -> Vec<&TraitItemFn> {
    trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let TraitItem::Fn(f) = item {
                Some(f)
            } else {
                None
            }
        })
        .collect()
}

/// Generate the full output for a `#[capability(...)]` annotation.
pub(crate) fn expand_capability(args: CapabilityArgs, trait_item: ItemTrait) -> TokenStream {
    if let Err(err) = validate_trait(&trait_item) {
        return err;
    }

    let trait_name = &trait_item.ident;
    let vis = &trait_item.vis;
    let cap_name_str = &args.name;
    let description_str = &args.description;
    let methods = trait_methods(&trait_item);

    // Names for generated items
    let shared_as_local_name = format_ident!("SharedAsLocal{}", trait_name);
    // Convert CamelCase to SCREAMING_SNAKE_CASE for the static name.
    let mut static_suffix = String::new();
    for (i, c) in trait_name.to_string().chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            static_suffix.push('_');
        }
        static_suffix.push(c.to_ascii_uppercase());
    }
    let known_cap_static = format_ident!("_KNOWN_CAP_{}", static_suffix);

    // Generate method signatures for local trait (#[async_trait(?Send)])
    // Methods with default bodies preserve them; methods without get `;`.
    let local_methods: Vec<TokenStream> = methods
        .iter()
        .map(|m| {
            let sig = &m.sig;
            let attrs = &m.attrs;
            if let Some(body) = &m.default {
                quote! { #(#attrs)* #sig #body }
            } else {
                quote! { #(#attrs)* #sig; }
            }
        })
        .collect();

    // Generate method signatures for shared trait (#[async_trait] + Send)
    let shared_methods: Vec<TokenStream> = methods
        .iter()
        .map(|m| {
            let sig = &m.sig;
            let attrs = &m.attrs;
            if let Some(body) = &m.default {
                quote! { #(#attrs)* #sig #body }
            } else {
                quote! { #(#attrs)* #sig; }
            }
        })
        .collect();

    // Generate SharedAsLocal adapter delegation methods
    let adapter_methods: Vec<TokenStream> = methods
        .iter()
        .map(|m| {
            let sig = &m.sig;
            let fn_name = &m.sig.ident;
            let attrs = &m.attrs;

            // Collect parameter names (skip self)
            let param_names: Vec<&Ident> = m
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            Some(&pat_ident.ident)
                        } else {
                            // Non-ident patterns (e.g., destructuring) would
                            // silently break delegation. This can't happen for
                            // capability traits validated above (no associated
                            // types, simple &self methods), but panic defensively.
                            panic!("#[capability] adapter delegation requires simple ident parameters, got a pattern")
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let is_async = m.sig.asyncness.is_some();
            let call = if is_async {
                quote! { self.0.#fn_name(#(#param_names),*).await }
            } else {
                quote! { self.0.#fn_name(#(#param_names),*) }
            };

            quote! {
                #(#attrs)*
                #sig {
                    #call
                }
            }
        })
        .collect();

    // Collect doc attrs from the original trait
    let trait_docs: Vec<_> = trait_item
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .collect();

    quote! {
        /// Local (!Send) version of the capability trait.
        #vis mod local {
            use super::*;

            #(#trait_docs)*
            #[::async_trait::async_trait(?Send)]
            pub trait #trait_name {
                #(#local_methods)*
            }
        }

        /// Shared (Send) version of the capability trait.
        #vis mod shared {
            use super::*;

            #(#trait_docs)*
            #[::async_trait::async_trait]
            pub trait #trait_name: Send {
                #(#shared_methods)*
            }
        }

        /// `SharedAsLocal` adapter — wraps a shared implementation and
        /// exposes it as a local trait object.
        struct #shared_as_local_name(Box<dyn shared::#trait_name>);

        #[::async_trait::async_trait(?Send)]
        impl local::#trait_name for #shared_as_local_name {
            #(#adapter_methods)*
        }

        /// Zero-sized registration struct for the capability.
        ///
        /// Used as the type parameter in
        /// [`Capabilities::require_local`](crate::capability::registry::Capabilities::require_local)
        /// and
        /// [`Capabilities::require_shared`](crate::capability::registry::Capabilities::require_shared).
        #vis struct #trait_name;

        // Seals `ExtensionCapability` so only `#[capability]`-generated
        // types can implement it (prevents external impls / misuse).
        impl crate::capability::CapabilitySealed for #trait_name {}

        // Wires the zero-sized registration struct into the capability
        // system: exposes the capability name, the local/shared trait
        // object types, and the adapter that turns a shared impl into a
        // local trait object (used by the registry for resolve-time fan-out).
        impl crate::capability::ExtensionCapability for #trait_name {
            const NAME: &'static str = #cap_name_str;
            type Local = dyn local::#trait_name;
            type Shared = dyn shared::#trait_name;

            fn adapt_shared_to_local(
                factory: &dyn crate::capability::registry::SharedCapabilityFactory,
            ) -> Option<::std::rc::Rc<dyn ::std::any::Any>> {
                let boxed_any = factory.produce_any();
                // `produce_any` MUST return Box<Box<dyn shared::Trait>> (double-boxed).
                // If this panics, the factory was constructed with the wrong boxing
                // convention — this is always a bug in the registration code.
                let boxed_shared: Box<dyn shared::#trait_name> =
                    *boxed_any.downcast::<Box<dyn shared::#trait_name>>().expect(
                        concat!(
                            "BUG: factory for capability '", #cap_name_str,
                            "' must produce Box<Box<dyn shared::", stringify!(#trait_name),
                            ">>; got a different type — check the SharedCapabilityEntry registration",
                        )
                    );
                let adapter = #shared_as_local_name(boxed_shared);
                let rc_local: ::std::rc::Rc<dyn local::#trait_name> =
                    ::std::rc::Rc::new(adapter);
                Some(::std::rc::Rc::new(rc_local))
            }
        }

        // Registers the capability in the `KNOWN_CAPABILITIES` distributed
        // slice at link time, so the engine can enumerate all capabilities
        // compiled into the binary (by name, description, and TypeId) without
        // needing an explicit registration call.
        #[::linkme::distributed_slice(crate::capability::KNOWN_CAPABILITIES)]
        #[linkme(crate = ::linkme)]
        static #known_cap_static: crate::capability::KnownCapability =
            crate::capability::KnownCapability {
                name: #cap_name_str,
                description: #description_str,
                type_id: || ::std::any::TypeId::of::<#trait_name>(),
                adapt_shared_to_local: <#trait_name as crate::capability::ExtensionCapability>::adapt_shared_to_local,
            };
    }
}
