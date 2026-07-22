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
//! The generated `local` / `shared` trait modules are emitted as `pub(crate)`,
//! not `pub`: they are an implementation detail, so the capability module can be
//! a plain public module exposing only its data types and registration handle
//! directly, while the trait variants are made public solely through the
//! hand-written `{local,shared}::capability::<name>` re-exports. This keeps each
//! item on a single public surface (no capability trait is also reachable at
//! `capability::<name>::{local,shared}`).
//!
//! # Supported
//!
//! - Methods with `&self` or `&mut self` receivers (sync and async;
//!   explicit lifetimes like `&'a self` are also accepted)
//! - Method-level lifetime parameters and where clauses
//! - Default method bodies (preserved in generated local/shared traits)
//! - Doc attributes on the trait (propagated to generated traits)
//! - Visibility modifiers on the trait
//!
//! # Unsupported (rejected with compile-time errors)
//!
//! These limitations are fundamental to the type-erased `HashMap<TypeId, Entry>`
//! registry design and the `SharedAsLocal` adapter delegation pattern:
//!
//! - **Trait-level generics or lifetime parameters** — e.g.
//!   `#[capability] trait Foo<T>` or `#[capability] trait Bar<'a>`.
//!   Method-level lifetimes (one bullet up) *are* supported. Trait-level
//!   parameters, by contrast, mean the trait isn't one type but a
//!   *family* — `Foo<u32>`, `Foo<String>`, etc. — each with its own
//!   `TypeId::of::<Foo<T>>()`. The registry keys entries by a single
//!   `TypeId` per capability, so there is no monomorphized type for the
//!   generated registration struct to advertise. Concretize the
//!   parameter at the trait definition (e.g. work over `String` or a
//!   sealed enum) or split the family into separate `#[capability]`
//!   traits.
//! - **Method-level generic type or const parameters** — e.g.
//!   `fn get<T>(&self, key: T) -> T` or `fn pad<const N: usize>(&self)`.
//!   The macro generates `Box<dyn local::Trait>` / `Box<dyn shared::Trait>`
//!   handles, and dyn-compatibility (object safety) forbids generic type
//!   or const parameters on dispatchable methods because each
//!   monomorphization would need its own vtable slot. Use a concrete type
//!   (or a sealed enum) at the trait method signature instead. Method-level
//!   *lifetime* parameters are accepted.
//! - **Supertraits** (`trait Foo: Bar`) — the `SharedAsLocal` adapter only
//!   delegates methods defined directly on the `#[capability]` trait. It cannot
//!   auto-implement supertrait methods. Define all methods directly on the
//!   capability trait instead.
//! - **Associated types** — the type-erased `Box<dyn Any>` / downcast pattern
//!   requires knowing the concrete associated type at compile time. Different
//!   implementations could have different associated types, making a single
//!   registry entry impossible.
//! - **Associated constants** — same fundamental issue as associated types.
//! - **Receiver shapes other than `&self` / `&mut self`** — rejected at the
//!   macro level with a fail-fast diagnostic. Specifically:
//!     - No `self` associated functions (`fn foo()`) — not dispatchable through
//!       `dyn Trait`.
//!     - Consuming `self` (`fn foo(self)`) — non-object-safe; the
//!       `SharedAsLocal` adapter holds `Box<dyn shared::Trait>` and cannot
//!       call methods that consume the trait object.
//!     - Arbitrary self types (`self: Box<Self>`, `self: Arc<Self>`, …) — the
//!       adapter delegates through a `Box<dyn shared::Trait>` field and only
//!       knows how to forward `&self` / `&mut self` receivers.
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
    FnArg, GenericParam, Ident, ItemTrait, LitStr, Meta, TraitItem, TraitItemFn,
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
    // Reject trait-level generics (including lifetimes). Method-level
    // lifetimes are accepted; method-level type/const parameters are
    // rejected separately below with their own diagnostic.
    if !trait_item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &trait_item.generics,
            "#[capability] does not support trait-level generics or lifetimes; \
             method-level lifetimes are supported, but type or const parameters \
             must be expressed with concrete method signatures (or by splitting \
             the family into separate capability traits)",
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
            TraitItem::Fn(f) => {
                // Reject unsupported receiver shapes up front so the
                // diagnostic points at the bad receiver rather than at
                // generated `dyn Trait` / adapter code far from the
                // capability definition.
                //
                // Supported: `&self`, `&mut self` (with or without an
                //            explicit lifetime, e.g. `&'a self`).
                // Rejected: no `self` associated functions, consuming
                //           `self`, and arbitrary self types
                //           (`self: Box<Self>`, `self: Arc<Self>`, …).
                match f.sig.inputs.first() {
                    Some(FnArg::Receiver(recv)) => {
                        if recv.colon_token.is_some() {
                            // Typed receiver such as `self: Box<Self>`.
                            return Err(syn::Error::new_spanned(
                                recv,
                                "#[capability] does not support arbitrary self types \
                                 (e.g. `self: Box<Self>`, `self: Arc<Self>`); the \
                                 SharedAsLocal adapter delegates through a `Box<dyn \
                                 shared::Trait>` field and can only forward `&self` / \
                                 `&mut self` receivers",
                            )
                            .to_compile_error());
                        }
                        if recv.reference.is_none() {
                            // Consuming `self`.
                            return Err(syn::Error::new_spanned(
                                recv,
                                "#[capability] does not support methods that consume \
                                 `self`; the SharedAsLocal adapter holds a \
                                 `Box<dyn shared::Trait>` and cannot call methods \
                                 that take `self` by value. Use `&self` or `&mut self` \
                                 instead.",
                            )
                            .to_compile_error());
                        }
                    }
                    _ => {
                        // First input is not a receiver — either no inputs
                        // at all or `fn foo(arg: T)` style associated fn.
                        return Err(syn::Error::new_spanned(
                            &f.sig,
                            "#[capability] does not support associated functions \
                             without a `self` receiver; every method must take \
                             `&self` or `&mut self` because the registry dispatches \
                             through `Box<dyn local::Trait>` / `Box<dyn shared::Trait>`",
                        )
                        .to_compile_error());
                    }
                }

                // Reject method-level generic type and const parameters.
                // The macro generates `Box<dyn local::Trait>` /
                // `Box<dyn shared::Trait>` handles, and dyn-compatibility
                // (object safety) forbids generic type or const parameters
                // on dispatchable methods. Lifetimes are fine — they
                // don't affect dyn-compatibility.
                for gp in &f.sig.generics.params {
                    match gp {
                        GenericParam::Type(_) | GenericParam::Const(_) => {
                            return Err(syn::Error::new_spanned(
                                gp,
                                "#[capability] does not support method-level generic type \
                                 or const parameters; the generated `Box<dyn Trait>` handles \
                                 require dyn-compatibility. Use a concrete type (or sealed \
                                 enum) in the method signature instead.",
                            )
                            .to_compile_error());
                        }
                        GenericParam::Lifetime(_) => {}
                    }
                }

                // Reject non-ident parameter patterns (e.g. destructured
                // arguments like `(a, b): (u64, u64)`). The adapter
                // delegation requires simple ident parameters so it can
                // forward by name. Catching this here gives a clear
                // compile error instead of a proc-macro panic deeper in
                // codegen.
                for arg in &f.sig.inputs {
                    if let FnArg::Typed(pat_type) = arg
                        && !matches!(&*pat_type.pat, syn::Pat::Ident(_))
                    {
                        return Err(syn::Error::new_spanned(
                            &pat_type.pat,
                            "#[capability] requires simple identifier parameters; \
                             destructured patterns (e.g. `(a, b): (u64, u64)`) are not \
                             supported because the generated adapter delegates by parameter name.",
                        )
                        .to_compile_error());
                    }
                }
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

/// Emit a trait-method signature for a generated `local::` / `shared::`
/// trait: keep the attributes, preserve a default body if present, or
/// emit a `;`-terminated signature otherwise. Both generated traits
/// want the same shape \u2014 the only difference between them is the
/// `#[async_trait]` vs `#[async_trait(?Send)]` outer attribute added by
/// the caller.
fn emit_method(m: &TraitItemFn) -> TokenStream {
    let sig = &m.sig;
    let attrs = &m.attrs;
    if let Some(body) = &m.default {
        quote! { #(#attrs)* #sig #body }
    } else {
        quote! { #(#attrs)* #sig; }
    }
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

    // Generate method signatures for the local trait (#[async_trait(?Send)])
    // and the shared trait (#[async_trait] + Send). Shape is identical
    // between the two; only the outer async_trait attribute differs.
    let local_methods: Vec<TokenStream> = methods.iter().map(|m| emit_method(m)).collect();
    let shared_methods: Vec<TokenStream> = methods.iter().map(|m| emit_method(m)).collect();

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
                    if let FnArg::Typed(pat_type) = arg {
                        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            Some(&pat_ident.ident)
                        } else {
                            // validate_trait rejects non-ident parameter
                            // patterns up-front with a syn::Error, so this
                            // branch is unreachable in practice. Kept as a
                            // belt-and-suspenders panic in case a future
                            // change to validation lets one slip through.
                            unreachable!(
                                "#[capability] adapter delegation requires simple ident parameters; \
                                 should have been rejected by validate_trait"
                            )
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
        ///
        /// `pub(crate)`: reachable publicly only through the hand-written
        /// `local::capability::<name>` re-export, never directly under
        /// `capability::<name>`, so the trait has a single public surface.
        pub(crate) mod local {
            use super::*;

            #(#trait_docs)*
            #[::async_trait::async_trait(?Send)]
            pub trait #trait_name {
                #(#local_methods)*
            }
        }

        /// Shared (Send) version of the capability trait.
        ///
        /// `pub(crate)`: reachable publicly only through the hand-written
        /// `shared::capability::<name>` re-export, never directly under
        /// `capability::<name>`, so the trait has a single public surface.
        pub(crate) mod shared {
            use super::*;

            #(#trait_docs)*
            #[::async_trait::async_trait]
            pub trait #trait_name: Send {
                #(#shared_methods)*
            }
        }

        /// `SharedAsLocal` adapter — wraps a shared implementation and
        /// exposes it as a local trait object.
        struct #shared_as_local_name(::std::boxed::Box<dyn shared::#trait_name>);

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

        // Macro-generated bridges between an extension's
        // `SharedInstanceFactory` / `LocalInstanceFactory` and the
        // capability registry. The `extension_capabilities!` macro calls
        // these with a clone of the instance factory per capability.
        //
        // The `where E: shared::#trait_name` / `E: local::#trait_name`
        // bound is the compile-time assertion: an extension that lists a
        // capability it doesn't implement fails to compile at the
        // `extension_capabilities!` call site with a clear "trait not
        // satisfied" message pointing at `E`.
        impl #trait_name {
            /// Build a registry entry bridging an extension's
            /// `SharedInstanceFactory` to this capability's shared trait
            /// object.
            ///
            /// The returned entry's produce closure calls the stored
            /// instance factory, downcasts the erased
            /// `Box<dyn Any + Send>` to `Box<E>`, and coerces to
            /// `Box<dyn shared::#trait_name>` — under the double-box
            /// envelope the registry expects.
            #[allow(non_snake_case, clippy::missing_errors_doc)]
            #vis fn shared_entry<E>(
                extension_id: ::otap_df_config::ExtensionId,
                factory: crate::capability::SharedInstanceFactory,
            ) -> crate::capability::registry::SharedCapabilityEntry
            where
                E: shared::#trait_name + 'static,
            {
                let produce = move || -> ::std::boxed::Box<dyn ::std::any::Any + Send> {
                    let erased = factory.produce();
                    let concrete: ::std::boxed::Box<E> = erased
                        .downcast()
                        .expect("instance_factory produced wrong type for capability");
                    let shared: ::std::boxed::Box<dyn shared::#trait_name> = concrete;
                    ::std::boxed::Box::new(shared) as ::std::boxed::Box<dyn ::std::any::Any + Send>
                };

                let adapt_as_local: fn(
                    ::std::boxed::Box<dyn ::std::any::Any + Send>,
                ) -> ::std::boxed::Box<dyn ::std::any::Any> = |erased| {
                    let shared: ::std::boxed::Box<::std::boxed::Box<dyn shared::#trait_name>> =
                        erased
                            .downcast()
                            .expect("shared_entry produce closure returned wrong envelope");
                    let boxed_local = <#trait_name as crate::capability::ExtensionCapability>::
                        wrap_shared_as_local(*shared);
                    ::std::boxed::Box::new(boxed_local) as ::std::boxed::Box<dyn ::std::any::Any>
                };

                crate::capability::registry::SharedCapabilityEntry::new(
                    extension_id,
                    produce,
                    adapt_as_local,
                )
            }

            /// Build a registry entry bridging an extension's
            /// `LocalInstanceFactory` to this capability's local trait
            /// object.
            ///
            /// The entry's produce closure calls the stored instance
            /// factory, downcasts the erased `Box<dyn Any>` to `Box<E>`,
            /// coerces to `Box<dyn local::#trait_name>`, and re-erases
            /// under the double-`Box` envelope expected by the registry.
            #[allow(non_snake_case, clippy::missing_errors_doc)]
            #vis fn local_entry<E>(
                extension_id: ::otap_df_config::ExtensionId,
                factory: crate::capability::LocalInstanceFactory,
            ) -> crate::capability::registry::LocalCapabilityEntry
            where
                E: local::#trait_name + 'static,
            {
                let produce = move || -> ::std::boxed::Box<dyn ::std::any::Any> {
                    let erased = factory.produce();
                    let concrete: ::std::boxed::Box<E> = erased
                        .downcast()
                        .expect("instance_factory produced wrong type for capability");
                    let local: ::std::boxed::Box<dyn local::#trait_name> = concrete;
                    ::std::boxed::Box::new(local) as ::std::boxed::Box<dyn ::std::any::Any>
                };

                crate::capability::registry::LocalCapabilityEntry::new(extension_id, produce)
            }
        }

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

            fn wrap_shared_as_local(
                shared: ::std::boxed::Box<Self::Shared>,
            ) -> ::std::boxed::Box<Self::Local> {
                let adapter = #shared_as_local_name(shared);
                ::std::boxed::Box::new(adapter)
            }
        }

        // Registers the capability in the `KNOWN_CAPABILITIES` distributed
        // slice at link time, so the engine can enumerate all capabilities
        // compiled into the binary (by name, description, and TypeId) without
        // needing an explicit registration call.
        //
        // `#[allow(unsafe_code)]` is required because `linkme::distributed_slice`
        // emits a static with `#[link_section = "..."]`, which the engine
        // crate's lints (`-D unsafe-code`) would otherwise reject.
        #[allow(unsafe_code)]
        #[::linkme::distributed_slice(crate::capability::KNOWN_CAPABILITIES)]
        #[linkme(crate = ::linkme)]
        static #known_cap_static: crate::capability::KnownCapability =
            crate::capability::KnownCapability {
                name: #cap_name_str,
                description: #description_str,
                type_id: || ::std::any::TypeId::of::<#trait_name>(),
            };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a trait body and run it through `validate_trait`, returning the
    /// error message text on rejection or `None` on success.
    fn validate(src: &str) -> Option<String> {
        let trait_item: ItemTrait = syn::parse_str(src).expect("parse trait");
        validate_trait(&trait_item).err().map(|ts| ts.to_string())
    }

    #[test]
    fn accepts_ref_self() {
        assert!(validate("trait Cap { fn get(&self) -> u32; }").is_none());
    }

    #[test]
    fn accepts_ref_self_with_lifetime() {
        assert!(validate("trait Cap { fn get<'a>(&'a self) -> &'a str; }").is_none());
    }

    #[test]
    fn accepts_async_ref_self() {
        assert!(validate("trait Cap { async fn get(&self) -> u32; }").is_none());
    }

    #[test]
    fn accepts_default_method_body() {
        assert!(validate("trait Cap { fn get(&self) -> u32 { 0 } }").is_none());
    }

    #[test]
    fn accepts_mut_self_reference() {
        assert!(validate("trait Cap { fn set(&mut self); }").is_none());
    }

    #[test]
    fn accepts_mut_self_reference_with_lifetime() {
        assert!(validate("trait Cap { fn set<'a>(&'a mut self); }").is_none());
    }

    #[test]
    fn accepts_async_mut_self() {
        assert!(validate("trait Cap { async fn set(&mut self); }").is_none());
    }

    #[test]
    fn accepts_method_lifetime_param() {
        // Lifetime parameters do not affect dyn-compatibility, so they
        // are allowed.
        assert!(validate("trait Cap { fn get<'a>(&'a self) -> &'a str; }").is_none());
    }

    #[test]
    fn rejects_method_generic_type_param() {
        // Generic type parameters break dyn-compatibility of the
        // generated `Box<dyn local/shared::Trait>` handles.
        let err = validate("trait Cap { fn get<T>(&self, key: T) -> T; }").expect("should reject");
        assert!(
            err.contains("method-level generic type"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_method_generic_const_param() {
        // Const generics likewise break dyn-compatibility.
        let err = validate("trait Cap { fn pad<const N: usize>(&self) -> [u8; N]; }")
            .expect("should reject");
        assert!(
            err.contains("method-level generic type"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_destructured_parameter_pattern() {
        // Destructured arg patterns (e.g. `(a, b): (u64, u64)`) cannot
        // be forwarded by name through the SharedAsLocal adapter; we
        // reject them with a syn::Error rather than panicking deep in
        // codegen.
        let err =
            validate("trait Cap { fn set(&self, (a, b): (u64, u64)); }").expect("should reject");
        assert!(
            err.contains("simple identifier parameters"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_no_self_associated_fn() {
        let err = validate("trait Cap { fn make() -> u32; }").expect("should reject");
        assert!(
            err.contains("without a `self` receiver"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_consuming_self() {
        let err = validate("trait Cap { fn finish(self); }").expect("should reject");
        assert!(err.contains("consume `self`"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_typed_self_receiver_box() {
        let err = validate("trait Cap { fn run(self: Box<Self>); }").expect("should reject");
        assert!(
            err.contains("arbitrary self types"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_typed_self_receiver_arc() {
        let err =
            validate("trait Cap { fn run(self: std::sync::Arc<Self>); }").expect("should reject");
        assert!(
            err.contains("arbitrary self types"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn trait_level_generics_error_mentions_method_lifetimes() {
        // Regression for the previously misleading wording that just said
        // "use method-level generics instead" — method-level type/const
        // generics are also rejected, so the error should clarify that
        // only method-level lifetimes are supported.
        let err = validate("trait Cap<T> { fn get(&self) -> T; }").expect("should reject");
        assert!(
            err.contains("method-level lifetimes are supported"),
            "unexpected error: {err}"
        );
    }
}
