// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use super::TokenVec;
use super::message_info::MessageInfo;

/// Emits the visitor, visitable and adapters methods.
pub fn derive(msg: &MessageInfo) -> TokenStream {
    let outer_name = &msg.outer_name;
    let visitor_name = msg.related_typename("Visitor");
    let visitable_name = msg.related_typename("Visitable");
    let visitor_method_name = syn::Ident::new(
        &format!("visit_{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );
    let visitable_method_name = syn::Ident::new(
        &format!("accept_{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );

    let mut visitable_args: TokenVec = Vec::new();

    for info in &msg.all_fields {
        if let Some(oneof_cases) = info.oneof.as_ref() {
            // For oneof fields, generate separate parameters for each variant
            for case in oneof_cases {
                let variant_param_name =
                    syn::Ident::new(&format!("{}_{}", info.ident, case.name), info.ident.span());

                // TODO: Compute the visitor type name from the oneof case
                // Note probably need syn::parse_str::<syn::Type>(value)
                let visitor_type = &info.full_type_name;

                visitable_args.push(quote! { #variant_param_name: #visitor_type });
            }
            continue;
        }

        // For non-oneof fields, generate normal visitor parameter
        let param_name = &info.ident;
        let visitor_type = info.related_type("Visitable");
        visitable_args.push(quote! { #param_name: impl #visitor_type });
    }

    let expanded = quote! {
    pub trait #visitor_name<Argument> {
            fn #visitor_method_name(&mut self, arg: Argument, v: impl #visitable_name<Argument>) -> Argument;
    }

    pub trait #visitable_name<Argument> {
            fn #visitable_method_name(&self, arg: Argument, #(#visitable_args),*) -> Argument;
    }

    impl<Argument> #visitor_name<Argument> for crate::pdata::NoopVisitor {
            fn #visitor_method_name(&mut self, arg: Argument, _v: impl #visitable_name<Argument>) -> Argument {
        // NoopVisitor threads the argument through unchanged.
        arg
            }
    }
    };

    TokenStream::from(expanded)
}

/// Generate visitor call for a field with proper handling for different field types
pub fn generate_visitor_call(info: &FieldInfo) -> Option<proc_macro2::TokenStream> {
    // Oneof fields are handled separately in generate_oneof_visitor_calls
    if info.oneof.is_some() {
        return field_utils::generate_oneof_visitor_calls(info);
    }

    let field_name = &info.ident;
    let field_name_str = field_name.to_string();
    let clean_field_name = if field_name_str.starts_with("r#") {
        &field_name_str[2..]
    } else {
        &field_name_str
    };

    let visitor_param = ident_utils::visitor_param_name(&clean_field_name);

    let visit_method = generate_visit_method_for_field(info);
    let needs_adapter = needs_adapter_for_field(info);
    let is_bytes = type_utils::is_bytes_type(&info.field_type);

    match (info.is_optional, info.is_repeated, needs_adapter, is_bytes) {
        (false, false, true, _) => {
            let adapter_name = get_adapter_name_for_field(info);
            Some(quote! {
                arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(&self.data.#field_name)));
            })
        }
        (false, false, false, _) => {
            if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
            } else {
                Some(quote! { arg = #visitor_param.#visit_method(arg, *&self.data.#field_name); })
            }
        }
        (true, false, true, _) => {
            let adapter_name = get_adapter_name_for_field(info);
            Some(quote! {
                if let Some(f) = &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(f)));
                }
            })
        }
        (true, false, false, _) => {
            if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! {
                    if let Some(f) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, f);
                    }
                })
            } else {
                Some(quote! {
                    if let Some(f) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, *f);
                    }
                })
            }
        }
        (false, true, true, _) => {
            let adapter_name = get_adapter_name_for_field(info);
            Some(quote! {
                for item in &self.data.#field_name {
                    arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(item)));
                }
            })
        }
        (false, true, false, true) => {
            Some(quote! { arg = #visitor_param.#visit_method(arg, &self.data.#field_name); })
        }
        (false, true, false, false) => {
            if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! {
                    for item in &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, item);
                    }
                })
            } else {
                Some(quote! {
                    for item in &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, *item);
                    }
                })
            }
        }
        (true, true, _, _) => {
            if needs_adapter {
                let adapter_name = get_adapter_name_for_field(info);
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        for item in items {
                            arg = #visitor_param.#visit_method(arg, &(#adapter_name::new(item)));
                        }
                    }
                })
            } else if is_bytes {
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        arg = #visitor_param.#visit_method(arg, items);
                    }
                })
            } else if matches!(visit_method.to_string().as_str(), "visit_string") {
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        for item in items {
                            arg = #visitor_param.#visit_method(arg, item);
                        }
                    }
                })
            } else {
                Some(quote! {
                    if let Some(items) = &self.data.#field_name {
                        for item in items {
                            arg = #visitor_param.#visit_method(arg, *item);
                        }
                    }
                })
            }
        }
    }
}

/// Generate visitor calls for oneof fields based on their variants
pub fn generate_oneof_visitor_calls(
    info: &FieldInfo,
    oneof_mapping: &OneofMapping,
) -> Vec<proc_macro2::TokenStream> {
    let mut visitor_calls = Vec::new();

    if !info.is_oneof {
        return visitor_calls;
    }

    if let Some((oneof_name, oneof_cases)) = oneof_mapping {
        if oneof_name.ends_with(&format!(".{}", info.ident)) {
            let field_name = &info.ident;

            for case in oneof_cases {
                let variant_param_name = syn::Ident::new(
                    &format!("{}_{}_visitor", field_name, case.name),
                    field_name.span(),
                );

                // For now, generate a no-op visitor call that just threads the argument
                // This ensures we consume all the visitor parameters that were generated
                // TODO: Implement proper oneof variant matching and visiting
                let visitor_call = quote! {
                    // TODO: Implement oneof visitor call for #variant_param_name
                    let _ = &#variant_param_name; // Consume the parameter to avoid unused warnings
                };

                visitor_calls.push(visitor_call);
            }
        }
    }

    visitor_calls
}
