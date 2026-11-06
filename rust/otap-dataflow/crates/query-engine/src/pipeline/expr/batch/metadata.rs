// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementations of [`BatchEval`] used for inspecting and retrieving values
//! from Otap batch [`Metadata`]

use datafusion::scalar::ScalarValue;

use otel_arrow_contrib_data_engine_expressions::QueryLocation;
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::pdata_context::{
    ContextDomain, ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake,
    ContextEntryTarget, ContextNameMatching, Metadata, NodeBindings, ValueRead,
};
use otel_arrow_dfe_pdata::OtapArrowRecords;

use crate::error::{Error, Result};
use crate::pipeline::expr::eval::EvalContext;

use super::BatchEval;

trait MetadataUdfImpl {
    fn function_name() -> &'static str;

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

/// TODO docs
#[derive(Debug)]
pub(crate) struct HasTransportHeaderPredicate {
    declared_metadata_read: Option<ValueRead>,
}

impl MetadataUdfImpl for HasTransportHeaderPredicate {
    fn function_name() -> &'static str {
        "has_transport_header"
    }
}

impl HasTransportHeaderPredicate {
    pub fn try_new(
        header_name: &str,
        node_context_bindings: &NodeBindings,
        query_location: Option<&QueryLocation>,
    ) -> Result<Self> {
        let entry_selector = create_transport_header_context_selector(header_name, query_location)?;
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
        let result = match &self.declared_metadata_read {
            Some(declared_metadata_read) => {
                println!("IT's SOME");
                let metadata = eval_ctx.state().get_metadata();
                ScalarValue::Boolean(Some(value_exists(metadata, declared_metadata_read)?))
            }
            None => {
                println!("ITS null");
                ScalarValue::Null
            },
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
