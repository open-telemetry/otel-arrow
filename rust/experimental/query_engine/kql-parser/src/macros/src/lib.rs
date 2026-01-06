// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! TODO explain what this is doing

use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};

const BASE_PEST_SOURCE: &'static str = include_str!("../../base.pest");

extern crate proc_macro;

#[proc_macro_derive(BaseRuleCompatible)]
pub fn derive_base_rule_compatible(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pairs =
        pest_meta::parser::parse(pest_meta::parser::Rule::grammar_rules, BASE_PEST_SOURCE).unwrap();
    let ast = pest_meta::parser::consume_rules(pairs).unwrap();
    let rules = pest_meta::optimizer::optimize(ast);

    let base_rule_crate_base =
        match crate_name("data_engine_kql_parser").expect("data_engine_kql_parser is present") {
            FoundCrate::Itself => quote! { crate },
            FoundCrate::Name(name) => {
                let crate_name = format_ident!("r#{}", name);
                quote! { #crate_name }
            }
        };

    let rule_conversions = rules.iter().map(|rule| {
        let rule_name = format_ident!("r#{}", rule.name);
        quote! {
            Rule::#rule_name => Self::#rule_name
        }
    });

    let result = quote! {
        impl TryFrom<Rule> for #base_rule_crate_base::base_parser::Rule {
            type Error = data_engine_parser_abstractions::ParserError;

            fn try_from(value: Rule) -> Result<Self, Self::Error> {
                Ok(match value {
                    #(#rule_conversions),*,
                    _ => return Err(data_engine_parser_abstractions::ParserError::RuleConversionError(
                        format!("could not convert {value:?} to base_parser::Rule")
                    ))
                })
            }
        }
    };

    result.into()
}
