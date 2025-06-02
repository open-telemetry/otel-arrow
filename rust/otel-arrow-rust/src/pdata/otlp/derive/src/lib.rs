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

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, parse_macro_input};

#[derive(Clone)]
struct FieldInfo {
    ident: syn::Ident,
    is_param: bool,
    is_optional: bool,
    is_repeated: bool,
    is_oneof: bool,
    field_type: syn::Type,
    as_type: Option<syn::Type>,
}

type TokenVec = Vec<proc_macro2::TokenStream>;
type OneofMapping<'a> = Option<(&'a &'a str, &'a Vec<otlp_model::OneofCase>)>;

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

    // Get required parameters for this type.
    let param_names = otlp_model::REQUIRED_PARAMS.get(type_name.as_str()).unwrap();

    // Check if this struct has a oneof field
    let oneof_mapping = otlp_model::ONEOF_MAPPINGS
        .iter()
        .find(|(field, _)| field.starts_with(&type_name));

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
    if struct_fields.is_empty() {
        panic!("message with empty fields")
    }

    // Helper function to check if a type is Option<T>
    let is_option_type = |ty: &syn::Type| -> bool {
        if let syn::Type::Path(type_path) = ty {
            type_path
                .path
                .segments
                .last()
                .map(|segment| segment.ident == "Option")
                .unwrap_or(false)
        } else {
            false
        }
    };

    // Helper function to check if a type is Vec<T>
    let is_vec_type = |ty: &syn::Type| -> bool {
        if let syn::Type::Path(type_path) = ty {
            type_path
                .path
                .segments
                .last()
                .map(|segment| segment.ident == "Vec")
                .unwrap_or(false)
        } else {
            false
        }
    };

    // Function to check if a field is marked as optional
    let is_optional_repeated = |field: &syn::Field| {
        // Check prost attributes first
        let attr_optional = field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains("optional")
        });
        let attr_repeated = field.attrs.iter().any(|attr| {
            attr.path().is_ident("prost") && attr.to_token_stream().to_string().contains("repeated")
        });

        // Also check the actual type structure
        let type_optional = is_option_type(&field.ty);
        let type_repeated = is_vec_type(&field.ty);

        (
            attr_optional || type_optional,
            attr_repeated || type_repeated,
        )
    };

    // Extract option inner type as a standalone function for better reuse
    let extract_option_inner_type = |ty: &syn::Type| -> Option<(syn::Type, bool)> {
        if let syn::Type::Path(type_path) = ty {
            type_path
                .path
                .segments
                .last()
                .and_then(|segment| (segment.ident == "Option").then_some(segment))
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

    let fields_original: Vec<FieldInfo> = struct_fields
        .iter()
        .filter_map(|field| {
            // Early return if no identifier
            field.ident.as_ref().map(|ident| {
                let ident_str = ident.to_string();
                let field_path = format!("{}.{}", type_name, ident_str);
                let is_param = param_names.contains(&ident_str.as_str());
                let (is_optional, is_repeated) = is_optional_repeated(field);
                let is_oneof = oneof_mapping.map(|x| *x.0 == field_path).unwrap_or(false);

                // Process type information
                let (inner_type, is_optional_extraction_ok) = if is_optional {
                    extract_option_inner_type(&field.ty)
                        .unwrap_or_else(|| (field.ty.clone(), false))
                } else {
                    (field.ty.clone(), true)
                };

                // Validate optional field extraction
                if is_optional && !is_optional_extraction_ok {
                    panic!(
                        "Field '{}' is marked optional but does not have a valid Option<T> type",
                        ident
                    );
                }

                // Get type overrides if present
                let (field_type, as_type) = otlp_model::FIELD_TYPE_OVERRIDES
                    .get(field_path.as_str())
                    .map(|over| {
                        (
                            syn::parse_str::<syn::Type>(over.datatype).unwrap(),
                            Some(syn::parse_str::<syn::Type>(over.fieldtype).unwrap()),
                        )
                    })
                    .unwrap_or_else(|| (inner_type, None));

                FieldInfo {
                    ident: ident.clone(),
                    is_param,
                    is_optional,
                    is_repeated,
                    is_oneof,
                    field_type,
                    as_type,
                }
            })
        })
        .collect();

    // Partition fields into ordered parameters and remaining builder fields.
    let param_fields: Vec<_> = param_names
        .iter()
        .map(|param_name| {
            fields_original
                .iter()
                .find(|info| {
                    let ident = info.ident.to_string();
                    info.is_param && ident == *param_name
                })
                .unwrap()
        })
        .cloned()
        .collect();
    let builder_fields: Vec<_> = fields_original
        .iter()
        .filter(|info| !info.is_param)
        .cloned()
        .collect();
    let all_fields: Vec<_> = param_fields
        .iter()
        .chain(builder_fields.iter())
        .cloned()
        .collect();

    let mut tokens = TokenStream::new();

    tokens.extend(derive_otlp_builders(
        outer_name,
        param_names,
        &param_fields,
        &builder_fields,
        &all_fields,
        oneof_mapping,
    ));

    tokens.extend(derive_otlp_visitors(
        outer_name,
        param_names,
        &param_fields,
        &builder_fields,
        &all_fields,
        oneof_mapping,
    ));

    tokens
}

/// Emits the builders, new(), and finish() methods.
fn derive_otlp_builders(
    outer_name: &syn::Ident,
    param_names: &Vec<&str>,
    param_fields: &[FieldInfo],
    builder_fields: &[FieldInfo],
    all_fields: &[FieldInfo],
    oneof_mapping: OneofMapping,
) -> TokenStream {
    let builder_name = syn::Ident::new(&format!("{}Builder", outer_name), outer_name.span());

    // Generate generic type parameters names like ["T1", "T2", ...]
    let type_params: Vec<syn::Ident> = (0..all_fields.len())
        .map(|idx| {
            let type_name = format!("T{}", idx + 1);
            syn::Ident::new(&type_name, proc_macro2::Span::call_site())
        })
        .collect();

    // Generate a list of arguments to pass from build() to new().
    let param_args: TokenVec = param_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            quote! { #field_name }
        })
        .collect();

    // Generate parameter declarations and where bounds
    let (param_decls, param_bounds): (TokenVec, TokenVec) = param_fields
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let param_name = &info.ident;
            let type_param = &type_params[idx];
            let target_type = &info.field_type;

            let decl = quote! { #param_name: #type_param };
            let bound = quote! { #type_param: Into<#target_type> };

            (decl, bound)
        })
        .unzip();

    // Generate field assignments and initializers
    let (field_assignments, field_initializers): (TokenVec, TokenVec) = all_fields
        .iter()
        .map(|info| {
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
        })
        .unzip();

    // Default initializers for fields
    let default_initializers: TokenVec = all_fields
        .iter()
        .map(|info| {
            let field_name = &info.ident;
            let type_str = info.field_type.to_token_stream().to_string();
            if info.is_optional {
                quote! { #field_name: None, }
            } else {
                match type_str.as_str() {
                    "u8" | "u16" | "u32" | "u64" => quote! {#field_name: 0,},
                    "i8" | "i16" | "i32" | "i64" => quote! {#field_name: 0,},
                    "f32" | "f64" => quote! {#field_name: 0.0,},
                    "bool" => quote! {#field_name: false,},
                    _ => quote! {#field_name: ::core::default::Default::default(),},
                }
            }
        })
        .collect();

    // All field initializers includes parameters and defaults
    let all_field_initializers: Vec<_> = (0..all_fields.len())
        .map(|idx| {
            if idx < param_names.len() {
                field_initializers[idx].clone()
            } else {
                default_initializers[idx].clone()
            }
        })
        .collect();

    // Generate builder methods
    let builder_methods: TokenVec = all_fields
        .iter()
        .enumerate()
        .filter(|(_, info)| !info.is_oneof)
        .map(|(idx, info)| {
            let field_name = &info.ident;
            let field_type = &info.field_type;
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
    let derive_builder = !builder_fields.is_empty();

    // Function to build constructors used in oneof and normal cases.
    let create_constructor =
        |suffix: String,
         cur_param_bounds: &[proc_macro2::TokenStream],
         cur_param_decls: &[proc_macro2::TokenStream],
         cur_param_args: &[proc_macro2::TokenStream],
         cur_field_initializers: &[proc_macro2::TokenStream]| {
            let build_name =
                syn::Ident::new(&format!("build{}", suffix), proc_macro2::Span::call_site());
            let new_name =
                syn::Ident::new(&format!("new{}", suffix), proc_macro2::Span::call_site());

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
    let all_constructors: TokenVec = match oneof_mapping {
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
            // Extract the field name from the mapped path
            let oneof_name = oneof_mapping.0.split('.').last().unwrap();
            let oneof_ident = syn::Ident::new(oneof_name, proc_macro2::Span::call_site());

            let idx = param_names
                .iter()
                .position(|&name| name == oneof_name)
                .unwrap();

            // Generate a constructor for each oneof case
            oneof_mapping.1.iter().map(|case| {
                let case_type = syn::parse_str::<syn::Type>(case.type_param).unwrap();
                let variant_path = syn::parse_str::<syn::Expr>(case.value_variant).unwrap();
                let suffix = format!("_{}", case.name);

                // Duplicate the param bounds, assignments; param decls unchanged.
                let mut cur_param_bounds = param_bounds.clone();
                let mut cur_field_initializers = all_field_initializers.clone();
                let type_param = type_params[idx].clone();

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

                // Replace the parameter w/ oneof-specific expansion
                cur_param_bounds[idx] = value_bound;
                cur_field_initializers[idx] = value_initializer;

                create_constructor(suffix, &cur_param_bounds, &param_decls, &param_args, &cur_field_initializers)
            }).collect()
        }
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

/// Emits the visitor, visitable and adapters methods.
fn derive_otlp_visitors(
    outer_name: &syn::Ident,
    _param_names: &Vec<&str>,
    _param_fields: &[FieldInfo],
    _builder_fields: &[FieldInfo],
    all_fields: &[FieldInfo],
    oneof_mapping: OneofMapping,
) -> TokenStream {
    let visitor_name = syn::Ident::new(&format!("{}Visitor", outer_name), outer_name.span());
    let visitable_name = syn::Ident::new(&format!("{}Visitable", outer_name), outer_name.span());
    let method_name = syn::Ident::new(
        &format!("Visit{}", outer_name).to_case(Case::Snake),
        outer_name.span(),
    );

    let mut visitable_args: TokenVec = Vec::new();

    for info in all_fields {
        if info.is_oneof {
            // For oneof fields, generate separate parameters for each variant
            if let Some((oneof_name, oneof_cases)) = oneof_mapping {
                if oneof_name.ends_with(&format!(".{}", info.ident)) {
                    // This is the oneof field we're looking for
                    for case in oneof_cases {
                        let variant_param_name = syn::Ident::new(
                            &format!("{}_{}", info.ident, case.name),
                            info.ident.span(),
                        );

                        // Parse the type_param to get the visitor trait path
                        if let Ok(case_type) = syn::parse_str::<syn::Type>(case.type_param) {
                            let visitor_type = generate_visitor_type_for_oneof_variant(&case_type);
                            visitable_args.push(quote! { #variant_param_name: #visitor_type });
                        }
                    }
                    continue;
                }
            }
        }

        // For non-oneof fields, generate normal visitor parameter
        let param_name = &info.ident;
        let type_tokens = info.base_type();
        visitable_args.push(quote! { #param_name: #type_tokens });
    }

    let expanded = quote! {
    pub trait #visitor_name {
    fn #method_name(&mut self, v: impl #visitable_name);
    }

    pub trait #visitable_name {
    fn #method_name(&self, #(#visitable_args),*);
    }
    };

    TokenStream::from(expanded)
}

/// Generate visitor type for a oneof variant
fn generate_visitor_type_for_oneof_variant(case_type: &syn::Type) -> proc_macro2::TokenStream {
    match case_type {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "Vec" => {
                        // Handle Vec<T> types
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                if let syn::Type::Path(inner_path) = inner_type {
                                    if let Some(inner_segment) = inner_path.path.segments.last() {
                                        match inner_segment.ident.to_string().as_str() {
                                            "u8" => quote! { impl crate::pdata::BytesVisitor },
                                            _ => {
                                                // For Vec<MessageType>, generate visitor for the message type
                                                let mut visitor_path = inner_path.path.clone();
                                                if let Some(last_segment) =
                                                    visitor_path.segments.last_mut()
                                                {
                                                    let type_name = last_segment.ident.to_string();
                                                    let visitor_name =
                                                        format!("{}Visitor", type_name);
                                                    last_segment.ident = syn::Ident::new(
                                                        &visitor_name,
                                                        last_segment.ident.span(),
                                                    );
                                                    last_segment.arguments =
                                                        syn::PathArguments::None;
                                                }
                                                quote! { impl #visitor_path }
                                            }
                                        }
                                    } else {
                                        quote! { impl UnknownVisitor }
                                    }
                                } else {
                                    quote! { impl UnknownVisitor }
                                }
                            } else {
                                quote! { impl UnknownVisitor }
                            }
                        } else {
                            quote! { impl UnknownVisitor }
                        }
                    }
                    // For primitive types, use the appropriate visitor trait
                    "String" => quote! { impl crate::pdata::StringVisitor },
                    "bool" => quote! { impl crate::pdata::BooleanVisitor },
                    "i32" | "i64" | "u32" | "u64" => quote! { impl crate::pdata::IntegerVisitor },
                    "f32" | "f64" => quote! { impl crate::pdata::FloatVisitor },
                    "u8" => quote! { impl crate::pdata::IntegerVisitor },
                    _ => {
                        // For message types, generate visitor trait path
                        let mut visitor_path = type_path.path.clone();
                        if let Some(last_segment) = visitor_path.segments.last_mut() {
                            let type_name = last_segment.ident.to_string();
                            let visitor_name = format!("{}Visitor", type_name);
                            last_segment.ident =
                                syn::Ident::new(&visitor_name, last_segment.ident.span());
                            last_segment.arguments = syn::PathArguments::None;
                        }
                        quote! { impl #visitor_path }
                    }
                }
            } else {
                quote! { impl UnknownVisitor }
            }
        }
        _ => quote! { impl UnknownVisitor },
    }
}

impl FieldInfo {
    fn base_type(&self) -> proc_macro2::TokenStream {
        // If this field has an as_type (enum field), use the underlying primitive type
        if let Some(as_type) = &self.as_type {
            return match as_type {
                syn::Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        match segment.ident.to_string().as_str() {
                            "String" => quote! { &str },
                            "bool" => quote! { bool },
                            "i32" => quote! { i32 },
                            "i64" => quote! { i64 },
                            "u32" => quote! { u32 },
                            "u64" => quote! { u64 },
                            "f32" => quote! { f32 },
                            "f64" => quote! { f64 },
                            "u8" => quote! { u8 },
                            _ => quote! { #as_type },
                        }
                    } else {
                        quote! { #as_type }
                    }
                }
                _ => quote! { #as_type },
            };
        }

        // Special handling for repeated Vec<u8> fields (bytes)
        if self.is_repeated {
            if let syn::Type::Path(type_path) = &self.field_type {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Vec" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                if let syn::Type::Path(inner_path) = inner_ty {
                                    if let Some(inner_segment) = inner_path.path.segments.last() {
                                        if inner_segment.ident == "u8" {
                                            return quote! { &[u8] };
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let base_type = self.extract_base_type();

        // Convert the base type to appropriate parameter type for visitor pattern
        match &base_type {
            syn::Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    match segment.ident.to_string().as_str() {
                        "String" => quote! { &str },
                        "bool" => quote! { bool },
                        "i32" => quote! { i32 },
                        "i64" => quote! { i64 },
                        "u32" => quote! { u32 },
                        "u64" => quote! { u64 },
                        "f32" => quote! { f32 },
                        "f64" => quote! { f64 },
                        "u8" => quote! { u8 }, // Handle raw u8 case
                        _ => {
                            // For message types, generate fully qualified visitor trait path
                            self.generate_visitor_trait_path(type_path)
                        }
                    }
                } else {
                    // Fallback for empty path
                    quote! { UnknownType }
                }
            }
            _ => {
                // For non-path types, generate a generic name
                quote! { GenericType }
            }
        }
    }

    /// Generate a fully qualified path to the visitor trait for a message type
    fn generate_visitor_trait_path(&self, type_path: &syn::TypePath) -> proc_macro2::TokenStream {
        // Clone the path and modify the last segment to add "Visitor" suffix
        let mut visitor_path = type_path.path.clone();

        if let Some(last_segment) = visitor_path.segments.last_mut() {
            let type_name = last_segment.ident.to_string();
            let visitor_name = format!("{}Visitor", type_name);
            last_segment.ident = syn::Ident::new(&visitor_name, last_segment.ident.span());
            // Clear any generic arguments from the visitor name
            last_segment.arguments = syn::PathArguments::None;
        }

        quote! { impl #visitor_path }
    }

    /// Extract the base type by stripping Option<T> and Vec<T> wrappers
    fn extract_base_type(&self) -> syn::Type {
        let mut current_type = self.field_type.clone();

        // Strip Vec<T> if repeated
        if self.is_repeated {
            current_type = self
                .strip_vec_wrapper(&current_type)
                .unwrap_or(current_type);
        }

        // Strip Option<T> if optional
        if self.is_optional {
            current_type = self
                .strip_option_wrapper(&current_type)
                .unwrap_or(current_type);
        }

        current_type
    }

    /// Strip Vec<T> wrapper and return T
    fn strip_vec_wrapper(&self, ty: &syn::Type) -> Option<syn::Type> {
        if let syn::Type::Path(type_path) = ty {
            let last_segment = type_path.path.segments.last()?;

            // Check if this is a Vec (could be std::vec::Vec, prost::alloc::vec::Vec, etc.)
            if last_segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type.clone());
                    }
                }
            }
        }
        None
    }

    /// Strip Option<T> wrapper and return T  
    fn strip_option_wrapper(&self, ty: &syn::Type) -> Option<syn::Type> {
        if let syn::Type::Path(type_path) = ty {
            let last_segment = type_path.path.segments.last()?;

            // Check if this is an Option (could be std::option::Option, core::option::Option, etc.)
            if last_segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type.clone());
                    }
                }
            }
        }
        None
    }
}
