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
    
    // Create a special attribute that will store the qualified name
    // Use parentheses format instead of equals sign to match how parse_args works
    let qualified_attr = syn::parse_quote! {
        #[otlp_qualified_name(#args_str)]
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

    // Get the fully qualified type name from attribute, or fall back to the struct name
    let struct_name = outer_name.to_string();
    let type_name = input.attrs.iter()
        .find_map(|attr| {
            if attr.path().is_ident("otlp_qualified_name") {
                attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
            } else {
                None
            }
        })
        .unwrap_or_else(|| struct_name.clone());

    let detail = otlp_model::DETAILS
        .iter()
        .find(|detail| detail.name.ends_with(&format!(".{}", type_name)))
        .cloned()
        .unwrap_or(otlp_model::Detail::default());

    // Extract params or get an empty Vec if none exists
    let params = detail.params.map_or_else(Vec::new, |p| p.to_vec());

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

    // Function to check if a field is marked as optional
    let is_optional = |field: &syn::Field| {
        field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains("optional")
        })
    };

    // Find fields that are not explicitly initialized via params
    let remaining_fields = struct_fields
        .iter()
        .filter(|field| {
            field.ident.as_ref().map_or(false, |id| {
                !params.iter().any(|param| param.name == id.to_string())
            })
        })
        .collect::<Vec<_>>();

    // Generate parameters for new() function
    let param_decls: Vec<_> = params
        .iter()
        .map(|field| {
            let param_name = syn::Ident::new(field.name, outer_name.span());
            let param_type = syn::parse_str::<syn::Type>(field.ty).unwrap();
            quote! { #param_name: #param_type }
        })
        .collect();

    let param_bounds: Vec<_> = params
        .iter()
        .filter(|field| !field.bound.is_empty())
        .map(|field| {
            let param_type = syn::parse_str::<syn::Type>(field.ty).unwrap();
            let param_bound = syn::parse_str::<syn::Type>(field.bound).unwrap();
            quote! { #param_type: #param_bound }
        })
        .collect();

    // Generate assignments in the new() function
    let param_assignments: Vec<_> = params
        .iter()
        .map(|field| {
            let param_name = syn::Ident::new(field.name, outer_name.span());

            // Find corresponding field in struct definition to check if it's optional
            let is_optional = match &input.data {
                syn::Data::Struct(data) => {
                    if let syn::Fields::Named(fields) = &data.fields {
                        fields
                            .named
                            .iter()
                            .find(|f| f.ident.as_ref().map_or(false, |id| id == field.name))
                            .map_or(false, |f| {
                                f.attrs.iter().any(|attr| {
                                    attr.path().is_ident("prost")
                                        && attr.to_token_stream().to_string().contains("optional")
                                })
                            })
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if field.get.is_empty() {
                if is_optional {
                    quote! { inner.#param_name = Some(#param_name); }
                } else {
                    quote! { inner.#param_name = #param_name; }
                }
            } else {
                // Parse the string directly as a token stream instead of as an expression
                let get_tokens = field.get.parse::<proc_macro2::TokenStream>().unwrap();
                if is_optional {
                    quote! { inner.#param_name = Some(#param_name.#get_tokens); }
                } else {
                    quote! { inner.#param_name = #param_name.#get_tokens; }
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

            if is_optional(field) {
                // For optional fields, extract inner type from Option<T> if possible
                if let syn::Type::Path(type_path) = field_type {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "Option" {
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                if let Some(syn::GenericArgument::Type(inner_type)) =
                                    args.args.first()
                                {
                                    // Check if we have a type override for this field
                                    let field_path = format!("{}.{}", type_name, field_ident);

                                    // Determine which type to use (override or inner)
                                    let type_to_use = if let Some(override_type) =
                                        otlp_model::FIELD_TYPE_OVERRIDES.get(field_path.as_str())
                                    {
                                        syn::parse_str::<syn::Type>(override_type).unwrap()
                                    } else {
                                        inner_type.clone()
                                    };

                                    return quote! {
                                        pub fn #field_ident<T>(mut self, value: T) -> Self
                                        where T: Into<#type_to_use>
                                        {
                                            self.inner.#field_ident = Some(value.into());
                                            self
                                        }
                                    };
                                }
                            }
                        }
                    }
                }

                // Fallback
                quote! {
                    pub fn #field_ident<T>(mut self, value: T) -> Self
                    where T: Into<_>
                    {
                        self.inner.#field_ident = Some(value.into());
                        self
                    }
                }
            } else {
                quote! {
                    pub fn #field_ident<T>(mut self, value: T) -> Self
                    where T: Into<#field_type>
                    {
                        self.inner.#field_ident = value.into();
                        self
                    }
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
