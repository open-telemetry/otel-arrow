// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Condense Attributes processor
#[cfg(feature = "condense-attributes-processor")]
pub mod condense_attributes_processor;

/// Recordset KQL OTLP Query Engine processor
#[cfg(feature = "recordset-kql-processor")]
pub mod recordset_kql_processor;

/// Resource Validator processor for validating resource attributes
#[cfg(feature = "resource-validator-processor")]
pub mod resource_validator_processor;
