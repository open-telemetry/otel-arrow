// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Expression function definitions.
//!
//! Currently in this query-engine, all functions are "well-known" (e.g. effectively part of the
/// "standard library") and identified by name. This module contains the definitions of these
/// functions, which will be used by the expression planner.
use std::sync::Arc;

use arrow::datatypes::DataType;
use datafusion::functions::crypto::{md5, sha256, sha512};
use datafusion::functions::datetime::to_char;
use datafusion::functions::encoding::encode;
use datafusion::functions::math::log10;
use datafusion::functions::string::{ends_with, lower, ltrim, rtrim, starts_with, upper, uuid};
use datafusion::logical_expr::ScalarUDF;
use otel_arrow_contrib_data_engine_expressions::{InvokeFunctionArgument, QueryLocation};
use otel_arrow_dfe_engine::pdata_context::{ContextEntrySelector, NodeBindings};

#[cfg(feature = "sha1-hash")]
use crate::consts::SHA1_FUNC_NAME;
use crate::consts::{
    ENCODE_FUNC_NAME, ENDS_WITH_FUNC_NAME, FNV_FUNC_NAME, FORMAT_DATETIME_FUNC_NAME,
    HAS_TRANSPORT_HEADER_FUNC_NAME, LOG_FUNC_NAME, LOWER_CASE_FUNC_NAME, LTRIM_FUNC_NAME,
    MD5_FUNC_NAME, MURMUR3_FUNC_NAME, NOW_FUNC_NAME, REGEXP_SUBSTR_FUNC_NAME, RTRIM_FUNC_NAME,
    SHA256_FUNC_NAME, SHA512_FUNC_NAME, STARTS_WITH_FUNC_NAME, UPPER_CASE_FUNC_NAME,
    UUID_FUNC_NAME, UUIDV7_FUNC_NAME, XXH3_FUNC_NAME, XXH128_FUNC_NAME,
};
use crate::error::Result;
use crate::pipeline::expr::batch::BatchEval;
use crate::pipeline::expr::batch::metadata::{
    HasTransportHeaderPredicateFactory, MetadataUdfFactory,
};
use crate::pipeline::expr::types::ExprLogicalType;
#[cfg(feature = "sha1-hash")]
use crate::pipeline::functions::sha1_hash;
use crate::pipeline::functions::{
    fnv_hash, murmur3_hash, now, regexp_substr, uuidv7, xxh3_hash, xxh128_hash,
};

/// The definition of the function.
pub(crate) enum FunctionDefinition {
    /// The function will be evaluated as a datafusion `[ScalarUDF]` as part of some datafusion
    /// expression evaluation
    DataFusion(DataFusionFunctionDef),

    /// The function will be evaluated a [`BatchEval`].
    BatchEval(BatchEvalFuncDef),
}

impl FunctionDefinition {
    pub fn from_func_name(func_name: &str) -> Option<Self> {
        // TODO: some functions may produce different result types depending on the input type.
        // In these cases, we may wish to not have a hard-coded return type, and instead attempt
        // to compute the return type from the types of the input expressions.
        // TODO: some of these functions that involve expanding to dictionary, we may wish to
        // implement our own versions that can operate directly on dictionary arrays (or fix this
        // upstream in datafusion_functions)
        Some(match func_name {
            ENCODE_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                encode(),
                ExprLogicalType::String,
                false,
                None,
            )),
            ENDS_WITH_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                ends_with(),
                ExprLogicalType::Boolean,
                true,
                None,
            )),
            LOG_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                log10(),
                ExprLogicalType::Float64,
                true,
                None,
            )),
            LTRIM_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                ltrim(),
                ExprLogicalType::String,
                true,
                None,
            )),
            NOW_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                now(),
                ExprLogicalType::TimestampNanosecond,
                false,
                None,
            )),
            REGEXP_SUBSTR_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                regexp_substr(),
                ExprLogicalType::String,
                false,
                None,
            )),
            FORMAT_DATETIME_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                to_char(),
                ExprLogicalType::String,
                false,
                None,
            )),
            RTRIM_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                rtrim(),
                ExprLogicalType::String,
                true,
                None,
            )),
            SHA256_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                sha256(),
                ExprLogicalType::Binary,
                true,
                None,
            )),
            MD5_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                md5(),
                ExprLogicalType::String,
                true,
                Some(DataType::Utf8),
            )),
            FNV_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                fnv_hash(),
                ExprLogicalType::Int64,
                true,
                None,
            )),
            MURMUR3_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                murmur3_hash(),
                ExprLogicalType::Int64,
                true,
                None,
            )),
            #[cfg(feature = "sha1-hash")]
            SHA1_FUNC_NAME => Self::new(sha1_hash(), ExprLogicalType::Binary, true, None),
            SHA512_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                sha512(),
                ExprLogicalType::Binary,
                true,
                None,
            )),
            STARTS_WITH_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                starts_with(),
                ExprLogicalType::Boolean,
                true,
                None,
            )),
            XXH3_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                xxh3_hash(),
                ExprLogicalType::Int64,
                true,
                None,
            )),
            XXH128_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                xxh128_hash(),
                ExprLogicalType::Binary,
                true,
                None,
            )),
            UUID_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                uuid(),
                ExprLogicalType::String,
                false,
                None,
            )),
            UUIDV7_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                uuidv7(),
                ExprLogicalType::String,
                false,
                None,
            )),
            UPPER_CASE_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                upper(),
                ExprLogicalType::String,
                true,
                None,
            )),
            LOWER_CASE_FUNC_NAME => Self::DataFusion(DataFusionFunctionDef::new(
                lower(),
                ExprLogicalType::String,
                true,
                None,
            )),

            HAS_TRANSPORT_HEADER_FUNC_NAME => Self::BatchEval(BatchEvalFuncDef::new(
                HasTransportHeaderPredicateFactory::new(),
            )),
            _ => return None,
        })
    }
}

/// Definition of function that can be invoked as part of a datafusion expression evaluation
pub(crate) struct DataFusionFunctionDef {
    pub scalar_udf: Arc<ScalarUDF>,
    pub return_type: ExprLogicalType,
    pub requires_dict_downcast: bool,
    pub cast_result_to: Option<DataType>,
}

impl DataFusionFunctionDef {
    fn new(
        scalar_udf: Arc<ScalarUDF>,
        return_type: ExprLogicalType,
        requires_dict_downcast: bool,
        cast_result_to: Option<DataType>,
    ) -> Self {
        Self {
            scalar_udf,
            return_type,
            requires_dict_downcast,
            cast_result_to,
        }
    }
}

// TODO - this type might not be needed, and we could probably move
// this code back into the expr planner?
pub struct BatchEvalFuncDef {
    factory: Box<dyn MetadataUdfFactory>,
}

impl BatchEvalFuncDef {
    fn new(factory: impl MetadataUdfFactory + 'static) -> Self {
        Self {
            factory: Box::new(factory),
        }
    }

    pub fn logical_type(&self) -> ExprLogicalType {
        self.factory.logical_type()
    }

    pub fn context_entry_selector(
        &self,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Option<ContextEntrySelector>> {
        self.factory.context_entry_selector(args, query_location)
    }

    pub fn create_batch_eval_instance(
        &self,
        node_bindings: &NodeBindings,
        args: &[InvokeFunctionArgument],
        query_location: Option<&QueryLocation>,
    ) -> Result<Box<dyn BatchEval>> {
        self.factory.create(node_bindings, args, query_location)
    }
}
