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
    
    // Parse input and add the qualified attribute in a more functional way
    let input_ast = syn::parse_macro_input!(input as syn::DeriveInput)
        .into_token_stream()
        .to_string();
    
    // Create a special doc comment that will store the qualified name
    let qualified_attr = syn::parse_quote! {
        #[doc(hidden, otlp_qualified_name = #args_str)]
    };
    
    // Parse again and add the attribute
    let mut final_ast = syn::parse_str::<DeriveInput>(&input_ast).unwrap();
    final_ast.attrs.push(qualified_attr);

    // Return the modified struct definition
    quote::quote!(#final_ast).into()
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

    // Process all fields once with common logic
    struct FieldInfo {
        ident: syn::Ident,
        is_param: bool,
        is_optional: bool,
        param_type: syn::Type,
        as_type: Option<syn::Type>,
    }

    // Extract option inner type as a standalone function for better reuse
    let extract_option_inner_type = |ty: &syn::Type| -> Option<(syn::Type, bool)> {
        if let syn::Type::Path(type_path) = ty {
            type_path.path.segments.last()
                .and_then(|segment| {
                    (segment.ident == "Option").then_some(segment)
                })
                .and_then(|segment| {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        args.args.first()
                    } else {
                        None
                    }
                })
                .and_then(|arg| {
                    if let syn::GenericArgument::Type(inner_type) = arg {
                        Some((inner_type.clone(), true))
                    } else {
                        None
                    }
                })
        } else {
            None
        }
    };
    
    let field_infos: Vec<FieldInfo> = struct_fields
        .iter()
        .filter_map(|field| {
            // Early return if no identifier
            field.ident.as_ref().map(|ident| {
                let ident_str = ident.to_string();
                let is_param = param_names.contains(&ident_str.as_str());
                let is_optional = is_optional(field);
                let field_path = format!("{}.{}", type_name, ident_str);
                
                // Process type information
                let (inner_type, is_optional_extraction_ok) = if is_optional {
                    extract_option_inner_type(&field.ty)
                        .unwrap_or_else(|| (field.ty.clone(), false))
                } else {
                    (field.ty.clone(), true)
                };

                // Validate optional field extraction
                if is_optional && !is_optional_extraction_ok {
                    panic!("Field '{}' is marked optional but does not have a valid Option<T> type", ident);
                }

                // Get type overrides if present
                let (param_type, as_type) = otlp_model::FIELD_TYPE_OVERRIDES
                    .get(field_path.as_str())
                    .map(|over| (
                        syn::parse_str::<syn::Type>(over.datatype).unwrap(),
                        Some(syn::parse_str::<syn::Type>(over.fieldtype).unwrap())
                    ))
                    .unwrap_or_else(|| (inner_type, None));

                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    param_type,
                    as_type,
                }
            })
        })
        .collect();

    // Partition fields into parameters and builder fields in one pass
    let (param_fields, builder_fields): (Vec<&FieldInfo>, Vec<&FieldInfo>) = field_infos.iter()
        .partition(|info| info.is_param);

    // Generate generic type parameters for parameters using functional patterns
    let type_params: Vec<syn::Ident> = param_fields.iter().enumerate()
        .map(|(idx, _)| {
            let type_name = format!("T{}", idx + 1);
            syn::Ident::new(&type_name, proc_macro2::Span::call_site())
        })
        .collect();

    // Generate parameter declarations and where bounds together with zipped iterators
    let (param_decls, param_bounds): (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) = 
        param_fields.iter().enumerate().map(|(idx, info)| {
            let param_name = &info.ident;
            let type_param = &type_params[idx];
            let target_type = &info.param_type;
            
            let decl = quote! { #param_name: #type_param };
            let bound = quote! { #type_param: Into<#target_type> };
            
            (decl, bound)
        })
        .unzip();

    // Generate assignments for parameters in new() function with cleaner functional approach
    let param_assignments: Vec<proc_macro2::TokenStream> = param_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            
            match (info.is_optional, &info.as_type) {
                (true, Some(as_type)) => quote! { inner.#field_name = Some(#field_name.into() as #as_type); },
                (true, None) => quote! { inner.#field_name = Some(#field_name.into()); },
                (false, Some(as_type)) => quote! { inner.#field_name = #field_name.into() as #as_type; },
                (false, None) => quote! { inner.#field_name = #field_name.into(); },
            }
        })
        .collect();

    // Generate builder methods
    let builder_methods: Vec<proc_macro2::TokenStream> = builder_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            let param_type = &info.param_type;
            
            let value_assignment = match (info.is_optional, &info.as_type) {
                (true, Some(ref as_type)) => quote! { self.inner.#field_name = Some(value.into() as #as_type); },
                (true, None) => quote! { self.inner.#field_name = Some(value.into()); },
                (false, Some(ref as_type)) => quote! { self.inner.#field_name = value.into() as #as_type; },
                (false, None) => quote! { self.inner.#field_name = value.into(); },
            };

            quote! {
                pub fn #field_name<T>(mut self, value: T) -> Self
                where T: Into<#param_type>
                {
                    #value_assignment
                    self
                }
            }
        })
        .collect();

    let expanded = if builder_fields.is_empty() {
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
