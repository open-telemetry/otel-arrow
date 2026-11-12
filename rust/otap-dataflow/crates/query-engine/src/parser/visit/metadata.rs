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

    /// Convert into a list of declared contexts entry accesses performed by the query.
    ///
    /// # Invariant
    ///
    /// The returned `entries` slice inside each `ContextConsumerSelector::Entries`
    /// **must** be sorted and deduplicated. The engine's
    /// `CompiledNodeState::declarations_match` compares declarations
    /// structurally, so unsorted or duplicate entries would cause
    /// `pipeline_context_compatible` to reject live-update candidates that are
    /// semantically identical (e.g. a query that reorders header accesses or
    /// references the same header in multiple branches).
    #[must_use]
    pub fn into_context_declarations(mut self) -> Vec<ContextDeclaration> {
        if self.entry_reads.is_empty() {
            return Vec::new();
        }
        self.entry_reads.sort();
        self.entry_reads.dedup();
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

#[cfg(test)]
mod tests {
    use otel_arrow_contrib_data_engine_kql_parser::Parser;
    use otel_arrow_dfe_engine::pdata_context::{
        ContextConsumerSelector, ContextDeclaration, ContextDomain, ContextEntrySelector,
        ContextEntrySelectorForm, ContextEntryTake, ContextEntryTarget, ContextNameMatching,
    };
    use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

    use crate::parser::default_parser_options;
    use crate::parser::visit::visit_expr_ast;

    use super::MetadataAccessDeclarationVisitor;

    /// Parse an OPL query and return the context declarations produced by
    /// the metadata visitor.
    fn declarations_for_query(query: &str) -> Vec<ContextDeclaration> {
        let parsed = OplParser::parse_with_options(query, default_parser_options()).unwrap();
        let mut visitor = MetadataAccessDeclarationVisitor::new();
        visit_expr_ast(&parsed.pipeline, &mut visitor).unwrap();
        visitor.into_context_declarations()
    }

    /// Build the expected selector for a transport header with the given name.
    ///
    /// Must match the selector produced by
    /// `create_transport_header_context_selector` in the metadata module.
    fn header_selector(name: &str) -> ContextEntrySelector {
        ContextEntrySelector {
            target: ContextEntryTarget::Primitive {
                domain: ContextDomain::TransportHeader,
                name: name.try_into().unwrap(),
                matching: ContextNameMatching::Exact,
            },
            form: ContextEntrySelectorForm::StoredKeyValue,
            take: ContextEntryTake::First,
        }
    }

    /// Extract the entry selectors from a single Consumes declaration.
    fn unwrap_entries(declarations: Vec<ContextDeclaration>) -> Vec<ContextEntrySelector> {
        assert_eq!(declarations.len(), 1);
        match declarations.into_iter().next().unwrap() {
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries { entries },
            } => entries.into_vec(),
            other => panic!("expected Consumes/Entries, got {other:?}"),
        }
    }

    /// Scenario: a query references the same transport header in two branches
    ///     of an if-expression.
    /// Guarantees: duplicate entry reads are collapsed to a single selector.
    #[test]
    fn duplicate_header_references_are_deduped() {
        let declarations = declarations_for_query(
            r#"signals | if (has_transport_header("foo")) {
                set resource.attributes["x"] = "a"
            } else {
                set resource.attributes["x"] = has_transport_header("foo")
            }"#,
        );
        let entries = unwrap_entries(declarations);
        assert_eq!(entries.len(), 1, "duplicate reads should be deduplicated");
        assert_eq!(entries[0], header_selector("foo"));
    }

    /// Scenario: a query references two different transport headers in an
    ///     order that differs from their natural sort order (z before a).
    /// Guarantees: the resulting entry reads are sorted regardless of the
    ///     order they appear in the query, so that structurally equal
    ///     declarations are produced for semantically equivalent queries.
    #[test]
    fn header_references_are_sorted() {
        let decl_z_first = declarations_for_query(
            r#"signals | if (has_transport_header("z_header")) {
                set resource.attributes["x"] = "a"
            } else {
                set resource.attributes["x"] = has_transport_header("a_header")
            }"#,
        );
        let decl_a_first = declarations_for_query(
            r#"signals | if (has_transport_header("a_header")) {
                set resource.attributes["x"] = "a"
            } else {
                set resource.attributes["x"] = has_transport_header("z_header")
            }"#,
        );

        let entries_z = unwrap_entries(decl_z_first);
        let entries_a = unwrap_entries(decl_a_first);

        // Both must produce the same sorted order.
        assert_eq!(
            entries_z, entries_a,
            "entries should be in the same order regardless of query order"
        );
        assert_eq!(entries_z.len(), 2);
        assert_eq!(entries_z[0], header_selector("a_header"));
        assert_eq!(entries_z[1], header_selector("z_header"));
    }

    /// Scenario: a query contains no metadata function calls.
    /// Guarantees: the visitor produces an empty declaration list.
    #[test]
    fn no_metadata_access_yields_empty_declarations() {
        let declarations = declarations_for_query(r#"signals | where attributes["x"] == "y""#);
        assert!(declarations.is_empty());
    }
}
