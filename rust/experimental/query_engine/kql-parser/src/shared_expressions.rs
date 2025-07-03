use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, scalar_expression::parse_scalar_expression,
    scalar_primitive_expressions::parse_accessor_expression,
};

pub(crate) fn parse_assignment_expression(
    assignment_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<TransformExpression, ParserError> {
    let query_location = to_query_location(&assignment_expression_rule);

    let mut assignment_rules = assignment_expression_rule.into_inner();

    let destination_rule = assignment_rules.next().unwrap();
    let destination_rule_location = to_query_location(&destination_rule);
    let destination_rule_str = destination_rule.as_str();

    let accessor = match destination_rule.as_rule() {
        // Note: Root-level static accessors are not valid in an assignment
        // expression so allow_root_scalar=false is passed here. Example:
        // accessor(some_constant1) = [expression] cannot be folded as
        // String("constant1") = [expression] we need to treat the accessor in
        // this case as an assignment on the source
        // Source(MapKey("some_constant1")) = [expression].
        Rule::accessor_expression => parse_accessor_expression(destination_rule, state, false)?,
        _ => panic!("Unexpected rule in assignment_expression: {destination_rule}"),
    };

    let destination = match accessor {
        ScalarExpression::Source(s) => {
            if !s.get_value_accessor().has_selectors() {
                return Err(ParserError::SyntaxError(
                    destination_rule_location,
                    format!(
                        "Cannot write directly to '{}' in an assignment expression use an accessor expression referencing a path on source instead",
                        destination_rule_str.trim()
                    ),
                ));
            }

            MutableValueExpression::Source(s)
        }
        ScalarExpression::Variable(v) => MutableValueExpression::Variable(v),
        _ => {
            return Err(ParserError::SyntaxError(
                destination_rule_location,
                format!(
                    "'{}' destination accessor must refer to source or a variable to be used in an assignment expression",
                    destination_rule_str.trim()
                ),
            ));
        }
    };

    let source_rule = assignment_rules.next().unwrap();

    let scalar = match source_rule.as_rule() {
        Rule::scalar_expression => parse_scalar_expression(source_rule, state)?,
        _ => panic!("Unexpected rule in assignment_expression: {source_rule}"),
    };

    Ok(TransformExpression::Set(SetTransformExpression::new(
        query_location,
        ImmutableValueExpression::Scalar(scalar),
        destination,
    )))
}

pub(crate) fn parse_let_expression(
    let_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<TransformExpression, ParserError> {
    let query_location = to_query_location(&let_expression_rule);

    let mut let_rules = let_expression_rule.into_inner();

    let identifier_rule = let_rules.next().unwrap();

    let name = identifier_rule.as_str().trim();
    if state.is_well_defined_identifier(name) {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: to_query_location(&identifier_rule).clone(),
            diagnostic_id: "KS201",
            message: format!("A variable with the name '{name}' has already been declared"),
        });
    }

    let identifier = StringScalarExpression::new(to_query_location(&identifier_rule), name);

    let scalar = parse_scalar_expression(let_rules.next().unwrap(), state)?;

    Ok(TransformExpression::Set(SetTransformExpression::new(
        query_location,
        ImmutableValueExpression::Scalar(scalar),
        MutableValueExpression::Variable(VariableScalarExpression::new(
            identifier.get_query_location().clone(),
            identifier.get_value(),
            ValueAccessor::new(),
        )),
    )))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_assignment_expression() {
        let run_test_success = |input: &str, expected: TransformExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::assignment_expression, input).unwrap();

            let expression = parse_assignment_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::assignment_expression, input).unwrap();

            let error = parse_assignment_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        run_test_success(
            "new_attribute = 1",
            TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "new_attribute",
                        )),
                    )]),
                )),
            )),
        );

        run_test_success(
            "variable = 'hello world'",
            TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello world",
                    )),
                )),
                MutableValueExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
            )),
        );

        run_test_failure(
            "source = 1",
            "Cannot write directly to 'source' in an assignment expression use an accessor expression referencing a path on source instead",
        );

        run_test_failure(
            "resource.attributes['new_attribute'] = 1",
            "'resource.attributes['new_attribute']' destination accessor must refer to source or a variable to be used in an assignment expression",
        );
    }

    #[test]
    pub fn test_parse_let_expression() {
        let run_test_success = |input: &str, expected: TransformExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::let_expression, input).unwrap();

            let expression = parse_let_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::let_expression, input).unwrap();

            let error = parse_let_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "let var1 = 1;",
            TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )),
                MutableValueExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "var1",
                    ValueAccessor::new(),
                )),
            )),
        );

        run_test_failure(
            "let variable = 1;",
            "KS201",
            "A variable with the name 'variable' has already been declared",
        );

        run_test_failure(
            "let resource = 1;",
            "KS201",
            "A variable with the name 'resource' has already been declared",
        );
    }
}
