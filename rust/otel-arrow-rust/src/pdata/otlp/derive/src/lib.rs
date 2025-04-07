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

#[proc_macro_derive(Oneof)]
pub fn derive_otlp_oneof(_: TokenStream) -> TokenStream {
    TokenStream::new()
}
