// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_parser_abstractions::{ParserError, ParserState};
use pest::iterators::Pair;

use crate::parser::Rule;
use crate::parser::operator::parse_operator_call;

pub(crate) fn parse_pipeline(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::source => {
                // TODO - will this be needed later in parser?
                // ignore for now
            }
            Rule::pipeline_stage => parse_pipeline_stage(rule, state)?,
            _ => {
                todo!("handle invalid rule in pipeline {rule:?}")
            }
        }
    }
    Ok(())
}

pub(crate) fn parse_pipeline_stage(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::operator_call => parse_operator_call(rule, state)?,
            _ => {
                todo!("handle invalid rule in pipeline_stage {rule:?}")
            }
        }
    }

    Ok(())
}
