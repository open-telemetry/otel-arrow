// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! TODO explain what this is doing

use proc_macro::TokenStream;
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

#[proc_macro_derive(ScalarExprPrattParser)]
pub fn derive_scalar_expr_pratt_parser(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let base_rule_crate_base =
        match crate_name("data_engine_kql_parser").expect("data_engine_kql_parser is present") {
            FoundCrate::Itself => quote! { crate },
            FoundCrate::Name(name) => {
                let crate_name = format_ident!("r#{}", name);
                quote! { #crate_name }
            }
        };

    let result = quote! {
        static PRATT_PARSER: std::sync::LazyLock<pest::pratt_parser::PrattParser<Rule>> = std::sync::LazyLock::new(|| {
            use pest::pratt_parser::{Assoc::*, Op, PrattParser};
            use Rule::*;

            // Lowest precedence first
            PrattParser::new()
                // or
                .op(Op::infix(or_token, Left))
                // and
                .op(Op::infix(and_token, Left))
                // == !=
                .op(Op::infix(equals_token, Left)
                    | Op::infix(equals_insensitive_token, Left)
                    | Op::infix(not_equals_token, Left)
                    | Op::infix(not_equals_insensitive_token, Left)
                    | Op::infix(invalid_equals_token, Left))
                // <= >= < >
                .op(Op::infix(less_than_or_equal_to_token, Left)
                    | Op::infix(greater_than_or_equal_to_token, Left)
                    | Op::infix(less_than_token, Left)
                    | Op::infix(greater_than_token, Left))
                // contains & has
                .op(Op::infix(not_contains_cs_token, Left)
                    | Op::infix(not_contains_token, Left)
                    | Op::infix(not_has_cs_token, Left)
                    | Op::infix(not_has_token, Left)
                    | Op::infix(contains_cs_token, Left)
                    | Op::infix(contains_token, Left)
                    | Op::infix(has_cs_token, Left)
                    | Op::infix(has_token, Left))
                // in
                .op(Op::infix(not_in_insensitive_token, Left)
                    | Op::infix(not_in_token, Left)
                    | Op::infix(in_insensitive_token, Left)
                    | Op::infix(in_token, Left))
                // matches
                .op(Op::infix(matches_regex_token, Left))
                // + -
                .op(Op::infix(plus_token, Left) | Op::infix(minus_token, Left))
                // * / %
                .op(Op::infix(multiply_token, Left)
                    | Op::infix(divide_token, Left)
                    | Op::infix(modulo_token, Left))

            // ^ ** (right-associative)
            //.op(Op::infix(power, Right))
        });


        impl #base_rule_crate_base::ScalarExprPrattParser for Rule {
            fn pratt_parser() -> &'static pest::pratt_parser::PrattParser<Self> {
                &PRATT_PARSER
            }
        }
    };

    result.into()
}
