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
            fn placeholder() {
                eprintln!(stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}
