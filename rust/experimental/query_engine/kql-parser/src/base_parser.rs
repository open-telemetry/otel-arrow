// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Base for parsers that support KQL-like syntax based on the grammar defined in base.pest
//!
//! Multiple query languages may share a common base grammar for shared constructs such as
//! scalar expressions, but differ in their overall query structure or supported tabular expressions.
//! Typically, parsers for specific KQL-like languages will use the grammar defined in base.pest
//! along with their own grammar file to define the full language syntax.
//!
//! In order to have common utilities for parsing the expressions in base.pest, we define a
//! BasePestParser here that only includes the base.pest grammar. Many of the parsing utilities
//! for handling scalar expressions will be generic over derived `Rule` types, that can be converted
//! to the base rules derived for [`BasePestParser`].
//!
//! This module also defines the [`TryAsBaseRule`] trait for converting derived parser `Rule` to the
//! base parser `Rule`. It's not necessary to implement this trait manually, as it can be derived
//! ```ignore`
//! use data_engine_kql_parser_macros::BaseRuleCompatible;
//!
//! #[derive(pest_derive::Parser, BaseRuleCompatible)]
//! #[grammar = "path/to/derived_language.pest"]
//! struct DerivedLanguagePestParser {}
//! ```

use data_engine_kql_parser_macros::ScalarExprPrattParser;
use data_engine_parser_abstractions::ParserError;
use pest::{RuleType, iterators::Pair};

use crate::ScalarExprRules;

#[derive(pest_derive::Parser, ScalarExprPrattParser)]
#[grammar = "base.pest"]
pub struct BasePestParser;

/// Trait for converting derived parser Rule types to the base parser Rule type.
/// This is used to allow parsing utilities to work with different parser Rule types
/// that share a common base grammar.
///
/// It's not necessary to implement this trait manually, as it can be derived for any
/// parsers that use the base.pest grammar via the
/// [`BaseRuleCompatible`](data_engine_kql_parser_macros::BaseRuleCompatible) macro.
pub trait TryAsBaseRule {
    fn try_as_base_rule(&self) -> Result<Rule, ParserError>;
}

impl<T, E> TryAsBaseRule for Pair<'_, T>
where
    Rule: TryFrom<T, Error = E>,
    T: RuleType + ScalarExprRules + 'static,
    E: Into<ParserError>,
{
    fn try_as_base_rule(&self) -> Result<Rule, ParserError> {
        Rule::try_from(self.as_rule()).map_err(|e| e.into())
    }
}
