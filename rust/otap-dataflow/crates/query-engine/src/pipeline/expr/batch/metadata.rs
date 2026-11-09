// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementations of [`BatchEval`] used for inspecting and retrieving values
//! from Otap batch [`Metadata`].
//!
//! Metadata access is facilitated via a suite of available functions for accessing and (in the
//! future) modifying batch telemetry. For example `has_transport_header(header_name)` is an
//! available function returning a scalar boolean value.
//!
//! The types in this module are used by the expression planner to create the appropriate
//! expressions instances that evaluate these function calls, and also to declare which metadata
//! accesses will occur so that the appropriate context declarations can be populated on the
//! [`NodeBindings`] by the DFE engine.

use std::marker::PhantomData;

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

/// Instances of functions that access batch metadata must implement this trait
pub trait MetadataFunction: BatchEval + Sized {
    /// Name of function
    fn function_name() -> &'static str;

    /// Return type of function
    fn logical_type() -> ExprLogicalType;

    /// Fallible constructor of function instance
    fn try_new(
        node_context_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Self>;

    /// Returns selector for context entry that may be accessed when the function evaluates
    fn context_entry_selector(
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>>;

    fn create_declared_metadata_read(
        entry_selector: &ContextEntrySelector,
        node_context_bindings: &NodeBindings,
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ValueRead>> {
        match node_context_bindings.value(&entry_selector) {
            Ok(r) => Ok(Some(r.clone())),
            Err(ConfigError::InvalidUserConfig { .. }) => {
                // No context entry will be preserved in metadata such that the passed entry
                // selector has something to select
                Ok(None)
            }
            Err(e) => Err(Error::InvalidPipelineError {
                cause: format!("Error planning function '{}': {e}", Self::function_name()),
                query_location: query_location.cloned(),
            }),
        }
    }
}

/// Factory type for creating instances of the [`MetadataFunction`].
///
/// Instances of this type can be also be used to inspect information about the function without
/// creating a concrete instance of which is useful during any query planning/validation that
/// happens before some DFE node has a compiled [`NodeBindings`] instance (as an instance of this
/// would be required to construct the function instance).
pub struct MetadataFunctionFactoryImpl<T: MetadataFunction> {
    _pd: PhantomData<T>,
}

impl<T> MetadataFunctionFactoryImpl<T>
where
    T: MetadataFunction + 'static,
{
    /// Constructor for instance of this type
    pub fn new() -> Self {
        Self {
            _pd: Default::default(),
        }
    }
}

/// Trait wrapper used in cases where there's a need to treat the [`MetadataFunctionFactoryImpl`]
/// as a `dyn` type.
///
/// Basically this type exists because having a `dyn MetadataFunctionFactory` is more convenient
/// during function invocation planning, so code does not need to be generic over the function
/// impl type.
pub trait MetadataFunctionFactory {
    /// Get the return type of the function this factory will create
    fn logical_type(&self) -> ExprLogicalType;

    /// Get the context entry selector that will be used to access some of the batch metadata when
    /// the function this factory creates will be invoked
    fn context_entry_selector(
        &self,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>>;

    /// Create an instance of the metadata function.
    ///
    /// Note: this returns [`BatchEval`] because [`MetadataFunction`] isn't object-safe.
    fn create(
        &self,
        node_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Box<dyn BatchEval>>;
}

impl<T> MetadataFunctionFactory for MetadataFunctionFactoryImpl<T>
where
    T: MetadataFunction + 'static,
{
    fn context_entry_selector(
        &self,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>> {
        T::context_entry_selector(args, query_location)
    }

    fn logical_type(&self) -> ExprLogicalType {
        T::logical_type()
    }

    fn create(
        &self,
        node_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Box<dyn BatchEval>> {
        Ok(Box::new(T::try_new(node_bindings, args, query_location)?))
    }
}

/// Metadata function used as a predicate to check if the batch contains some transport header.
///
/// Signature:
/// ```text
/// has_transport_header(header_name: String) -> Boolean
/// ```
///
/// Returns `true`` if the batch on which this function is invoked has the transport header in its
/// [`Metadata`] or `false` otherwise.
///
/// The header name is the name the header is stored as by the header capture policy (not the
/// original name). Also note the `header_name` is expected to be a static string literal. Any
/// other argument being passed will result in an `InvalidPipeline` error.
#[derive(Debug)]
pub(crate) struct HasTransportHeaderPredicate {
    declared_metadata_read: Option<ValueRead>,
}

impl MetadataFunction for HasTransportHeaderPredicate {
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

    fn try_new(
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
}

impl BatchEval for HasTransportHeaderPredicate {
    fn evaluate(
        &self,
        _otel_arrow: &OtapArrowRecords,
        eval_ctx: &EvalContext<'_>,
    ) -> Result<ScalarValue> {
        Ok(match &self.declared_metadata_read {
            Some(declared_metadata_read) => {
                let metadata = eval_ctx.state().get_metadata();
                ScalarValue::Boolean(Some(value_exists(metadata, declared_metadata_read)?))
            }
            None => ScalarValue::Null,
        })
    }
}

impl HasTransportHeaderPredicate {
    /// Create the [`ContextEntrySelector`] used to access the header whose name is identified
    /// as the first argument.
    fn context_entry_selector_for_header_name(
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<ContextEntrySelector> {
        // validate that we have the expected number of arguments:
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
            // we expect the argument to be a static string
            InvokeFunctionArgument::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(header_name),
            )) => create_transport_header_context_selector(header_name.get_value(), query_location),

            // invalid argument:
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

/// Helper function to create an instance of [`ContextEntrySelector`] that selects a transport
/// header by it's name. This returns an `Err` if the `header_name` is not a valid transport
/// header name.
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

/// Helper to check if the value accessed by the [`ValueRead`] exists in the [`Metadata`].
fn value_exists(metadata: &Metadata, declared_metadata_read: &ValueRead) -> Result<bool> {
    let mut value_exists = false;
    if let Some(read) = declared_metadata_read.read(metadata)? {
        read.visit_members(|read_member| {
            value_exists = read_member.values().next().is_some();
        });
    }

    Ok(value_exists)
}
