//! # Transformation Plan Module
//!
//! This module defines the `TransformationPlan` and its supporting structures and
//! abstractions to manage column-based and multi-column transformations for structured
//! datasets (e.g., Arrow tables). It provides a flexible and reusable framework for
//! defining and applying transformations by encapsulating operations in a clear, modular manner.
//!
//! ## Key Components
//!
//! ### Structs
//!
//! - **[`TransformationPlan`]:**
//!   Represents a full transformation plan, containing both column-level and multi-column-level operations.
//!   It acts as the primary interface for specifying transformations. Internally, it uses a `ColumnOperations`
//!   struct to manage single-column transformations and a `Vec` to manage multi-column transformations.
//!
//! - **[`ColumnOperations`]:**
//!   Encapsulates and manages single-column transformations. Each column can have a list of operations
//!   (e.g., renaming, type casting). This abstraction keeps the single-column logic separate and flexible.
//!
//! ### Enums
//!
//! - **[`ColumnTransformOp`]:**
//!   Represents individual transformations that can be applied to a single column (e.g., renaming, adding offsets, re-indexing).
//!
//! - **[`MultiColumnTransformOp`]:**
//!   Represents transformations that span multiple columns, such as flattening nested structures or converting attributes into different formats.
//!
//! ### Core Methods in `TransformationPlan`
//!
//! - **`from_config`**: Constructs a `TransformationPlan` based on a given `ArrowPayloadType` and configuration.
//!   Handles both column-level and multi-column transformations depending on the data type and configuration.
//!
//! - **`merge_from`**: Merges another `TransformationPlan` into the current one, combining column-level and multi-column-level transformations.
//!
//! - **`merged`**: Returns a new `TransformationPlan` that combines the transformations of two plans without modifying either.
//!
//! ### Core Methods in `ColumnOperations`
//!
//! - **`add_op`**: Adds a specific operation to a given column’s transformation list.
//! - **`clear`**: Clears all operations in the `ColumnOperations` map.
//! - **`rename_column`**: Schedules a column to be renamed.
//! - **`cast_column`**: Schedules a cast operation for a specific column.
//! - **`noop_column`**: Marks a column as a no-op, leaving it unchanged during the transformation.
//! - **`get_ops`**: Retrieves the list of operations associated with a given column (if any).
//!
//! ### Usage
//!
//! The `TransformationPlan` is designed to flexibly handle a variety of transformation scenarios,
//! from simple renaming or casting of columns to complex restructuring of nested data. By combining
//! column-level and multi-column-level transformations, it can be adapted to meet diverse requirements.
//!
//! ### Extensibility
//!
//! - To add new column-level transformations, extend the `ColumnTransformOp` enum and add any necessary
//!   business logic in the respective handling functions.
//! - To add new multi-column transformations, extend the `MultiColumnTransformOp` enum and define the application logic.
//! - New payload types can be added by extending the logic in `from_config` to account for their specific attribute handling.
//!
use std::collections::HashMap;

use arrow::datatypes::DataType;

use crate::clickhouse_exporter::config::{AttributeRepresentation, Config};
use crate::clickhouse_exporter::consts as ch_consts;
use otap_df_pdata::{proto::opentelemetry::arrow::v1::ArrowPayloadType, schema::consts};

const OTAP_SPAN_FLAGS: &str = "span_flags";

/// Describes how struct fields should be expanded into new columns.
#[derive(Clone, Debug, PartialEq)]
pub struct FlattenStructSpec {
    /// Mapping of struct field name -> output column name
    /// e.g. "id" -> "resource_id"
    pub field_mapping: HashMap<String, String>,

    /// Remove the struct column or not
    pub remove_struct_col: bool,
}

/// Describes how a set of typed value structs should be coerced to a string value.
#[derive(Clone, Debug, PartialEq)]
pub struct CoerceStructStringSpec {
    /// The name of the type field
    pub type_field: String,
    /// The name of the output column
    pub output_column: String,
}

/// Selects a proto enum → string mapping function.
///
/// Used with [`ColumnTransformOp::EnumToString`] to convert an `Int32` Arrow column
/// (possibly dictionary-encoded) into a `StringArray` using the proto enum's
/// `as_str_name()` method.
#[derive(Clone, Debug, PartialEq)]
pub enum EnumStringMapper {
    /// Maps `trace::v1::span::SpanKind` integer values to their proto string names
    /// (e.g. 2 → `"SPAN_KIND_SERVER"`).
    SpanKind,
    /// Maps `trace::v1::status::StatusCode` integer values to their proto string names
    /// (e.g. 2 → `"STATUS_CODE_ERROR"`).
    StatusCode,
}

/// Describes the operations that should be applied to a column.
#[derive(Clone, Debug, PartialEq)]
pub enum ColumnTransformOp {
    /// Leave the column untouched.
    NoOp,

    /// Rename the column — schema-level change only, does not modify values.
    Rename(String),

    /// Cast the column to the specified Arrow [`DataType`] and rename it.
    CastAndRename(String, DataType),

    /// Drop the column entirely.
    Drop,

    /// Flatten a struct column into child columns.
    FlattenStructField(FlattenStructSpec),

    /// Coerce a single body string from the various typed body fields.
    CoerceBodyToString(CoerceStructStringSpec),

    /// Inline the attribute into the corresponding payload.
    InlineAttribute(ArrowPayloadType, AttributeRepresentation),

    /// Inline ListArray values containing 'child' records to the main signal batch (e.g. spanEvents).
    InlineChildLists(ArrowPayloadType),

    // Inline MapArray values containing 'child' records to the main signal batch (e.g. spanAttrs)
    InlineChildMap(ArrowPayloadType, String, String),

    /// Extract a single key from an inlined attribute column into a companion string column.
    ExtractMapValue {
        key: String,
        output_column: String,
        default_value: String,
    },

    /// Map integer enum values to their proto string names and rename the column.
    ///
    /// The input column is expected to be `Int32` (or `Dict(u8/u16, Int32)` per OTAP
    /// optimized encodings). Dictionary encoding is unpacked before conversion.
    EnumToString {
        output_name: String,
        mapper: EnumStringMapper,
    },
}

/// Describes which fields should be extracted, grouped by a parent id
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractGroupedFieldSpec {
    /// The name of the ID field to group by
    pub id_field_name: String,
    /// Mapping of desired field name -> output column name
    pub field_mapping: HashMap<String, String>,
}

/// Describes the operations that should be applied across multiple columns.
#[derive(Clone, Debug, PartialEq)]
pub enum MultiColumnTransformOp {
    /// Group attribute keys and values with the same ID into a single JSON string
    AttributesToJSONString,
    /// Group attribute keys and values with the same ID into a single map(string,string)
    AttributesToStringMap,
    /// Extract fields from one payload type, grouped by an id field (e.g. parent id).
    ExtractFields(ExtractGroupedFieldSpec),
}

/// Defines the structural and value transformations for every column in a RecordBatch.
#[derive(Clone, Debug, PartialEq)]
/// Represents a transformation plan for applying column-level and multi-column-level
/// operations to a dataset.
///
/// The `TransformationPlan` is designed to facilitate the application of structured
/// transformations to columnar data (e.g., OTAP Arrow tables). It maintains two main components:
/// - `column_operations`: Handles single-column operations such as renaming columns,
///   type casting, inlining attributes, and more.
/// - `multi_column_ops`: Handles operations involving multiple columns, such as
///   dropping all attributes or transforming attributes into other formats.
///
/// This struct is designed to support both detailed single-column transformations
/// and broader transformations that affect multiple columns collectively.
///
/// # Fields
/// - `column_operations`: Stores the per-column transformations. This encapsulates
///   all transformations applied to individual columns in the dataset.
/// - `multi_column_ops`: A list of transformations that apply to multiple columns,
///   such as flattening nested fields or converting attributes into JSON.
pub struct TransformationPlan {
    /// Encapsulates operations applied to individual columns, such as renaming,
    /// type casting, or re-indexing attributes.
    pub column_ops: ColumnOperations,

    /// Stores transformations that affect multiple columns. Examples include
    /// dropping all attributes, converting key-value pairs into JSON, and
    /// extracting grouped fields from nested attributes.
    pub multi_column_ops: Vec<MultiColumnTransformOp>,

    /// Boolean value indicating whether the mutated columns should be re-assembled into
    /// a new record batch (e.g. will be written to a clickhouse table).
    pub recreate_batch: bool,
}

impl TransformationPlan {
    /// Create a new Transformation Plan.
    pub fn new() -> Self {
        Self {
            column_ops: ColumnOperations::new(),
            multi_column_ops: Vec::new(),
            recreate_batch: false,
        }
    }
    /// Applies a flattening transformation to a specified struct column, breaking it into its child fields.
    fn apply_flattening(
        &mut self,
        column: &str,
        mapping: HashMap<String, String>,
        remove_col: bool,
    ) {
        self.column_ops.flatten_column(
            column,
            FlattenStructSpec {
                field_mapping: mapping,
                remove_struct_col: remove_col,
            },
        );
    }

    fn recreate_batch(&mut self) {
        self.recreate_batch = true;
    }

    /// Creates a `TransformationPlan` based on the provided payload type and configuration.
    ///
    /// The `from_config` method is a key entry point for generating a transformation plan tailored
    /// to a specific `ArrowPayloadType`. It applies logic based on the given payload type (e.g., `Logs`,
    /// `Spans`, etc.) and the configuration specified.
    ///
    /// This method initializes a plan and defines transformations applicable to each column according
    /// to the provided `payload_type` and `Config`. This includes flattening struct fields, reindexing,
    /// inlining attributes, renaming columns, and applying any other column-based or multi-column-based
    /// operations.
    ///
    /// # Arguments
    ///
    /// * `payload_type` - The type of payload for which the transformation plan is being created,
    ///   represented as an `ArrowPayloadType` enum. Examples include `ArrowPayloadType::Logs`,
    ///   `ArrowPayloadType::Spans`, etc.
    /// * `config` - The configuration specifying how various attributes such as `resource`,
    ///   `scope`, and `log` are to be handled. This typically includes information about storage,
    ///   type representation, and other parameters.
    ///
    /// # Returns
    ///
    /// Returns a fully constructed `TransformationPlan` that contains both column-level and
    /// multi-column-level transformations.
    pub fn from_config(payload_type: &ArrowPayloadType, config: &Config) -> Self {
        let mut tp = TransformationPlan::new();

        match payload_type {
            ArrowPayloadType::Logs => tp.configure_for_logs(config),
            ArrowPayloadType::Spans => tp.configure_for_spans(config),
            // Handling for these types includes conversion to inline ListArrays (e.g. each span row will get Links.SpanId: [])
            ArrowPayloadType::SpanLinks => tp.configure_for_span_links(),
            ArrowPayloadType::SpanEvents => tp.configure_for_span_events(),
            ArrowPayloadType::SpanLinkAttrs => tp.configure_for_inline_attributes(),
            ArrowPayloadType::SpanEventAttrs => tp.configure_for_inline_attributes(),
            // Attributes are converted to the desired format (json, map[string, string]) for insertion.
            ArrowPayloadType::ResourceAttrs => {
                tp.configure_for_attributes(&config.attributes.resource.representation)
            }
            ArrowPayloadType::ScopeAttrs => {
                tp.configure_for_attributes(&config.attributes.scope.representation)
            }
            ArrowPayloadType::LogAttrs => {
                tp.configure_for_attributes(&config.attributes.log.representation)
            }
            ArrowPayloadType::SpanAttrs => {
                tp.configure_for_attributes(&config.attributes.trace.representation)
            }
            // TODO: [support_new_signal] add payload names & config methods here
            _ => unimplemented!("Unsupported payload type: {:?}", payload_type),
        }

        tp
    }

    /// Handle transformation logic for attribute payload types
    fn configure_for_attributes(&mut self, repr: &AttributeRepresentation) {
        match repr {
            AttributeRepresentation::StringMap => self.attributes_to_string_map(),
            AttributeRepresentation::Json => self.attributes_to_json(),
        }
        self.clear_single_column_ops();
    }

    /// Handle transformation logic for Logs
    fn configure_for_logs(&mut self, config: &Config) {
        // log tables are always rebuilt / sent to clickhouse
        self.recreate_batch();
        self.apply_flattening(
            consts::RESOURCE,
            HashMap::from([
                (consts::ID.into(), ch_consts::RESOURCE_ID.into()),
                (
                    consts::SCHEMA_URL.into(),
                    ch_consts::CH_RESOURCE_SCHEMA_URL.into(),
                ),
            ]),
            true,
        );

        self.column_ops.inline_attributes(
            ch_consts::RESOURCE_ID,
            ArrowPayloadType::ResourceAttrs,
            config.attributes.resource.representation.clone(),
        );
        self.column_ops.extract_map_value(
            ch_consts::CH_RESOURCE_ATTRIBUTES,
            "service.name",
            ch_consts::CH_SERVICE_NAME,
            "",
        );

        self.apply_flattening(
            consts::SCOPE,
            HashMap::from([
                (consts::ID.into(), ch_consts::SCOPE_ID.into()),
                (
                    consts::SCHEMA_URL.into(),
                    ch_consts::CH_SCOPE_SCHEMA_URL.into(),
                ),
                (consts::NAME.into(), ch_consts::CH_SCOPE_NAME.into()),
                (consts::VERSION.into(), ch_consts::CH_SCOPE_VERSION.into()),
            ]),
            true,
        );

        self.column_ops.inline_attributes(
            ch_consts::SCOPE_ID,
            ArrowPayloadType::ScopeAttrs,
            config.attributes.scope.representation.clone(),
        );
        self.column_ops.inline_attributes(
            consts::ID,
            ArrowPayloadType::LogAttrs,
            config.attributes.log.representation.clone(),
        );
        self.column_ops
            .rename_column(consts::TIME_UNIX_NANO, ch_consts::CH_TIMESTAMP);
        self.column_ops
            .cast_string_and_rename(consts::TRACE_ID, ch_consts::CH_TRACE_ID);
        self.column_ops
            .cast_string_and_rename(consts::SPAN_ID, ch_consts::CH_SPAN_ID);
        self.column_ops
            .rename_column(OTAP_SPAN_FLAGS, ch_consts::CH_TRACE_FLAGS);
        self.column_ops
            .rename_column(consts::SEVERITY_TEXT, ch_consts::CH_SEVERITY_TEXT);
        self.column_ops.cast_and_rename(
            consts::SEVERITY_NUMBER,
            ch_consts::CH_SEVERITY_NUMBER,
            DataType::UInt8,
        );
        self.column_ops
            .rename_column(consts::EVENT_NAME, ch_consts::CH_EVENT_NAME);
        self.column_ops.coerce_body_values();

        // Drop any remaining ID fields if present.
        self.column_ops.add_op(
            ch_consts::RESOURCE_ID,
            ColumnTransformOp::Drop { },
        );
        self.column_ops.add_op(
            ch_consts::SCOPE_ID,
            ColumnTransformOp::Drop { },
        );
        self.column_ops.add_op(
            consts::ID,
            ColumnTransformOp::Drop { },
        );
    }

    /// Handle transformation logic for Spans
    fn configure_for_spans(&mut self, config: &Config) {
        // span tables are always rebuilt / sent to clickhouse
        self.recreate_batch();
        self.apply_flattening(
            consts::RESOURCE,
            HashMap::from([
                (consts::ID.into(), ch_consts::RESOURCE_ID.into()),
                (
                    consts::SCHEMA_URL.into(),
                    ch_consts::CH_RESOURCE_SCHEMA_URL.into(),
                ),
            ]),
            true,
        );

        self.column_ops.inline_attributes(
            ch_consts::RESOURCE_ID,
            ArrowPayloadType::ResourceAttrs,
            config.attributes.resource.representation.clone(),
        );
        self.apply_flattening(
            consts::SCOPE,
            HashMap::from([
                (consts::ID.into(), ch_consts::SCOPE_ID.into()),
                (consts::NAME.into(), ch_consts::CH_SCOPE_NAME.into()),
                (consts::VERSION.into(), ch_consts::CH_SCOPE_VERSION.into()),
            ]),
            true,
        );

        // Add the ListArray values returned by multi-column op "ExtractChildFields" inline in the main signal batch.
        // Must come before inline_attributes against the main ID column since it might be dropped depending on representation.
        self.column_ops
            .inline_child_rb_arrays(consts::ID, ArrowPayloadType::SpanEvents);
        self.column_ops
            .inline_child_rb_arrays(consts::ID, ArrowPayloadType::SpanLinks);
        // Add the MapArray vlaues for child record batches (attribute types) to the batch.
        self.column_ops.inline_child_rb_map(
            consts::ID,
            ArrowPayloadType::SpanEventAttrs,
            consts::ATTRIBUTES,
            ch_consts::CH_EVENTS_ATTRIBUTES,
        );
        self.column_ops.inline_child_rb_map(
            consts::ID,
            ArrowPayloadType::SpanLinkAttrs,
            consts::ATTRIBUTES,
            ch_consts::CH_LINKS_ATTRIBUTES,
        );

        self.column_ops.inline_attributes(
            consts::ID,
            ArrowPayloadType::SpanAttrs,
            config.attributes.trace.representation.clone(),
        );

        self.column_ops
            .rename_column(consts::START_TIME_UNIX_NANO, ch_consts::CH_TIMESTAMP);
        self.column_ops
            .cast_string_and_rename(consts::TRACE_ID, ch_consts::CH_TRACE_ID);
        self.column_ops
            .cast_string_and_rename(consts::SPAN_ID, ch_consts::CH_SPAN_ID);
        self.column_ops
            .cast_string_and_rename(consts::PARENT_SPAN_ID, ch_consts::CH_PARENT_SPAN_ID);
        self.column_ops
            .rename_column(consts::TRACE_STATE, ch_consts::CH_TRACE_STATE);
        self.column_ops
            .rename_column(consts::NAME, ch_consts::CH_SPAN_NAME);
        self.column_ops
            .span_kind_to_string(consts::KIND, ch_consts::CH_SPAN_KIND);
        self.column_ops.extract_map_value(
            ch_consts::CH_RESOURCE_ATTRIBUTES,
            "service.name",
            ch_consts::CH_SERVICE_NAME,
            "",
        );
        self.column_ops
            .cast_int_and_rename(consts::DURATION_TIME_UNIX_NANO, ch_consts::CH_DURATION);

        self.apply_flattening(
            consts::STATUS,
            HashMap::from([
                (consts::STATUS_CODE.into(), ch_consts::CH_STATUS_CODE.into()),
                (
                    consts::STATUS_MESSAGE.into(),
                    ch_consts::CH_STATUS_MESSAGE.into(),
                ),
            ]),
            true,
        );

        // The flatten above extracts status.code as an Int32 column named CH_STATUS_CODE.
        // Convert the integer values to their proto string names (e.g. 2 → "STATUS_CODE_ERROR").
        // This runs in the second pass of apply_column_ops since CH_STATUS_CODE is a
        // synthetic column created by the flatten rather than an original batch column.
        self.column_ops
            .status_code_to_string(ch_consts::CH_STATUS_CODE, ch_consts::CH_STATUS_CODE);

        // Drop any remaining ID fields if present.
        self.column_ops.add_op(
            ch_consts::RESOURCE_ID,
            ColumnTransformOp::Drop { },
        );
        self.column_ops.add_op(
            ch_consts::SCOPE_ID,
            ColumnTransformOp::Drop { },
        );
        self.column_ops.add_op(
            consts::ID,
            ColumnTransformOp::Drop { },
        );
    }

    fn configure_for_span_links(&mut self) {
        self.extract_grouped_fields(ExtractGroupedFieldSpec {
            id_field_name: consts::PARENT_ID.into(),
            field_mapping: HashMap::<String, String>::from([
                (consts::TRACE_ID.into(), ch_consts::CH_LINKS_TRACE_ID.into()),
                (consts::SPAN_ID.into(), ch_consts::CH_LINKS_SPAN_ID.into()),
                (
                    consts::TRACE_STATE.into(),
                    ch_consts::CH_LINKS_TRACE_STATE.into(),
                ),
            ]),
        });
    }
    fn configure_for_span_events(&mut self) {
        self.extract_grouped_fields(ExtractGroupedFieldSpec {
            id_field_name: consts::PARENT_ID.into(),
            field_mapping: HashMap::<String, String>::from([
                (
                    consts::TIME_UNIX_NANO.into(),
                    ch_consts::CH_EVENTS_TIMESTAMP.into(),
                ),
                (consts::NAME.into(), ch_consts::CH_EVENTS_NAME.into()),
            ]),
        });
    }
    // Common steps for processing attribute payloads that will always be inlined as a ListArray.
    fn configure_for_inline_attributes(&mut self) {
        // This flattens the attributes and removes the typed primitive columns.
        self.attributes_to_string_map();
        // Preserve the attributes column through single-column ops.
        self.column_ops.noop_column(consts::ATTRIBUTES);
    }

    /// Clears all single-column operations.
    pub fn clear_single_column_ops(&mut self) {
        self.column_ops.clear();
    }

    /// Transform attribute key-value pairs into JSON strings.
    pub fn attributes_to_json(&mut self) {
        self.multi_column_ops
            .push(MultiColumnTransformOp::AttributesToJSONString);
        self.column_ops.noop_column(consts::ATTRIBUTES);
    }

    /// Transform attribute key-value pairs into a `HashMap<String, String>`.
    pub fn attributes_to_string_map(&mut self) {
        self.multi_column_ops
            .push(MultiColumnTransformOp::AttributesToStringMap);
        self.column_ops.noop_column(consts::ATTRIBUTES);
    }

    /// Extract grouped fields from a payload.
    pub fn extract_grouped_fields(&mut self, ef: ExtractGroupedFieldSpec) {
        self.multi_column_ops
            .push(MultiColumnTransformOp::ExtractFields(ef));
    }
}

/// A struct to handle column-based transformation operations.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnOperations {
    pub column_ops: HashMap<String, Vec<ColumnTransformOp>>,
}

impl ColumnOperations {
    /// Create a new, empty `ColumnOperations`.
    pub fn new() -> Self {
        Self {
            column_ops: HashMap::new(),
        }
    }
    /// Add an operation to a specific column.
    pub fn add_op(&mut self, column: &str, op: ColumnTransformOp) {
        self.column_ops
            .entry(column.to_string())
            .or_default()
            .push(op);
    }

    /// Clear all column operations.
    pub fn clear(&mut self) {
        self.column_ops.clear();
    }

    /// Rename a column.
    pub fn rename_column(&mut self, old: &str, new: &str) {
        self.add_op(old, ColumnTransformOp::Rename(new.to_string()));
    }

    /// Cast a column to UInt64 and rename it.
    pub fn cast_int_and_rename(&mut self, old: &str, new: &str) {
        self.add_op(
            old,
            ColumnTransformOp::CastAndRename(new.to_string(), DataType::UInt64),
        );
    }

    /// Cast a column to a UTF-8 string and rename it.
    pub fn cast_string_and_rename(&mut self, old: &str, new: &str) {
        self.add_op(
            old,
            ColumnTransformOp::CastAndRename(new.to_string(), DataType::Utf8),
        );
    }

    /// Cast a column to the given Arrow [`DataType`] and rename it.
    pub fn cast_and_rename(&mut self, old: &str, new: &str, target: DataType) {
        self.add_op(
            old,
            ColumnTransformOp::CastAndRename(new.to_string(), target),
        );
    }

    /// Convert a `SpanKind` Int32 column to its proto string name and rename it.
    pub fn span_kind_to_string(&mut self, old: &str, new: &str) {
        self.add_op(
            old,
            ColumnTransformOp::EnumToString {
                output_name: new.to_string(),
                mapper: EnumStringMapper::SpanKind,
            },
        );
    }

    /// Convert a `StatusCode` Int32 column to its proto string name and rename it.
    pub fn status_code_to_string(&mut self, old: &str, new: &str) {
        self.add_op(
            old,
            ColumnTransformOp::EnumToString {
                output_name: new.to_string(),
                mapper: EnumStringMapper::StatusCode,
            },
        );
    }

    /// Mark a column as a no-op.
    pub fn noop_column(&mut self, column: &str) {
        self.add_op(column, ColumnTransformOp::NoOp);
    }

    /// Inline attributes for a column.
    pub fn inline_attributes(
        &mut self,
        column: &str,
        payload_type: ArrowPayloadType,
        representation: AttributeRepresentation,
    ) {
        self.add_op(
            column,
            ColumnTransformOp::InlineAttribute(payload_type, representation),
        );
    }

    /// Inline child record batch arrays for a column.
    pub fn inline_child_rb_arrays(&mut self, column: &str, payload_type: ArrowPayloadType) {
        self.add_op(column, ColumnTransformOp::InlineChildLists(payload_type));
    }

    /// Inline a map of child record batches into a column.
    pub fn inline_child_rb_map(
        &mut self,
        column: &str,
        payload_type: ArrowPayloadType,
        from_col: &str,
        to_col: &str,
    ) {
        self.add_op(
            column,
            ColumnTransformOp::InlineChildMap(
                payload_type,
                from_col.to_string(),
                to_col.to_string(),
            ),
        );
    }

    /// Extract a value from an inlined attribute column into a new string column.
    pub fn extract_map_value(
        &mut self,
        column: &str,
        key: &str,
        output_column: &str,
        default_value: &str,
    ) {
        self.add_op(
            column,
            ColumnTransformOp::ExtractMapValue {
                key: key.to_string(),
                output_column: output_column.to_string(),
                default_value: default_value.to_string(),
            },
        );
    }

    /// Flatten a struct column into its child columns.
    pub fn flatten_column(&mut self, column: &str, spec: FlattenStructSpec) {
        self.add_op(column, ColumnTransformOp::FlattenStructField(spec));
    }

    /// Coerce attribute values for a column to a string type.
    pub fn coerce_body_values(&mut self) {
        let spec = CoerceStructStringSpec {
            type_field: consts::ATTRIBUTE_TYPE.into(),
            output_column: ch_consts::CH_BODY.into(),
        };
        self.add_op(consts::BODY, ColumnTransformOp::CoerceBodyToString(spec));
    }
    /// Retrieve the list of operations for a given column.
    #[allow(dead_code)]
    pub fn get_ops(&self, column: &str) -> Option<&Vec<ColumnTransformOp>> {
        self.column_ops.get(column)
    }
}

#[cfg(test)]
mod tests {
    use crate::clickhouse_exporter::config::ConfigPatch;

    use super::*;

    #[test]
    fn test_column_operations_basic_usage() {
        let mut column_ops = ColumnOperations::new();

        // Test renaming a column
        column_ops.rename_column("old_name", "new_name");
        assert!(column_ops.get_ops("old_name").is_some());
        assert!(matches!(
            column_ops.get_ops("old_name").unwrap().first(),
            Some(ColumnTransformOp::Rename(new_name)) if new_name == "new_name"
        ));

        // Test clearing operations
        column_ops.clear();
        assert!(column_ops.get_ops("old_name").is_none());
    }

    #[test]
    fn test_apply_flattening() {
        let mut plan = TransformationPlan {
            column_ops: ColumnOperations::new(),
            multi_column_ops: Vec::new(),
            recreate_batch: false,
        };

        let field_mapping = HashMap::from([
            ("field1".to_string(), "flattened_field1".to_string()),
            ("field2".to_string(), "flattened_field2".to_string()),
        ]);
        plan.apply_flattening("struct_column", field_mapping.clone(), true);

        // Check that the flattening operation was applied to "struct_column"
        let ops = plan.column_ops.get_ops("struct_column").unwrap();
        assert_eq!(ops.len(), 1);
        assert!(matches!(
            ops[0],
            ColumnTransformOp::FlattenStructField(ref spec) if spec.field_mapping == field_mapping && spec.remove_struct_col
        ));
    }

    #[test]
    fn test_multi_column_operations() {
        let mut plan = TransformationPlan {
            column_ops: ColumnOperations::new(),
            multi_column_ops: Vec::new(),
            recreate_batch: false,
        };

        // Add multiple transformations
        plan.attributes_to_json();
        plan.attributes_to_string_map();

        // Check that multi-column transformations are added correctly
        assert_eq!(plan.multi_column_ops.len(), 2);
        assert!(matches!(
            plan.multi_column_ops[0],
            MultiColumnTransformOp::AttributesToJSONString
        ));
        assert!(matches!(
            plan.multi_column_ops[1],
            MultiColumnTransformOp::AttributesToStringMap
        ));
    }

    fn test_config() -> Config {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "user",
            "password": "pass"
        });
        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        Config::from_patch(patch)
    }

    #[test]
    fn test_logs_plan_has_trace_flags_and_scope_schema_url() {
        let plan = TransformationPlan::from_config(&ArrowPayloadType::Logs, &test_config());

        let scope_ops = plan
            .column_ops
            .get_ops(consts::SCOPE)
            .expect("scope column should have flatten ops");
        let scope_flatten = scope_ops
            .iter()
            .find_map(|op| match op {
                ColumnTransformOp::FlattenStructField(spec) => Some(spec),
                _ => None,
            })
            .expect("scope flatten spec should exist");
        assert_eq!(
            scope_flatten
                .field_mapping
                .get(consts::SCHEMA_URL)
                .map(String::as_str),
            Some(ch_consts::CH_SCOPE_SCHEMA_URL)
        );

        assert!(matches!(
            plan.column_ops
                .get_ops(OTAP_SPAN_FLAGS)
                .and_then(|ops| ops.first()),
            Some(ColumnTransformOp::Rename(new_name)) if new_name == ch_consts::CH_TRACE_FLAGS
        ));

        assert!(matches!(
            plan.column_ops
                .get_ops(consts::RESOURCE)
                .and_then(|ops| ops.first()),
            Some(ColumnTransformOp::FlattenStructField(spec))
                if spec.field_mapping.get(consts::SCHEMA_URL).map(String::as_str)
                    == Some(ch_consts::CH_RESOURCE_SCHEMA_URL)
        ));

        assert!(
            plan.column_ops
                .get_ops(ch_consts::CH_RESOURCE_ATTRIBUTES)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::ExtractMapValue { key, output_column, default_value }
                        if key == "service.name"
                            && output_column == ch_consts::CH_SERVICE_NAME
                            && default_value.is_empty()
                ))),
            "logs plan should extract service.name from resource attributes"
        );
    }

    #[test]
    fn test_spans_plan_does_not_inline_scope_attributes() {
        let plan = TransformationPlan::from_config(&ArrowPayloadType::Spans, &test_config());

        assert!(
            plan.column_ops.get_ops(ch_consts::SCOPE_ID).is_none(),
            "spans plan should not inline ScopeAttrs for traces"
        );

        assert!(
            plan.column_ops
                .get_ops(consts::ID)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::InlineChildLists(ArrowPayloadType::SpanEvents)
                ))),
            "spans plan should inline span events"
        );

        assert!(
            plan.column_ops
                .get_ops(consts::ID)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::InlineChildLists(ArrowPayloadType::SpanLinks)
                ))),
            "spans plan should inline span links"
        );

        assert!(
            plan.column_ops
                .get_ops(consts::ID)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::InlineChildMap(
                        ArrowPayloadType::SpanEventAttrs,
                        from,
                        to
                    ) if from == consts::ATTRIBUTES && to == ch_consts::CH_EVENTS_ATTRIBUTES
                ))),
            "spans plan should inline span event attributes"
        );

        assert!(
            plan.column_ops
                .get_ops(consts::ID)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::InlineChildMap(
                        ArrowPayloadType::SpanLinkAttrs,
                        from,
                        to
                    ) if from == consts::ATTRIBUTES && to == ch_consts::CH_LINKS_ATTRIBUTES
                ))),
            "spans plan should inline span link attributes"
        );

        assert!(
            plan.column_ops
                .get_ops(ch_consts::CH_RESOURCE_ATTRIBUTES)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::ExtractMapValue { key, output_column, default_value }
                        if key == "service.name"
                            && output_column == ch_consts::CH_SERVICE_NAME
                            && default_value.is_empty()
                ))),
            "spans plan should extract service.name from resource attributes"
        );

        assert!(matches!(
            plan.column_ops
                .get_ops(consts::DURATION_TIME_UNIX_NANO)
                .and_then(|ops| ops.first()),
            Some(ColumnTransformOp::CastAndRename(new_name, DataType::UInt64)) if new_name == ch_consts::CH_DURATION
        ));
    }

    #[test]
    fn test_spans_plan_converts_kind_to_string() {
        let plan = TransformationPlan::from_config(&ArrowPayloadType::Spans, &test_config());

        assert!(
            plan.column_ops
                .get_ops(consts::KIND)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::EnumToString {
                        output_name,
                        mapper: EnumStringMapper::SpanKind,
                    } if output_name == ch_consts::CH_SPAN_KIND
                ))),
            "spans plan should convert kind Int32 to SpanKind string"
        );
    }

    #[test]
    fn test_spans_plan_converts_status_code_to_string() {
        let plan = TransformationPlan::from_config(&ArrowPayloadType::Spans, &test_config());

        // status.code is extracted by flattening the status struct into CH_STATUS_CODE,
        // then a second-pass EnumToString op converts the Int32 to a proto string.
        assert!(
            plan.column_ops
                .get_ops(ch_consts::CH_STATUS_CODE)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::EnumToString {
                        output_name,
                        mapper: EnumStringMapper::StatusCode,
                    } if output_name == ch_consts::CH_STATUS_CODE
                ))),
            "spans plan should convert StatusCode Int32 to proto string name"
        );
    }

    #[test]
    fn test_logs_plan_casts_severity_number_to_uint8() {
        let plan = TransformationPlan::from_config(&ArrowPayloadType::Logs, &test_config());

        assert!(
            plan.column_ops
                .get_ops(consts::SEVERITY_NUMBER)
                .is_some_and(|ops| ops.iter().any(|op| matches!(
                    op,
                    ColumnTransformOp::CastAndRename(new_name, DataType::UInt8)
                        if new_name == ch_consts::CH_SEVERITY_NUMBER
                ))),
            "logs plan should cast severity_number from Int32 to UInt8"
        );
    }
}
