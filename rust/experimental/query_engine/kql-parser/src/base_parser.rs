// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_parser_abstractions::ParserError;
use data_engine_parser_macros::ScalarExprPrattParser;
use pest::{RuleType, iterators::Pair};
use proc_macro2::TokenStream;

/// TODO comment on what's going on with this
#[derive(pest_derive::Parser, ScalarExprPrattParser)]
#[grammar = "base.pest"]
pub struct BasePestParser;
