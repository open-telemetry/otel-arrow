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

    // Validate: either all fields are parameters or no fields are parameters
    // Ignored fields are excluded from this count
    let param_count = msg.param_names.len();
    let ignored_count = msg.ignored_names.len();
    let all_field_count = msg.all_fields.len();
    let expected_param_count = all_field_count - ignored_count;

    if param_count != 0 && param_count != expected_param_count {
        panic!(
            "Type '{}' must have either all non-ignored fields as parameters ({}) or no parameters (0), but has {} (with {} ignored)",
            outer_name, expected_param_count, param_count, ignored_count
        );
    }

    // Determine the mode: true = use new() constructors, false = use build() + builder pattern
    let use_constructors = param_count > 0;

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
    // Exclude oneof fields that are not in params, since they get variant-specific methods instead
    let builder_methods: TokenVec = msg
        .builder_fields
        .iter()
        .filter(|info| {
            // If this field is a oneof field (has oneof cases) and is not a parameter,
            // exclude it from regular builder methods since we generate variant methods
            info.oneof.is_none() || info.is_param
        })
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

    // Function to build constructors - only generates new() methods in constructor mode
    let create_constructor =
        |suffix: String,
         cur_param_bounds: &[proc_macro2::TokenStream],
         cur_param_decls: &[proc_macro2::TokenStream],
         _cur_param_args: &[proc_macro2::TokenStream],
         cur_field_initializers: &[proc_macro2::TokenStream]| {
            let new_name = create_ident(&format!("new{}", suffix));

            quote! {
                pub fn #new_name<#(#cur_param_bounds),*>(#(#cur_param_decls),*) -> Self {
                    Self{
                        #(#cur_field_initializers)*
                    }
                }
            }
        };

    // Generate setter methods for ignored fields
    // These methods consume self and return Self for fluent API
    let ignored_setters: TokenVec = msg
        .all_fields
        .iter()
        .filter(|info| msg.ignored_names.contains(&info.ident.to_string()))
        .map(|info| {
            let field_name = &info.ident;
            let setter_name = create_ident(&format!("set_{}", field_name));
            let field_type = &info.full_type_name;

            quote! {
                pub fn #setter_name<T: Into<#field_type>>(mut self, #field_name: T) -> Self {
                    self.#field_name = #field_name.into();
                    self
                }
            }
        })
        .collect();

    // Generate either constructors (mode 1) or builder pattern (mode 2), never both
    let expanded = if use_constructors {
        // Mode 1: Generate new() constructors (with oneof variants if applicable)
        let all_constructors = match msg.oneof_mapping.as_ref() {
            None => {
                vec![create_constructor(
                    "".to_string(),
                    &param_bounds,
                    &param_decls,
                    &param_args,
                    &all_field_initializers,
                )]
            }
            Some(oneof_mapping) => {
                // All fields are parameters, so oneof must be in parameters
                common::builder_oneof_constructors(
                    oneof_mapping,
                    &msg.param_names,
                    &param_bounds,
                    &param_decls,
                    &param_args,
                    &all_field_initializers,
                    &type_params,
                    create_constructor,
                )
            }
        };

        quote! {
            impl #outer_name {
                #(#all_constructors)*
                #(#ignored_setters)*
            }
        }
    } else {
        // Mode 2: Generate build() + builder pattern with setter methods
        let oneof_builder_methods = msg
            .oneof_mapping
            .as_ref()
            .map(common::builder_oneof_methods)
            .unwrap_or_default();

        quote! {
            impl #outer_name {
                pub fn build() -> #builder_name {
                    #builder_name {
                        inner: Self {
                            #(#all_field_initializers)*
                        },
                    }
                }

                #(#ignored_setters)*
            }

            pub struct #builder_name {
                inner: #outer_name,
            }

            impl #builder_name {
                #(#builder_methods)*
                #(#oneof_builder_methods)*

                pub fn finish(self) -> #outer_name {
                    self.inner
                }
            }

            impl std::convert::From<#builder_name> for #outer_name {
                fn from(builder: #builder_name) -> Self {
                    builder.finish()
                }
            }
        }
    };

    TokenStream::from(expanded)
}
