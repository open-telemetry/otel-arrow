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
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives the OTLP Message trait implementation for protocol buffer
/// message types. This enables additional OTLP-specific functionality
/// beyond what prost::Message provides.
#[proc_macro_derive(Message)]
pub fn derive_otlp_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());

    let expanded = quote! {
        pub struct #builder_name {
            inner: #name,
        }

        impl #builder_name {
            /// Creates a new builder for #name
            pub fn new() -> Self {
                Self {
                    inner: #name::default(),
                }
            }
        }

        impl std::convert::Into<#name> for #builder_name {
            fn into(self) -> #name {
                self.inner
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
        }
    };
    
    TokenStream::from(expanded)
}

#[proc_macro_derive(Oneof)]
pub fn derive_otlp_oneof(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Extract enum variants - ensure this is applied to an enum
    let variants = match &input.data {
        syn::Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new(
                name.span(),
                "Oneof can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };
    
    // Collect results for each variant
    let mut methods = Vec::new();
    
    for variant in variants {
        let variant_name = &variant.ident;
        let name_for_method = variant_name.to_string();
        
        // Convert to snake_case (e.g., StringValue -> string_value or String -> string)
        let snake_name = convert_case::Casing::to_case(
            &name_for_method,
            convert_case::Case::Snake
        );
        let method_name = syn::Ident::new(&format!("new_{}", snake_name), variant_name.span());
        
        // Extract the field type from the variant
        if let syn::Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() == 1 {
                let field = fields.unnamed.first().unwrap();
                let ty = &field.ty;
                
                // Generate the constructor method
                methods.push(quote! {
                    pub fn #method_name(value: #ty) -> Self {
                        Self::#variant_name(value)
                    }
                });
            } else {
                return syn::Error::new(
                    variant_name.span(),
                    "Oneof variants must have exactly one unnamed field"
                )
                .to_compile_error()
                .into();
            }
        } else {
            return syn::Error::new(
                variant_name.span(),
                "Oneof variants must have exactly one unnamed field"
            )
            .to_compile_error()
            .into();
        }
    }
    
    // Generate the impl block with all methods
    let expanded: proc_macro2::TokenStream = quote! {
        impl #name {
            #(#methods)*
        }
    };
    
    TokenStream::from(expanded)
}
