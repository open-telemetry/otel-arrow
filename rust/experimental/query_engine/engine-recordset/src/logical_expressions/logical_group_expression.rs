use std::{cell::OnceCell, collections::LinkedList};

use crate::{error::Error, execution_context::ExecutionContext, expression::*};

use super::logical_expression::*;

#[derive(Debug)]
pub struct LogicalGroupExpression {
    id: usize,
    first_expression: Box<dyn LogicalExpression>,
    expressions: LinkedList<LogicalGroupExpressionValue>,
    chain_type: LogicalGroupExpressionChain,
    hash: OnceCell<ExpressionHash>,
}

impl LogicalGroupExpression {
    pub fn new(first_expression: impl LogicalExpression + 'static) -> LogicalGroupExpression {
        Self {
            id: get_next_id(),
            first_expression: Box::new(first_expression),
            expressions: LinkedList::new(),
            chain_type: LogicalGroupExpressionChain::None,
            hash: OnceCell::new(),
        }
    }

    pub fn add_logical_expression_with_and(
        &mut self,
        expression: impl LogicalExpression + 'static,
    ) {
        self.add(LogicalGroupExpressionValue::AndExpression(Box::new(
            expression,
        )));
    }

    pub fn add_logical_expression_with_or(&mut self, expression: impl LogicalExpression + 'static) {
        self.add(LogicalGroupExpressionValue::OrExpression(Box::new(
            expression,
        )));
    }

    fn add(&mut self, expression: LogicalGroupExpressionValue) {
        let chain_type = match expression {
            LogicalGroupExpressionValue::AndExpression(_) => LogicalGroupExpressionChain::AndChain,
            LogicalGroupExpressionValue::OrExpression(_) => LogicalGroupExpressionChain::OrChain,
        };

        if self.chain_type != LogicalGroupExpressionChain::MixedChain {
            if self.chain_type == LogicalGroupExpressionChain::None {
                self.chain_type = chain_type;
            } else if self.chain_type != chain_type {
                self.chain_type = LogicalGroupExpressionChain::MixedChain;
            }
        }

        self.expressions.push_back(expression);
    }
}

impl Expression for LogicalGroupExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"logical_group");
                h.add_bytes(b"expressions:");
                h.add_bytes(self.first_expression.get_hash().get_bytes());
                for expression in self.expressions.iter() {
                    match expression {
                        LogicalGroupExpressionValue::AndExpression(e) => {
                            h.add_bytes(b"&&");
                            h.add_bytes(e.get_hash().get_bytes());
                        }
                        LogicalGroupExpressionValue::OrExpression(e) => {
                            h.add_bytes(b"||");
                            h.add_bytes(e.get_hash().get_bytes());
                        }
                    }
                }
            })
        })
    }

    fn write_debug(
        &self,
        execution_context: &dyn ExecutionContext,
        _heading: &'static str,
        level: i32,
        output: &mut String,
    ) {
        let padding = "\t".repeat(level as usize);

        output.push_str(&padding);
        output.push_str("(\n");

        self.first_expression
            .write_debug(execution_context, "", level + 1, output);

        for expression in self.expressions.iter() {
            let inner_expression_operation_type;
            let inner_expression;
            match expression {
                LogicalGroupExpressionValue::AndExpression(e) => {
                    inner_expression_operation_type = LogicalGroupExpressionChain::AndChain;
                    inner_expression = e;
                }
                LogicalGroupExpressionValue::OrExpression(e) => {
                    inner_expression_operation_type = LogicalGroupExpressionChain::OrChain;
                    inner_expression = e;
                }
            }

            output.push_str(&padding);
            output.push(' ');
            match inner_expression_operation_type {
                LogicalGroupExpressionChain::AndChain => output.push_str("&&"),
                LogicalGroupExpressionChain::OrChain => output.push_str("||"),
                _ => panic!(),
            }
            output.push('\n');

            inner_expression.write_debug(execution_context, "", level + 1, output);
        }

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl LogicalExpressionInternal for LogicalGroupExpression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<bool, Error>
    where
        'a: 'b,
    {
        let mut result = self.first_expression.evaluate(execution_context)?;
        let chain_type = self.chain_type;

        for expression in self.expressions.iter() {
            if !result && chain_type == LogicalGroupExpressionChain::AndChain {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info("LogicalGroupExpression AND chain clause evaluated as false and short-circuited".to_string()));
                break;
            }

            if result && chain_type == LogicalGroupExpressionChain::OrChain {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info("LogicalGroupExpression OR chain clause evaluated as true and short-circuited".to_string()));
                break;
            }

            match expression {
                LogicalGroupExpressionValue::AndExpression(e) => {
                    result = result && e.evaluate(execution_context)?;
                }
                LogicalGroupExpressionValue::OrExpression(e) => {
                    result = result || e.evaluate(execution_context)?;
                }
            }
        }

        execution_context.add_message_for_expression(
            self,
            ExpressionMessage::info(
                either!(result => "LogicalGroupExpressionn evaluated as true"; "LogicalGroupExpression evaluated as false").to_string()));

        Ok(result)
    }
}

#[derive(Debug)]
pub(crate) enum LogicalGroupExpressionValue {
    AndExpression(Box<dyn LogicalExpression>),
    OrExpression(Box<dyn LogicalExpression>),
}

#[derive(
    Debug,     // enables formatting in "{:?}"
    Clone,     // required by Copy
    Copy,      // enables copy-by-value semantics
    PartialEq, // enables value equality (==)
)]
enum LogicalGroupExpressionChain {
    None = 0,
    AndChain = 1,
    OrChain = 2,
    MixedChain = 3,
}
