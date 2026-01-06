// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_parser_abstractions::ParserError;
use pest::{RuleType, iterators::Pair};

/// TODO comment on what's going on with this
#[derive(pest_derive::Parser)]
#[grammar = "base.pest"]
pub(crate) struct BasePestParser;

pub fn to_base<'a, T>(value: Pair<'a, T>) -> Result<Pair<'a, Rule>, ParserError>
where
    T: RuleType + TryInto<Rule, Error = ParserError>,
{
    todo!()
}
