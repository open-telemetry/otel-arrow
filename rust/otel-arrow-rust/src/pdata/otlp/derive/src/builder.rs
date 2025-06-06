// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::create_ident;
use super::field_info::FieldInfo;
use super::message_info::MessageInfo;
use otlp_model::OneofCase;

/// Emits the builders, new(), and finish() methods.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let builder_name = msg.related_typename("Builder");

    // Generate generic type parameters names like ["T1", "T2", ...]
    let type_params: Vec<syn::Ident> = (0..msg.all_fields.len())
        .map(|idx| create_ident(&format!("T{}", idx + 1)))
        .collect();

    // Generate a list of arguments to pass from build() to new().
    let param_args: TokenVec = msg
        .param_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            quote! { #field_name }
        })
        .collect();

    // Generate parameter declarations and where bounds
    let (param_decls, param_bounds): (TokenVec, TokenVec) = msg
        .param_fields
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let param_name = &info.ident;
            let type_param = &type_params[idx];
            let target_type = &info.full_type_name;

            let decl = quote! { #param_name: #type_param };
            let bound = quote! { #type_param: Into<#target_type> };

            (decl, bound)
        })
        .unzip();

    // Generate field assignments and initializers
    let (field_assignments, field_initializers): (TokenVec, TokenVec) =
        msg.all_fields.iter().map(generate_field_assignment).unzip();

    // Default initializers for fields
    let default_initializers: TokenVec = msg
        .all_fields
        .iter()
        .map(generate_default_initializer)
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

    // Generate builder methods
    let builder_methods: TokenVec = msg
        .all_fields
        .iter()
        .enumerate()
        .filter(|(_, info)| info.oneof.is_some())
        .map(|(idx, info)| {
            let field_name = &info.ident;
            let field_type = &info.full_type_name;
            let value_assignment = field_assignments[idx].clone();

            quote! {
                pub fn #field_name<T: Into<#field_type>>(mut self, #field_name: T) -> Self
                {
                    #value_assignment
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
        Some(oneof_mapping) => generate_oneof_constructors(
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

/// Generate field assignment patterns for different field types
fn generate_field_assignment(
    info: &FieldInfo,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let field_name = &info.ident;

    match (info.is_optional, &info.as_type) {
        (true, Some(as_type)) => (
            quote! { self.inner.#field_name = Some(#field_name.into() as #as_type); },
            quote! { #field_name: Some(#field_name.into() as #as_type), },
        ),
        (true, None) => (
            quote! { self.inner.#field_name = Some(#field_name.into()); },
            quote! { #field_name: Some(#field_name.into()), },
        ),
        (false, Some(as_type)) => (
            quote! { self.inner.#field_name = #field_name.into() as #as_type; },
            quote! { #field_name: #field_name.into() as #as_type, },
        ),
        (false, None) => (
            quote! { self.inner.#field_name = #field_name.into(); },
            quote! { #field_name: #field_name.into(), },
        ),
    }
}

/// Generate default initializer for a field
fn generate_default_initializer(info: &FieldInfo) -> proc_macro2::TokenStream {
    let field_name = &info.ident;

    if info.is_optional {
        quote! { #field_name: None, }
    } else {
        match info.base_type_name.as_str() {
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                quote! { #field_name: 0, }
            }
            "f32" | "f64" => quote! { #field_name: 0.0, },
            "bool" => quote! { #field_name: false, },
            _ => quote! { #field_name: ::core::default::Default::default(), },
        }
    }
}

/// Generate constructor for a single oneof case
fn generate_oneof_constructor(
    case: &otlp_model::OneofCase,
    oneof_name: &str,
    oneof_idx: usize,
    param_bounds: &[proc_macro2::TokenStream],
    param_decls: &[proc_macro2::TokenStream],
    param_args: &[proc_macro2::TokenStream],
    all_field_initializers: &[proc_macro2::TokenStream],
    type_params: &[syn::Ident],
    create_constructor: &dyn Fn(
        String,
        &[proc_macro2::TokenStream],
        &[proc_macro2::TokenStream],
        &[proc_macro2::TokenStream],
        &[proc_macro2::TokenStream],
    ) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let case_type = syn::parse_str::<syn::Type>(case.type_param).unwrap();
    let variant_path = syn::parse_str::<syn::Expr>(case.value_variant).unwrap();
    let suffix = format!("_{}", case.name);
    let oneof_ident = syn::Ident::new(oneof_name, proc_macro2::Span::call_site());

    // Duplicate the param bounds and field initializers
    let mut cur_param_bounds = param_bounds.to_vec();
    let mut cur_field_initializers = all_field_initializers.to_vec();
    let type_param = &type_params[oneof_idx];

    let value_bound = quote! { #type_param: Into<#case_type> };
    let value_initializer = if let Some(extra_call) = &case.extra_call {
        let extra_call_path = syn::parse_str::<syn::Expr>(extra_call).unwrap();
        quote! {
            #oneof_ident: Some(#variant_path(#extra_call_path(#oneof_ident.into()))),
        }
    } else {
        quote! {
            #oneof_ident: Some(#variant_path(#oneof_ident.into())),
        }
    };

    // Replace the parameter with oneof-specific expansion
    cur_param_bounds[oneof_idx] = value_bound;
    cur_field_initializers[oneof_idx] = value_initializer;

    create_constructor(
        suffix,
        &cur_param_bounds,
        param_decls,
        param_args,
        &cur_field_initializers,
    )
}

/// Generate all constructors for a oneof mapping
pub fn generate_oneof_constructors(
    oneof_mapping: &(String, Vec<OneofCase>),
    param_names: &[String],
    param_bounds: &[proc_macro2::TokenStream],
    param_decls: &[proc_macro2::TokenStream],
    param_args: &[proc_macro2::TokenStream],
    all_field_initializers: &[proc_macro2::TokenStream],
    type_params: &[syn::Ident],
    create_constructor: &dyn Fn(
        String,
        &[proc_macro2::TokenStream],
        &[proc_macro2::TokenStream],
        &[proc_macro2::TokenStream],
        &[proc_macro2::TokenStream],
    ) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let (oneof_path, oneof_cases) = oneof_mapping;
    let oneof_name = oneof_path.split('.').last().unwrap();
    let oneof_idx = param_names
        .iter()
        .position(|name| name.as_str() == oneof_name)
        .unwrap();

    oneof_cases
        .iter()
        .map(|case| {
            generate_oneof_constructor(
                case,
                oneof_name,
                oneof_idx,
                param_bounds,
                param_decls,
                param_args,
                all_field_initializers,
                type_params,
                create_constructor,
            )
        })
        .collect()
}
