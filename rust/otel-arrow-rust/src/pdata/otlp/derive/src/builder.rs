// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::common;
use super::create_ident;
use super::message_info::MessageInfo;

/// Emits the builders, new(), and finish() methods.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let builder_name = msg.related_typename("Builder");

    // Generate generic type parameters names like ["T1", "T2", ...]
    let type_params = common::generic_type_names(msg.all_fields.len());

    // Generate a list of arguments to pass from build() to new().
    let param_args = common::builder_argument_list(&msg.param_fields);

    // Generate parameter declarations and where bounds
    let (param_decls, param_bounds) =
        common::builder_formal_parameters(&msg.param_fields, &type_params);

    // Generate field assignments and initializers
    let (field_assignments, field_initializers): (TokenVec, TokenVec) = msg
        .all_fields
        .iter()
        .map(common::builder_field_assignment)
        .unzip();

    // Default initializers for fields
    let default_initializers: TokenVec = msg
        .all_fields
        .iter()
        .map(common::builder_default_initializer)
        .collect();

    // All field initializers includes parameters and defaults
    let all_field_initializers: Vec<_> = (0..msg.all_fields.len())
        .map(|idx| {
            if idx < msg.param_names.len() {
                field_initializers[idx].clone()
            } else {
                default_initializers[idx].clone()
            }
        })
        .collect();

    // Generate builder methods for all non-parameter fields
    let builder_methods: TokenVec = msg
        .builder_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            let field_type = &info.full_type_name;

            // Find the corresponding field assignment by matching field name
            let field_assignment = msg.all_fields
                .iter()
                .position(|f| f.ident == info.ident)
                .map(|idx| field_assignments[idx].clone())
                .unwrap_or_else(|| {
                    // Fallback assignment for cases where position lookup fails
                    if info.proto_type == "enumeration" {
                        match (info.is_optional, &info.as_type) {
                            (true, Some(as_type)) => quote! { self.inner.#field_name = Some(#field_name as #as_type); },
                            (true, None) => quote! { self.inner.#field_name = Some(#field_name.into()); },
                            (false, Some(as_type)) => quote! { self.inner.#field_name = #field_name as #as_type; },
                            (false, None) => quote! { self.inner.#field_name = #field_name.into(); },
                        }
                    } else {
                        match (info.is_optional, &info.as_type) {
                            (true, Some(as_type)) => quote! { self.inner.#field_name = Some(#field_name.into() as #as_type); },
                            (true, None) => quote! { self.inner.#field_name = Some(#field_name.into()); },
                            (false, Some(as_type)) => quote! { self.inner.#field_name = #field_name.into() as #as_type; },
                            (false, None) => quote! { self.inner.#field_name = #field_name.into(); },
                        }
                    }
                });

            // For fields with enum types, use the enum type for the constraint
            // For other fields, use the field type
            let constraint_type = if let Some(ref enum_type) = info.enum_type {
                enum_type.clone()
            } else {
                field_type.clone()
            };

            quote! {
                pub fn #field_name<T: Into<#constraint_type>>(mut self, #field_name: T) -> Self
                {
                    #field_assignment
                    self
                }
            }
        })
        .collect();

    // When there are no builder fields, we can skip the builder struct.
    let derive_builder = !msg.builder_fields.is_empty();

    // Function to build constructors used in oneof and normal cases.
    let create_constructor =
        |suffix: String,
         cur_param_bounds: &[proc_macro2::TokenStream],
         cur_param_decls: &[proc_macro2::TokenStream],
         cur_param_args: &[proc_macro2::TokenStream],
         cur_field_initializers: &[proc_macro2::TokenStream]| {
            let build_name = create_ident(&format!("build{}", suffix));
            let new_name = create_ident(&format!("new{}", suffix));

            let mut cons = quote! {
            pub fn #new_name<#(#cur_param_bounds),*>(#(#cur_param_decls),*) -> Self {
                        Self{
                #(#cur_field_initializers)*
                }
            }
            };
            if derive_builder {
                cons.extend(quote! {
                pub fn #build_name<#(#cur_param_bounds),*>(#(#cur_param_decls),*) -> #builder_name {
                            #builder_name{
                    inner: #outer_name::#new_name(#(#cur_param_args),*),
                            }
                }
                });
            }
            cons
        };

    // Build constructors for both regular and oneof cases.
    let all_constructors: TokenVec = match msg.oneof_mapping.as_ref() {
        None => {
            vec![create_constructor(
                "".to_string(),
                &param_bounds,
                &param_decls,
                &param_args,
                &all_field_initializers,
            )]
        }
        Some(oneof_mapping) => common::builder_oneof_constructors(
            oneof_mapping,
            &msg.param_names,
            &param_bounds,
            &param_decls,
            &param_args,
            &all_field_initializers,
            &type_params,
            &create_constructor,
        ),
    };

    // Produce expanded implementation
    let mut expanded = quote! {
            impl #outer_name {
        #(#all_constructors)*
        }
    };

    if derive_builder {
        expanded.extend(quote! {
                pub struct #builder_name {
                    inner: #outer_name,
                }

                impl #builder_name {
                    #(#builder_methods)*

                    pub fn finish(self) -> #outer_name {
                        self.inner
                    }
                }

                impl std::convert::From<#builder_name> for #outer_name {
                    fn from(builder: #builder_name) -> Self {
                        builder.finish()
                    }
                }
        });
    }

    TokenStream::from(expanded)
}
