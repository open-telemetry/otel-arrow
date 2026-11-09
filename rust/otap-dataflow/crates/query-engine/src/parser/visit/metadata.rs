// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for visiting parser's expression AST for determining which batch metadata may be
//! during accessed execution of the query.
//!
//! This is needed so it can be determined which [`ContextDeclaration`]s must be given to the DFE
//! engine by the node wishing to execute the query.

use otel_arrow_contrib_data_engine_expressions::{
    Expression, PipelineFunction, PipelineFunctionImplementation, ScalarExpression,
};
use otel_arrow_dfe_engine::pdata_context::{
    ContextConsumerSelector, ContextDeclaration, ContextEntrySelector,
};

use crate::error::Result;
use crate::pipeline::expr::planner::func::FunctionDefinition;

use super::ExprAstVisitor;

/// [`ExprAstVisitor`] implementation for determining which metadata will be accessed by the
/// parsed query and tracking required context declarations.
#[derive(Default)]
pub struct MetadataAccessDeclarationVisitor {
    entry_reads: Vec<ContextEntrySelector>,
}

impl MetadataAccessDeclarationVisitor {
    /// Create a new instance
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert into a list of declared contexts entry accesses performed by the query
    #[must_use]
    pub fn into_context_declarations(self) -> Vec<ContextDeclaration> {
        vec![ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::Entries {
                entries: self.entry_reads.into_boxed_slice(),
            },
        }]
    }
}

impl ExprAstVisitor for MetadataAccessDeclarationVisitor {
    fn visit_scalar_expr(
        &mut self,
        scalar_expr: &ScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<()> {
        match scalar_expr {
            ScalarExpression::InvokeFunction(invoke_expr) => {
                // get the definition of the function by its name
                let func_def = functions
                    .get(invoke_expr.get_function_id())
                    .and_then(|function| match function.get_implementation() {
                        PipelineFunctionImplementation::External(func_name) => {
                            Some(func_name.as_ref())
                        }
                        _ => None,
                    })
                    .and_then(FunctionDefinition::from_func_name);

                // if it is a BatchEval function, it may be evaluating on batch metadata...
                // capture any metadata accesses it may perform via a context entry selector
                if let Some(FunctionDefinition::BatchEval(func_impl_factory)) = func_def
                    && let Some(context_entry_selector) = func_impl_factory.context_entry_selector(
                        invoke_expr.get_arguments(),
                        Some(invoke_expr.get_query_location()),
                    )?
                {
                    self.entry_reads.push(context_entry_selector)
                }
            }
            _ => {
                // nothing to do - currently metadata is _only_ accessed through invocation
                // of special well known functions.
            }
        }

        Ok(())
    }
}
