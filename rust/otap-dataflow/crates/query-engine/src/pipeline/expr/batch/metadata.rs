// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementations of [`BatchEval`] used for inspecting and retrieving values
//! from Otap batch [`Metadata`]

use datafusion::scalar::ScalarValue;

use otel_arrow_contrib_data_engine_expressions::{
    InvokeFunctionArgument, QueryLocation, ScalarExpression, StaticScalarExpression, StringValue,
};
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::pdata_context::{
    ContextDomain, ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake,
    ContextEntryTarget, ContextNameMatching, Metadata, NodeBindings, ValueRead,
};
use otel_arrow_dfe_pdata::OtapArrowRecords;

use crate::error::{Error, Result};
use crate::pipeline::expr::eval::EvalContext;
use crate::pipeline::expr::types::ExprLogicalType;

use super::BatchEval;

pub trait MetadataUdfImpl {
    /// TODO commentary
    fn function_name() -> &'static str;

    /// TODO commentary
    fn context_entry_selector(
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>>;

    fn logical_type() -> ExprLogicalType;

    fn create_declared_metadata_read(
        entry_selector: &ContextEntrySelector,
        node_context_bindings: &NodeBindings,
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ValueRead>> {
        match node_context_bindings.value(&entry_selector) {
            Ok(r) => Ok(Some(r.clone())),
            Err(ConfigError::InvalidUserConfig { .. }) => {
                // No context entry will be preserved in metadata with the given name
                Ok(None)
            }
            Err(e) => Err(Error::InvalidPipelineError {
                cause: format!("Error planning function '{}': {e}", Self::function_name()),
                query_location: query_location.cloned(),
            }),
        }
    }
}

pub trait MetadataUdfFactory {
    fn logical_type(&self) -> ExprLogicalType;

    fn context_entry_selector(
        &self,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>>;

    fn create(
        &self,
        node_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Box<dyn BatchEval>>;
}

#[derive(Default)]
pub struct HasTransportHeaderPredicateFactory {}

impl HasTransportHeaderPredicateFactory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MetadataUdfFactory for HasTransportHeaderPredicateFactory {
    fn logical_type(&self) -> ExprLogicalType {
        HasTransportHeaderPredicate::logical_type()
    }

    fn context_entry_selector(
        &self,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>> {
        HasTransportHeaderPredicate::context_entry_selector(args, query_location)
    }

    fn create(
        &self,
        node_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Box<dyn BatchEval>> {
        Ok(Box::new(HasTransportHeaderPredicate::try_new(
            node_bindings,
            args,
            query_location,
        )?))
    }
}

/// TODO docs
#[derive(Debug)]
pub(crate) struct HasTransportHeaderPredicate {
    declared_metadata_read: Option<ValueRead>,
}

impl MetadataUdfImpl for HasTransportHeaderPredicate {
    fn function_name() -> &'static str {
        "has_transport_header"
    }

    fn logical_type() -> ExprLogicalType {
        ExprLogicalType::Boolean
    }

    fn context_entry_selector(
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>> {
        Self::context_entry_selector_for_header_name(args, query_location).map(Some)
    }
}

impl HasTransportHeaderPredicate {
    pub fn try_new(
        node_context_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Self> {
        let entry_selector = Self::context_entry_selector_for_header_name(args, query_location)?;
        let declared_metadata_read = Self::create_declared_metadata_read(
            &entry_selector,
            node_context_bindings,
            query_location,
        )?;

        Ok(Self {
            declared_metadata_read,
        })
    }

    fn context_entry_selector_for_header_name(
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<ContextEntrySelector> {
        if args.len() != 1 {
            return Err(Error::InvalidPipelineError {
                cause: format!(
                    "Invalid num argument for {}. Expected 1, got {}",
                    Self::function_name(),
                    args.len()
                ),
                query_location: query_location.cloned(),
            });
        }

        // safety: we've checked that there is one arg in "if" block above
        match args.first().expect("one arg") {
            InvokeFunctionArgument::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(header_name),
            )) => create_transport_header_context_selector(header_name.get_value(), query_location),
            other => Err(Error::InvalidPipelineError {
                cause: format!(
                    "Invalid argument for {} at position 0. Expected static string, received {other:?}",
                    Self::function_name(),
                ),
                query_location: query_location.cloned(),
            }),
        }
    }
}

impl BatchEval for HasTransportHeaderPredicate {
    fn evaluate(
        &self,
        _otel_arrow: &OtapArrowRecords,
        eval_ctx: &EvalContext<'_>,
    ) -> Result<ScalarValue> {
        let result = match &self.declared_metadata_read {
            Some(declared_metadata_read) => {
                println!("IT's SOME");
                let metadata = eval_ctx.state().get_metadata();
                ScalarValue::Boolean(Some(value_exists(metadata, declared_metadata_read)?))
            }
            None => {
                println!("ITS null");
                ScalarValue::Null
            }
        };

        Ok(result)
    }
}

fn create_transport_header_context_selector(
    header_name: &str,
    query_location: Option<&QueryLocation>,
) -> Result<ContextEntrySelector> {
    Ok(ContextEntrySelector {
        target: ContextEntryTarget::Primitive {
            domain: ContextDomain::TransportHeader,
            name: ContextEntryName::try_from(header_name).map_err(|e| {
                Error::InvalidPipelineError {
                    cause: format!("Invalid transport header name '{header_name}': {e}"),
                    query_location: query_location.cloned(),
                }
            })?,
            matching: ContextNameMatching::Exact,
        },
        form: ContextEntrySelectorForm::StoredKeyValue,
        take: ContextEntryTake::First,
    })
}

/// TODO commentary
fn value_exists(metadata: &Metadata, declared_metadata_read: &ValueRead) -> Result<bool> {
    let mut value_exists = false;
    if let Some(read) = declared_metadata_read.read(metadata)? {
        read.visit_members(|read_member| {
            value_exists = read_member.values().next().is_some();
        });
    }

    Ok(value_exists)
}
