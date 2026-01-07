// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains macros for parser implementations that use the base KQL pest grammar.
//! These macros help with converting between derived parser `Rule` types and the base parser
//! `Rule` type, as well as providing a Pratt parser implementation for scalar expressions.

use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};

const BASE_PEST_SOURCE: &'static str = include_str!("../../base.pest");

extern crate proc_macro;

/// This marco derives the implementation of `TryFrom<`Rule`>` for `kql_parser::base_rule::Rule` 
/// for  `Rule` enum derived from the pest_derive::Parser marco. This allows converting between
/// the derived parser `Rule` type and the base parser `Rule` type, which allows the rules for 
/// the derived parser to used in the parsing utilities in the kql-parser crate.
#[proc_macro_derive(BaseRuleCompatible)]
pub fn derive_base_rule_compatible(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // parse the base.pest grammar to get the list of rules
    let pairs =
        pest_meta::parser::parse(pest_meta::parser::Rule::grammar_rules, BASE_PEST_SOURCE).unwrap();
    let ast = pest_meta::parser::consume_rules(pairs).unwrap();
    let rules = pest_meta::optimizer::optimize(ast);

    let rule_conversions = rules.iter().map(|rule| {
        let rule_name = format_ident!("r#{}", rule.name);
        quote! {
            Rule::#rule_name => Self::#rule_name
        }
    });

    // derive `TryFrom` the derived rule for the base rule.
    let base_rule_crate_base = kql_parser_crate_name();
    let rule_conversion = quote! {
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

    // derive the pratt parser for the scalar expressions
    let scalar_expr_pratt_parser_impl = generate_scalar_expr_pratt_parser();
    
    quote! {
        #rule_conversion
        #scalar_expr_pratt_parser_impl
    }.into()
}

/// Determine the crate path for data_engine_kql_parser, either "crate" if the macro is being 
/// executed in the kql parser crate, otheriwse reference the crate by name.
fn kql_parser_crate_name() -> proc_macro2::TokenStream {
    match crate_name("data_engine_kql_parser").expect("data_engine_kql_parser is present") {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let crate_name = format_ident!("r#{}", name);
            quote! { #crate_name }
        }
    }
}

#[proc_macro_derive(ScalarExprPrattParser)]
pub fn derive_scalar_expr_pratt_parser(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate_scalar_expr_pratt_parser().into()
}

/// Parsing scalar expressions uses a pratt parser, which takes Pair<Rule> as input. Because pest
/// doesn't expose a way to generically convert between different parser Pair types, we define
/// the pratt parser for each derived parser Rule type. This macro generates the pratt parser
/// and a trait implementation for the derived parser Rule type to access it.
fn generate_scalar_expr_pratt_parser() -> proc_macro2::TokenStream {
    let base_rule_crate_base = kql_parser_crate_name();

    quote! {
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


        impl #base_rule_crate_base::ScalarExprRules for Rule {
            fn pratt_parser() -> &'static pest::pratt_parser::PrattParser<Self> {
                &PRATT_PARSER
            }
        }
    }
}
