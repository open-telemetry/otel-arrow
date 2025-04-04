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

/// Derives the OTAP Message trait implementation for protocol buffer
/// message types. This enables additional OTAP-specific functionality
/// beyond what prost::Message provides.
#[proc_macro_derive(Message)]
pub fn derive_otap_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Generate the trait implementation
    let expanded = quote! {
        impl crate::proto::pdata::otap::Message for #name {
            // Implementation details will be added as needed
            fn message_type() -> &'static str {
                stringify!(#name)
            }
            
            // This would be expanded with more methods as needed
        }
    };

    TokenStream::from(expanded)
}

/// Example of another derive macro that could be added,
/// such as for creating builders for message types.
#[proc_macro_derive(MessageBuilder)]
pub fn derive_otap_message_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = quote::format_ident!("{}Builder", name);
    
    let expanded = quote! {
        pub struct #builder_name {
            // Fields would be generated based on the input struct
            inner: #name,
        }
        
        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    inner: #name::default(),
                }
            }
            
            pub fn build(self) -> #name {
                self.inner
            }
            
            // Additional builder methods would be generated here
        }
        
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    };
    
    TokenStream::from(expanded)
}