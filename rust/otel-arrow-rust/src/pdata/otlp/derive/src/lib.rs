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
use otlp_model::DETAILS;

/// Derives the OTLP Message trait implementation for protocol buffer
/// message types. This enables additional OTLP-specific functionality
/// beyond what prost::Message provides.
#[proc_macro_derive(Message)]
pub fn derive_otlp_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let outer_name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", outer_name), outer_name.span());
    
    // Find if this type has any builder params in DETAILS
    let type_name = outer_name.to_string();
    let details = DETAILS.iter()
        .find(|detail| detail.name.ends_with(&format!(".{}", type_name)));
    let params = details.and_then(|detail| detail.params.as_ref());

    let expanded = if let Some(params) = params {
        // Generate parameters for new() function
        let param_decls: Vec<_> = params.iter()
            .map(|field| {
                let param_name = syn::Ident::new(field.name, outer_name.span());
                let param_type = syn::parse_str::<syn::Type>(field.ty).unwrap();
                quote! { #param_name: #param_type }
            })
            .collect();

        let param_bounds: Vec<_> = params.iter()
            .filter(|field| !field.bound.is_empty())
            .map(|field| {
                let param_type = syn::parse_str::<syn::Type>(field.ty).unwrap();
                let param_bound = syn::parse_str::<syn::Type>(field.bound).unwrap();
                quote! { #param_type: #param_bound }
            })
            .collect();
        
        // Generate assignments in the new() function
        let param_assignments: Vec<_> = params.iter().map(|field| {
            let param_name = syn::Ident::new(field.name, outer_name.span());
            
            // Find corresponding field in struct definition to check if it's optional
            let is_optional = match &input.data {
                syn::Data::Struct(data) => {
                    if let syn::Fields::Named(fields) = &data.fields {
                        fields.named.iter()
                            .find(|f| f.ident.as_ref().map_or(false, |id| id == field.name))
                            .map_or(false, |f| {
                                f.attrs.iter().any(|attr| {
                                    attr.path().is_ident("prost") && 
                                    attr.to_token_stream().to_string().contains("optional")
                                })
                            })
                    } else {
                        false
                    }
                },
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

	let direct = details.unwrap().direct;

	if direct {
            quote! {
		impl #outer_name {
                    /// Creates a new builder for #name
                    pub fn new<#(#param_bounds),*>(#(#param_decls),*) -> #outer_name {
			let mut inner = #outer_name::default();
			#(#param_assignments)*
                    inner
                    }
		}
	    }
	} else {
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

		impl std::convert::Into<#outer_name> for #builder_name {
                    fn into(self) -> #outer_name {
			self.inner
                    }
		}
            }
	}
    } else {
        quote! {
            pub struct #builder_name {
                inner: #outer_name,
            }

            impl #outer_name {
                /// Creates a new builder for #name
                pub fn new() -> #builder_name {
                    #builder_name {
                        inner: #outer_name::default(),
                    }
                }
            }

            impl std::convert::Into<#outer_name> for #builder_name {
                fn into(self) -> #outer_name {
                    self.inner
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

            pub fn new_array(arr: ArrayValue) -> Self {
                Self{
                    value: Some(any_value::Value::ArrayValue(arr.into())),
                }
            }

            pub fn new_kvlist(kvlist: &[KeyValue]) -> Self {
                Self{
                    value: Some(any_value::Value::KvlistValue(KeyValueList{
			values: kvlist.to_vec(),
		    })),
                }
            }

            pub fn new_bytes(b: &[u8]) -> Self {
                Self{
                    value: Some(any_value::Value::BytesValue(b.to_vec())),
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}
