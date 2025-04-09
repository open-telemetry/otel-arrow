// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

/// Attribute macro for associating the OTLP protocol buffer fully
/// qualified type name.
#[proc_macro_attribute]
pub fn qualified(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str: String = args.to_string().trim_matches('"').into();
    let mut input_ast = syn::parse_macro_input!(input as syn::DeriveInput);

    // Create a special doc comment that will store the qualified name
    let qualified_attr = syn::parse_quote! {
        #[doc(hidden, otlp_qualified_name = #args_str)]
    };

    // Add the attribute to the struct
    input_ast.attrs.push(qualified_attr);

    // Return the modified struct definition
    quote::quote!(#input_ast).into()
}

/// Derives the OTLP Message trait implementation for protocol buffer
/// message types. This enables additional OTLP-specific functionality
/// beyond what prost::Message provides.
#[proc_macro_derive(Message)]
pub fn derive_otlp_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let outer_name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", outer_name), outer_name.span());

    // Get the fully qualified type name from attribute
    let type_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("doc") {
                // Use parse_nested_meta to extract the qualified name
                let mut qualified_name = None;
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("hidden") {
                        Ok(())
                    } else if meta.path.is_ident("otlp_qualified_name") {
                        let value = meta.value()?;
                        let lit: syn::LitStr = value.parse()?;
                        qualified_name = Some(lit.value());
                        Ok(())
                    } else {
                        Ok(())
                    }
                });
                qualified_name
            } else {
                None
            }
        })
        .unwrap();

    // Get optional details for the model.
    let detail = otlp_model::DETAILS
        .iter()
        .find(|detail| detail.name == type_name)
        .cloned()
        .unwrap_or(otlp_model::Detail::default());

    // Extract param names only from params or get an empty Vec if none exists
    let param_names = detail.params.unwrap_or_else(Vec::new);

    // Extract all fields from the struct definition
    let struct_fields = match &input.data {
        syn::Data::Struct(data) => {
            if let syn::Fields::Named(fields) = &data.fields {
                fields.named.iter().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    // If there are no fields, it's either an empty message or an enum,
    // either way should not be listed, no builder is needed.
    if struct_fields.len() == 0 {
	panic!("invalid Message derivation: {}", type_name);
    }

    // Function to check if a field is marked as optional
    let is_optional = |field: &syn::Field| {
        field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains("optional")
        })
    };

    // Find fields from struct definition that match the parameter names
    let param_fields = struct_fields
        .iter()
        .filter(|field| {
            field
                .ident
                .as_ref()
                .map_or(false, |id| param_names.contains(&id.to_string().as_str()))
        })
        .collect::<Vec<_>>();

    // Find fields that are not explicitly initialized via params
    let remaining_fields = struct_fields
        .iter()
        .filter(|field| {
            field
                .ident
                .as_ref()
                .map_or(false, |id| !param_names.contains(&id.to_string().as_str()))
        })
        .collect::<Vec<_>>();

    // Generate generic type parameters for each parameter
    let type_params: Vec<syn::Ident> = param_fields
        .iter()
        .enumerate()
        .map(|(idx, _)| syn::Ident::new(&format!("T{}", idx + 1), proc_macro2::Span::call_site()))
        .collect();

    // Generate parameters for new() function with generic type parameters
    let param_decls: Vec<_> = param_fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let param_name = field.ident.as_ref().unwrap();
            let type_param = &type_params[idx];

            quote! { #param_name: #type_param }
        })
        .collect();

    // Extract the target types for each parameter for the where bounds
    let param_target_types: Vec<_> = param_fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;

            // For optional fields, we want the inner type
            if is_optional(field) {
                // Extract inner type from Option<T> using a functional approach
                match field_type {
                    syn::Type::Path(type_path) => Some(type_path),
                    _ => None,
                }
                .and_then(|type_path| type_path.path.segments.last())
                .filter(|segment| segment.ident == "Option")
                .and_then(|segment| match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => Some(args),
                    _ => None,
                })
                .and_then(|args| args.args.first())
                .and_then(|arg| match arg {
                    syn::GenericArgument::Type(inner_type) => Some(inner_type.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| field_type.clone())
            } else {
                field_type.clone()
            }
        })
        .collect();

    // Generate the where bounds for the generic parameters
    let param_bounds: Vec<proc_macro2::TokenStream> = type_params
        .iter()
        .zip(param_target_types.iter())
        .map(|(type_param, target_type)| {
            quote! { #type_param: Into<#target_type> }
        })
        .collect();

    // Generate assignments in the new() function, using Into trait
    let param_assignments: Vec<_> = param_fields
        .iter()
        .map(|field| {
            let param_name = field.ident.as_ref().unwrap();
            let is_opt = is_optional(field);

            // Check if we need to convert the value using Into
            // Get field path to check for overrides
            let field_path = format!("{}.{}", type_name, param_name);
            let as_type = otlp_model::FIELD_TYPE_OVERRIDES
                .get(field_path.as_str())
                .map(|over| syn::parse_str::<syn::Type>(over.fieldtype).unwrap());

            if is_opt {
                if let Some(as_type) = as_type {
                    quote! { inner.#param_name = Some(#param_name.into() as #as_type); }
                } else {
                    quote! { inner.#param_name = Some(#param_name.into()); }
                }
            } else {
                if let Some(as_type) = as_type {
                    quote! { inner.#param_name = #param_name.into() as #as_type; }
                } else {
                    quote! { inner.#param_name = #param_name.into(); }
                }
            }
        })
        .collect();

    // Generate builder methods for remaining fields
    let builder_methods: Vec<_> = remaining_fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let field_path = format!("{}.{}", type_name, field_ident);

            // Determine if the field is optional and extract inner type
            let (is_opt, type_to_use) = if is_optional(field) {
                // For optional fields, extract inner type from Option<T>
                // This will panic if extraction fails, which is desired
                let inner_type = match field_type {
                    syn::Type::Path(type_path) => {
                        let segment = type_path
                            .path
                            .segments
                            .last()
                            .expect("Expected path to have at least one segment");
                        assert!(
                            segment.ident == "Option",
                            "Field is marked optional but type is not Option<T>"
                        );

                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            match args.args.first() {
                                Some(syn::GenericArgument::Type(inner)) => inner.clone(),
                                _ => panic!("Expected Option<T> to have a type parameter"),
                            }
                        } else {
                            panic!("Expected Option<T> to have angle-bracketed arguments");
                        }
                    }
                    _ => panic!("Expected optional field to have path type"),
                };

                (true, inner_type)
            } else {
                // Non-optional field
                (false, field_type.clone())
            };

            // Check if we have a type override for this field
            let (param_type, as_type) =
                if let Some(over) = otlp_model::FIELD_TYPE_OVERRIDES.get(field_path.as_str()) {
                    (
                        syn::parse_str::<syn::Type>(over.datatype).unwrap(),
                        Some(syn::parse_str::<syn::Type>(over.fieldtype).unwrap()),
                    )
                } else {
                    (type_to_use, None)
                };

            // Generate the builder method with appropriate value assignment
            let value_assignment = if is_opt {
                if let Some(ref as_type) = as_type {
                    quote! { self.inner.#field_ident = Some(value.into() as #as_type); }
                } else {
                    quote! { self.inner.#field_ident = Some(value.into()); }
                }
            } else {
                if let Some(ref as_type) = as_type {
                    quote! { self.inner.#field_ident = value.into() as #as_type; }
                } else {
                    quote! { self.inner.#field_ident = value.into(); }
                }
            };

            // Generate a single builder method with conditional logic
            quote! {
                pub fn #field_ident<T>(mut self, value: T) -> Self
                where T: Into<#param_type>
                {
                    #value_assignment
                    self
                }
            }
        })
        .collect();

    let expanded = if remaining_fields.is_empty() {
        // If there are no remaining fields, directly return the constructed object
        quote! {
            impl #outer_name {
                /// Creates a new instance of #name
                pub fn new<#(#param_bounds),*>(#(#param_decls),*) -> #outer_name {
                    let mut inner = #outer_name::default();
                    #(#param_assignments)*
                    inner
                }
            }
        }
    } else {
        // If there are remaining fields, generate builder methods for them
        quote! {
            pub struct #builder_name {
                inner: #outer_name,
            }

            impl #outer_name {
                /// Creates a new builder for #name
                pub fn new<#(#param_bounds),*>(#(#param_decls),*) -> #builder_name {
                    let mut inner = #outer_name::default();
                    #(#param_assignments)*
                    #builder_name {
                        inner
                    }
                }
            }

            impl #builder_name {
                #(#builder_methods)*

                pub fn build(self) -> #outer_name {
                    self.inner
                }
            }

            impl std::convert::From<#builder_name> for #outer_name {
                fn from(value: #builder_name) -> Self {
                    value.build()
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Value)]
pub fn derive_otlp_value(_: TokenStream) -> TokenStream {
    let expanded = quote! {
        impl AnyValue {
            pub fn new_string<T: AsRef<str>>(s: T) -> Self {
                Self{
                    value: Some(any_value::Value::StringValue(s.as_ref().to_string())),
                }
            }

            pub fn new_bool(b: bool) -> Self {
                Self{
                    value: Some(any_value::Value::BoolValue(b)),
                }
            }

            pub fn new_int(i: i64) -> Self {
                Self{
                    value: Some(any_value::Value::IntValue(i)),
                }
            }

            pub fn new_double(d: f64) -> Self {
                Self{
                    value: Some(any_value::Value::DoubleValue(d)),
                }
            }

            pub fn new_array<T: AsRef<[AnyValue]>>(arr: T) -> Self {
                Self{
                    value: Some(any_value::Value::ArrayValue(ArrayValue{
            values: arr.as_ref().to_vec(),
            })),
                }
            }

            pub fn new_kvlist<T: AsRef<[KeyValue]>>(kvlist: T) -> Self {
                Self{
                    value: Some(any_value::Value::KvlistValue(KeyValueList{
            values: kvlist.as_ref().to_vec(),
            })),
                }
            }

            pub fn new_bytes<T: AsRef<[u8]>>(b: T) -> Self {
                Self{
                    value: Some(any_value::Value::BytesValue(b.as_ref().to_vec())),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
