// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of expression evaluation for OTAP (OpenTelemetry Arrow Protocol) batches.
//!
//! # Expression Tree
//!
//! The expressions are planned an executed as a tree of various expression types. The input
//! for planning is an AST of expressions from the [`data_engine_expressions`]. The planning stage
//! converts this into a tree containing datafusion logical plans ([`Expr`]s). At runtime, these
//! logical plans are converted to datafusion physical expressions
//! ([`PhysicalExpr`s](datafusion::physical_expr::PhysicalExprRef)) during evaluation.
//!
//! There is an additional layer of abstraction in the expression tree containing these datafusion
//! logical/physical expressions. This is necessary because typically with typical datafusion
//! expression evaluation, there would be a single [`RecordBatch`] as input and a single expression
//! tree which produces the resulting [`ColumnValue`]. However, in the OTAP data-model, not all
//! data is in one [`RecordBatch`].
//!
//! For this reason, sections of the expression tree are grouped in higher level tree nodes,
//! each containing only the portion of the overall expression tree that can be executed on a
//! single "data scope". Each scope-specific node will evaluate its expression on the source data
//! and the results of these evaluations will be joined together as the expression evaluates.
//!
//! ## Data Scopes
//!
//! The "data scope" represents the source of the data for a given section of the expression tree.
//! It indicates both the source record batch, and the rows that will be selected.
//!
//! For example, consider evaluating an expression like `severity_number + attributes["x"]`. The
//! data scope of the left side of is the root record batch, and for the right side it's the
//! attributes record batch, filtered where `key=="x"`.
//!
//! Note, the data scope is _not_ simply an indicator of the arrow payload type for the record
//! batch. A given payload type can have multiple data scopes, for example `attributes["x"]` and
//! `attributes["y"]` would produce a different set of filtered rows from the same record batch.
//!
//! When evaluating a binary expression with inputs from different data scopes, the execution
//! will join the two inputs before executing the datafusion expression on the join result.
//!
//! *Current status* - for now this only supports a small set of binary arithmetic operations.

use std::sync::{Arc, LazyLock};

use arrow::array::{ArrayRef, RecordBatch};
use arrow::datatypes::{Field, Schema};
use datafusion::logical_expr::{ColumnarValue, Expr};
use datafusion::physical_expr::PhysicalExprRef;
use datafusion::scalar::ScalarValue;
use otap_df_config::SignalType;
use otap_df_pdata::schema::consts;
use otap_df_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::error::Result;
use crate::pipeline::planner::{AttributesIdentifier, ColumnAccessor};
use crate::pipeline::project::{Projection, ProjectionOptions};

mod bitmap;
pub(crate) mod eval;
pub(crate) mod join;
pub(crate) mod planner;
pub(crate) mod types;

pub(crate) const VALUE_COLUMN_NAME: &str = "value";
pub(crate) const LEFT_COLUMN_NAME: &str = "left";
pub(crate) const RIGHT_COLUMN_NAME: &str = "right";

/// Returns a column name for a multi-join argument at the given index.
///
/// Used when function arguments come from different data scopes and need to be joined
/// before the function can be evaluated. Each argument in the join result gets a column
/// named "arg_0", "arg_1", etc.
pub(crate) fn arg_column_name(index: usize) -> String {
    format!("arg_{index}")
}

/// Identifies which root-level parent struct column a [`DataScope::RootParent`] belongs to.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RootParentStruct {
    Resource,
    Scope,
}

/// Identifies OTAP data either consumed or produced by some expression.
///
/// OTAP batches contain multiple [`RecordBatch`]s, and within a given record batch, some expression
/// may indicate a different set of rows. This type is used to identify both the payload type of
/// the record batch, and which rows may have been selected from it.
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DataScope {
    /// Main telemetry batch (e.g., Logs with columns like severity_number, severity_text)
    Root,

    /// Attribute batch identified by [`AttributesIdentifier`] and filtered by some key.
    /// For example, (AttributesIdentifier::Root, "http.method") may refer to log attributes
    /// with key="http.method"
    Attribute(AttributesIdentifier, String),

    /// Raw (unfiltered) attribute batch identified by [`AttributesIdentifier`].
    ///
    /// Unlike [`Attribute`](Self::Attribute), this does NOT apply key filtering or value
    /// projection. The expression receives the full attributes `RecordBatch` as-is (with all
    /// columns: key, type, str, int, double, bool, bytes, ser, parent_id, id).
    ///
    /// Used when comparing attribute value for some key to a known type -- the planner builds
    /// a single expression like `col("key").eq(lit("x")).and(col("str").eq(lit("y")))` that
    /// operates directly on the raw attributes batch without an intermediate projection of the
    /// filtered attribute record batch.
    AttributesAll(AttributesIdentifier),

    /// A special data scope indicating the data is produced from a static scalar value defined
    /// in the input expression tree, rather than data from the OTAP batch.
    StaticScalar,

    /// A field read from a resource or scope struct column in the root record batch (e.g.,
    /// resource.schema_url or scope.name). Physically the data lives in the root batch (same
    /// as Root), but the parent struct records the hierarchy level for cardinality validation.
    RootParent(RootParentStruct),
}

impl DataScope {
    /// Determines if expressions for two scopes can be combined into a single expression without
    /// performing a join.
    ///
    /// Rules:
    /// - Any scope can combine with StaticScalar (constants)
    /// - Same scopes can combine (e.g., Root + Root), because the row order is the same.
    /// - Root and RootParent can combine because both live in the root record batch.
    /// - Two `AttributesAll` scopes with the same identifier can combine.
    pub(crate) fn can_combine(&self, other: &Self) -> bool {
        if self.is_scalar() || other.is_scalar() {
            return true;
        }
        let self_in_root = matches!(self, Self::Root | Self::RootParent(_));
        let other_in_root = matches!(other, Self::Root | Self::RootParent(_));
        (self_in_root && other_in_root) || self == other
    }

    /// Returns the [`AttributesIdentifier`] if this is an attribute-related scope
    /// (`Attribute` or `AttributesAll`).
    #[allow(dead_code)]
    pub(crate) fn attrs_id(&self) -> Option<&AttributesIdentifier> {
        match self {
            Self::Attribute(id, _) | Self::AttributesAll(id) => Some(id),
            _ => None,
        }
    }

    /// Returns true if this scope represents a static scalar value.
    pub fn is_scalar(&self) -> bool {
        *self == Self::StaticScalar
    }
}

impl From<&ColumnAccessor> for DataScope {
    fn from(value: &ColumnAccessor) -> Self {
        match value {
            ColumnAccessor::ColumnName(_) => Self::Root,
            ColumnAccessor::StructCol(struct_name, _) => match *struct_name {
                consts::RESOURCE => Self::RootParent(RootParentStruct::Resource),
                consts::SCOPE => Self::RootParent(RootParentStruct::Scope),
                _ => Self::Root,
            },
            ColumnAccessor::Attributes(attrs_id, attrs_key) => {
                Self::Attribute(*attrs_id, attrs_key.clone())
            }
        }
    }
}

/// An execution tree node for evaluating expressions on OTAP data.
///
/// Every node supports two execution methods:
///
/// - [`execute_as_value`](Self::execute_as_value) — produces a [`ScopedValue`] containing actual
///   values. Used when the consumer needs materialized data (assignment, arithmetic, function
///   arguments).
///
/// - [`execute_as_id_mask`](Self::execute_as_id_mask) — produces an
///   [`IdMask`](crate::pipeline::id_mask::IdMask) bitmap of matching IDs. Used when the consumer
///   only needs membership information (filtering, boolean combination).
///
/// The dual-mode design allows chains of boolean operations to stay in ID bitmap space without
/// materializing intermediate arrays.
pub(crate) enum ScopedExpr {
    /// Leaf: evaluate an expression on a specific data scope (RecordBatch).
    ///
    /// For `LeafEval::DatafusionExpr`, this reads from the scope's RecordBatch and evaluates a
    /// DataFusion expression. For `LeafEval::BatchPredicate`, the scope is conventionally
    /// `Root` since the result applies to root rows.
    Eval { scope: DataScope, eval: LeafEval },

    /// Join N children by materializing their results into a single RecordBatch (aligned
    /// by ID columns), then evaluate a DataFusion expression on the joined result.
    ///
    /// Used for cross-scope arithmetic, cross-scope comparisons, and multi-arg function
    /// calls where arguments come from different scopes.
    JoinAndEval {
        children: Vec<ScopedExpr>,
        eval: LeafEval,
    },

    /// Combine two boolean-producing children via IdMask bitmap intersection (AND).
    BitmapAnd(Box<ScopedExpr>, Box<ScopedExpr>),

    /// Combine two boolean-producing children via IdMask bitmap union (OR).
    BitmapOr(Box<ScopedExpr>, Box<ScopedExpr>),

    /// Negate a boolean-producing child via IdMask bitmap inversion (NOT).
    BitmapNot(Box<ScopedExpr>),
}
/// A column of values together with scope metadata describing where those values came from
/// and how they relate to other batches.
///
/// This is the result type for [`ScopedExpr::execute_as_value`].
#[derive(Debug)]
pub(crate) struct ScopedValue {
    /// The computed values, either a columnar array or a scalar.
    pub values: ColumnarValue,

    /// Which scope this data belongs to (identifies the source RecordBatch and, for child
    /// scopes, which key filter was applied).
    pub scope: DataScope,

    /// ID column from the source batch, used for joining or alignment with other scopes.
    pub ids: Option<ArrayRef>,

    /// Parent ID column from the source batch, used for joining child scopes back to their
    /// parent scope.
    pub parent_ids: Option<ArrayRef>,
}

impl ScopedValue {
    /// Create a new `ScopedValue` by extracting ID columns from the source `RecordBatch`.
    pub fn new(values: ColumnarValue, scope: DataScope, source_rb: &RecordBatch) -> Self {
        Self {
            values,
            scope,
            ids: source_rb.column_by_name(consts::ID).cloned(),
            parent_ids: source_rb.column_by_name(consts::PARENT_ID).cloned(),
        }
    }

    /// Create a `ScopedValue` representing a scalar result (e.g., from a static literal
    /// or a batch predicate).
    pub fn new_scalar(scalar_value: ScalarValue) -> Self {
        Self {
            values: ColumnarValue::Scalar(scalar_value),
            scope: DataScope::StaticScalar,
            ids: None,
            parent_ids: None,
        }
    }
}

/// Expression evaluation performed at the root of the `ScopedExpr` expression tree, e.g. where
/// the node type will be `Eval` or `JoinAndEval`.
pub(crate) enum LeafEval {
    /// A standard DataFusion expression, evaluated on the RecordBatch identified by the
    /// enclosing node's scope.
    ///
    /// For `Eval` nodes, the expression reads directly from the scope's RecordBatch.
    /// For `JoinAndEval` nodes, the expression reads from the joined RecordBatch produced
    /// by aligning the children's results.
    DatafusionExpr {
        /// The DataFusion logical expression.
        logical_expr: Expr,

        /// Lazily initialized physical expression, created on first execution when the
        /// actual Arrow schema is known.
        physical_expr: Option<PhysicalExprRef>,

        /// Column selection/reordering to match the physical expression's expected schema.
        projection: Projection,

        /// Options for projection (e.g., whether to remove dictionary encoding).
        projection_opts: ProjectionOptions,

        /// Whether to keep AnyValue columns as structs rather than splitting them into
        /// concrete typed columns. True when the expression is a simple column reference
        /// (e.g., `col("value")`).
        eval_anyval_as_struct: bool,

        /// Whether attribute key matching should be case-sensitive. Only relevant for
        /// `DataScope::Attributes` scopes. When `false`, attribute key filtering uses
        /// case-insensitive comparison. Defaults to `true`.
        attr_key_case_sensitive: bool,

        /// When true, absent data (missing columns, missing attribute keys) should be
        /// treated as "passes" rather than "fails". This is set to true for `is_null()`
        /// expressions, where a missing field means the field IS null — which is a match.
        missing_data_passes: bool,
    },

    /// A batch-level predicate that evaluates a property of the entire `OtapArrowRecords`
    /// batch rather than individual rows. The result is uniform across all rows (either all
    /// pass or all fail).
    ///
    /// Examples: signal type check ("is this batch Logs / Metrics / Traces?").
    BatchPredicate(Box<dyn BatchPredicate>),
}

impl LeafEval {
    /// Create a new `DatafusionExpr` leaf from a DataFusion logical expression.
    ///
    pub fn new_df_expr(logical_expr: Expr, requires_dict_downcast: bool) -> Result<Self> {
        Self::new_df_expr_with_key_case(logical_expr, requires_dict_downcast, true)
    }

    /// Create a new `DatafusionExpr` leaf that evaluates AnyValue columns as structs (no
    /// partitioning). Use when the expression already accesses a specific sub-field
    /// of an AnyValue column (e.g., `col("body").field("str")`).
    pub fn new_df_expr_anyval_as_struct(
        logical_expr: Expr,
        requires_dict_downcast: bool,
    ) -> Result<Self> {
        let projection = Projection::try_new(&logical_expr)?;

        Ok(Self::DatafusionExpr {
            logical_expr,
            physical_expr: None,
            projection,
            projection_opts: ProjectionOptions {
                downcast_dicts: requires_dict_downcast,
            },
            eval_anyval_as_struct: true,
            attr_key_case_sensitive: true,
            missing_data_passes: false,
        })
    }

    /// Create a new `DatafusionExpr` leaf with explicit attribute key case sensitivity.
    pub fn new_df_expr_with_key_case(
        logical_expr: Expr,
        requires_dict_downcast: bool,
        attr_key_case_sensitive: bool,
    ) -> Result<Self> {
        let eval_anyval_as_struct = matches!(logical_expr, Expr::Column(_));
        let missing_data_passes = matches!(logical_expr, Expr::IsNull(_));
        let projection = Projection::try_new(&logical_expr)?;

        Ok(Self::DatafusionExpr {
            logical_expr,
            physical_expr: None,
            projection,
            projection_opts: ProjectionOptions {
                downcast_dicts: requires_dict_downcast,
            },
            eval_anyval_as_struct,
            attr_key_case_sensitive,
            missing_data_passes,
        })
    }
}

/// A predicate that evaluates a property of the entire `OtapArrowRecords` batch.
///
/// Results are uniform across all rows: either all pass or all fail.
pub(crate) trait BatchPredicate: std::fmt::Debug {
    /// Evaluate the predicate against the entire batch.
    ///
    /// Returns true if the batch satisfies the predicate (all rows pass),
    /// false otherwise (all rows fail).
    fn evaluate(&self, otap_batch: &OtapArrowRecords) -> bool;
}

/// Checks whether the batch contains a specific signal type.
#[derive(Debug)]
pub(crate) struct SignalTypePredicate {
    pub signal_type: SignalType,
}

impl SignalTypePredicate {
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

impl BatchPredicate for SignalTypePredicate {
    fn evaluate(&self, otap_batch: &OtapArrowRecords) -> bool {
        otap_batch.signal_type() == self.signal_type
    }
}

/// To evaluate expressions that only produce scalar values, we need to pass some RecordBatch into
/// the call to PhysicalExpr::evaluate. We just pass a static empty record batch.
pub(crate) static SCALAR_RECORD_BATCH_INPUT: LazyLock<RecordBatch> =
    LazyLock::new(|| RecordBatch::new_empty(Arc::new(Schema::new(Vec::<Field>::new()))));

#[cfg(test)]
mod test {

    use arrow::array::{Array, BooleanArray};
    use arrow::buffer::BooleanBuffer;
    use datafusion::common::cast::as_boolean_array;
    use datafusion::logical_expr::{ColumnarValue, Expr, Operator, col, lit};
    use datafusion::scalar::ScalarValue;
    use otap_df_config::SignalType;
    use otap_df_pdata::otap::filter::IdBitmapPool;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otap_df_pdata::schema::consts;
    use otap_df_pdata::testing::round_trip::{otlp_to_otap, to_logs_data};

    use crate::pipeline::Pipeline;
    use crate::pipeline::expr::{DataScope, VALUE_COLUMN_NAME};
    use crate::pipeline::expr::{LeafEval, ScopedExpr, ScopedValue, SignalTypePredicate};
    use crate::pipeline::id_mask::IdMask;
    use crate::pipeline::planner::AttributesIdentifier;

    /// Helper: create an `Eval(DatafusionExpr)` node for a root-scoped expression.
    fn root_eval(expr: Expr) -> ScopedExpr {
        ScopedExpr::Eval {
            scope: DataScope::Root,
            eval: LeafEval::new_df_expr(expr, false).unwrap(),
        }
    }

    /// Helper: create an `Eval(DatafusionExpr)` node for an attribute-scoped expression.
    fn attrs_eval(attrs_id: AttributesIdentifier, key: &str, expr: Expr) -> ScopedExpr {
        ScopedExpr::Eval {
            scope: DataScope::Attribute(attrs_id, key.to_string()),
            eval: LeafEval::new_df_expr(expr, false).unwrap(),
        }
    }

    /// Helper: create an `Eval(DatafusionExpr)` node for an attribute-scoped expression
    /// with dictionary downcast enabled.
    fn attrs_eval_dict_downcast(
        attrs_id: AttributesIdentifier,
        key: &str,
        expr: Expr,
    ) -> ScopedExpr {
        ScopedExpr::Eval {
            scope: DataScope::Attribute(attrs_id, key.to_string()),
            eval: LeafEval::new_df_expr(expr, true).unwrap(),
        }
    }

    /// Helper: create a `Eval(BatchPredicate)` node for a signal type check.
    fn signal_type_eval(signal_type: SignalType) -> ScopedExpr {
        ScopedExpr::Eval {
            scope: DataScope::Root,
            eval: LeafEval::BatchPredicate(Box::new(SignalTypePredicate::new(signal_type))),
        }
    }

    /// Helper: create test log data with severity and attributes.
    fn test_logs_data() -> otap_df_pdata::OtapArrowRecords {
        let logs = to_logs_data(vec![
            LogRecord::build()
                .severity_text("WARN")
                .severity_number(13)
                .attributes(vec![
                    KeyValue::new("code.namespace", AnyValue::new_string("main")),
                    KeyValue::new("code.line.number", AnyValue::new_int(42)),
                ])
                .event_name("e1")
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .severity_number(17)
                .attributes(vec![
                    KeyValue::new("code.namespace", AnyValue::new_string("test")),
                    KeyValue::new("code.line.number", AnyValue::new_int(100)),
                ])
                .event_name("e2")
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .severity_number(13)
                .attributes(vec![
                    KeyValue::new("code.namespace", AnyValue::new_string("main")),
                    KeyValue::new("code.line.number", AnyValue::new_int(7)),
                ])
                .event_name("e3")
                .finish(),
        ]);
        otlp_to_otap(&OtlpProtoMessage::Logs(logs))
    }

    /// Helper: extract a boolean array from a ScopedValue.
    fn as_bool_arr(sv: &ScopedValue) -> BooleanArray {
        match &sv.values {
            ColumnarValue::Array(arr) => as_boolean_array(arr)
                .cloned()
                .expect("expected boolean array"),
            ColumnarValue::Scalar(ScalarValue::Boolean(Some(b))) => BooleanArray::new(
                if *b {
                    BooleanBuffer::new_set(1)
                } else {
                    BooleanBuffer::new_unset(1)
                },
                None,
            ),
            other => panic!("expected boolean result, got {other:?}"),
        }
    }

    #[test]
    fn test_root_field_eval_as_value() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();

        // severity_number > 14
        let mut op = root_eval(col(consts::SEVERITY_NUMBER).gt(lit(14i32)));

        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        assert_eq!(result.scope, DataScope::Root);

        let bool_arr = as_bool_arr(&result);
        // severity_numbers are [13, 17, 13], so only index 1 passes
        assert!(!bool_arr.value(0));
        assert!(bool_arr.value(1));
        assert!(!bool_arr.value(2));
    }

    #[test]
    fn test_root_field_eval_as_id_mask() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        // severity_number > 14
        let mut op = root_eval(col(consts::SEVERITY_NUMBER).gt(lit(14i32)));

        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        // exactly one row (index 1) passes
        match &mask {
            IdMask::Some(bitmap) => {
                // the root batch should have id values [0, 1, 2]
                // only id=1 (severity_number=17) passes
                assert!(bitmap.contains(1));
                assert!(!bitmap.contains(0));
                assert!(!bitmap.contains(2));
            }
            other => panic!("expected IdMask::Some, got {other:?}"),
        }
    }

    #[test]
    fn test_attribute_eval_as_value() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();

        // attributes["code.namespace"] (returns the AnyValue struct as value column)
        let mut op = attrs_eval(AttributesIdentifier::Root, "code.namespace", col("value"));

        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        assert_eq!(
            result.scope,
            DataScope::Attribute(AttributesIdentifier::Root, "code.namespace".to_string())
        );

        // should have 3 rows (one per log record)
        match &result.values {
            ColumnarValue::Array(arr) => assert_eq!(arr.len(), 3),
            other => panic!("expected array, got {other:?}"),
        }

        // should have parent_ids
        assert!(result.parent_ids.is_some());
    }

    #[test]
    fn test_batch_predicate_logs_true() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        let mut op = signal_type_eval(SignalType::Logs);

        // execute_as_value: should return true scalar
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        match &result.values {
            ColumnarValue::Scalar(ScalarValue::Boolean(Some(true))) => {}
            other => panic!("expected true scalar, got {other:?}"),
        }

        // execute_as_id_mask: should return IdMask::All
        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();
        assert_eq!(mask, IdMask::All);
    }

    #[test]
    fn test_batch_predicate_wrong_type() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        // checking for Traces on a Logs batch → false
        let mut op = signal_type_eval(SignalType::Traces);

        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        match &result.values {
            ColumnarValue::Scalar(ScalarValue::Boolean(Some(false))) => {}
            other => panic!("expected false scalar, got {other:?}"),
        }

        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();
        assert_eq!(mask, IdMask::None);
    }

    #[test]
    fn test_join_and_eval_cross_scope() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();

        // severity_number > attributes["code.line.number"]
        // This requires joining root (severity_number) with attributes (code.line.number value).
        //
        // Data:
        //   row 0: severity_number=13, code.line.number=42  → 13 > 42 → false
        //   row 1: severity_number=17, code.line.number=100 → 17 > 100 → false
        //   row 2: severity_number=13, code.line.number=7   → 13 > 7 → true
        //
        // Note: severity_number is Int32 on the root batch, but the AnyValue int column is
        // Int64. We cast the left side to Int64 to match.

        use crate::pipeline::expr::{LEFT_COLUMN_NAME, RIGHT_COLUMN_NAME};
        use datafusion::logical_expr::cast;

        // Left child: severity_number cast to Int64
        let left_child = ScopedExpr::Eval {
            scope: DataScope::Root,
            eval: LeafEval::new_df_expr(
                cast(
                    col(consts::SEVERITY_NUMBER),
                    arrow::datatypes::DataType::Int64,
                ),
                true,
            )
            .unwrap(),
        };

        // Right child: the int sub-column of the attributes value (already Int64)
        let right_child = attrs_eval_dict_downcast(
            AttributesIdentifier::Root,
            "code.line.number",
            col(VALUE_COLUMN_NAME),
        );

        let mut op = ScopedExpr::JoinAndEval {
            children: vec![left_child, right_child],
            eval: LeafEval::new_df_expr(
                Expr::BinaryExpr(datafusion::logical_expr::BinaryExpr::new(
                    Box::new(col(LEFT_COLUMN_NAME)),
                    Operator::Gt,
                    Box::new(col(RIGHT_COLUMN_NAME)),
                )),
                true,
            )
            .unwrap(),
        };

        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();

        let bool_arr = as_bool_arr(&result);
        assert_eq!(bool_arr.len(), 3);
        assert!(!bool_arr.value(0));
        assert!(!bool_arr.value(1));
        assert!(bool_arr.value(2));
    }

    #[test]
    fn test_bitmap_and_root_and_attribute() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        // severity_text == "WARN" AND attributes["code.namespace"] == "main"
        //
        // Data:
        //   row 0: severity_text="WARN", code.namespace="main" → true AND true → true
        //   row 1: severity_text="ERROR", code.namespace="test" → false AND false → false
        //   row 2: severity_text="WARN", code.namespace="main" → true AND true → true
        //
        // Note: attribute str values are dictionary-encoded, so we need dict downcast enabled.

        let left = root_eval(col(consts::SEVERITY_TEXT).eq(lit("WARN")));
        let right = attrs_eval_dict_downcast(
            AttributesIdentifier::Root,
            "code.namespace",
            col(VALUE_COLUMN_NAME).eq(lit("main")),
        );

        let mut op = ScopedExpr::BitmapAnd(Box::new(left), Box::new(right));

        // test execute_as_id_mask
        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0));
                assert!(!bitmap.contains(1));
                assert!(bitmap.contains(2));
            }
            other => panic!("expected IdMask::Some, got {other:?}"),
        }

        // test execute_as_value
        let mut op2 = ScopedExpr::BitmapAnd(
            Box::new(root_eval(col(consts::SEVERITY_TEXT).eq(lit("WARN")))),
            Box::new(attrs_eval_dict_downcast(
                AttributesIdentifier::Root,
                "code.namespace",
                col(VALUE_COLUMN_NAME).eq(lit("main")),
            )),
        );

        let result = op2.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        assert_eq!(result.scope, DataScope::Root);
        let bool_arr = as_bool_arr(&result);
        assert_eq!(bool_arr.len(), 3);
        assert!(bool_arr.value(0));
        assert!(!bool_arr.value(1));
        assert!(bool_arr.value(2));
    }

    #[test]
    fn test_bitmap_or_with_missing_data() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        // attributes["nonexistent"] == "x" OR severity_text == "WARN"
        //
        // Left side: "nonexistent" attribute doesn't exist → IdMask::None
        // Right side: severity_text == "WARN" → rows 0 and 2 pass
        // OR result: should be rows 0 and 2

        let left = attrs_eval(
            AttributesIdentifier::Root,
            "nonexistent",
            col(VALUE_COLUMN_NAME).eq(lit("x")),
        );
        let right = root_eval(col(consts::SEVERITY_TEXT).eq(lit("WARN")));

        let mut op = ScopedExpr::BitmapOr(Box::new(left), Box::new(right));

        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0));
                assert!(!bitmap.contains(1));
                assert!(bitmap.contains(2));
            }
            other => panic!("expected IdMask::Some, got {other:?}"),
        }
    }

    #[test]
    fn test_bitmap_not() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        // NOT(severity_text == "WARN")
        //
        // severity_text == "WARN" matches rows 0 and 2
        // NOT of that should match only row 1

        let inner = root_eval(col(consts::SEVERITY_TEXT).eq(lit("WARN")));
        let mut op = ScopedExpr::BitmapNot(Box::new(inner));

        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &mask {
            IdMask::NotSome(bitmap) => {
                // NotSome means "everything except what's in the bitmap"
                // The inner produced Some({0, 2}), so NOT is NotSome({0, 2})
                assert!(bitmap.contains(0));
                assert!(!bitmap.contains(1));
                assert!(bitmap.contains(2));
            }
            other => panic!("expected IdMask::NotSome, got {other:?}"),
        }

        // Also verify via execute_as_value
        let mut op2 = ScopedExpr::BitmapNot(Box::new(root_eval(
            col(consts::SEVERITY_TEXT).eq(lit("WARN")),
        )));
        let result = op2.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        let bool_arr = as_bool_arr(&result);
        assert_eq!(bool_arr.len(), 3);
        assert!(!bool_arr.value(0));
        assert!(bool_arr.value(1));
        assert!(!bool_arr.value(2));
    }

    #[test]
    fn test_compound_bitmap_and_chain() {
        let otap = test_logs_data();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();

        // attributes["code.namespace"] == "main"
        //   AND attributes["code.line.number"] == 42  (int comparison)
        //   AND severity_text == "WARN"
        //
        // Data:
        //   row 0: namespace="main", line=42, severity="WARN" → true
        //   row 1: namespace="test", line=100, severity="ERROR" → false
        //   row 2: namespace="main", line=7, severity="WARN" → false (line != 42)

        // Note: attribute str values are dictionary-encoded, so we need dict downcast.
        // Attribute int values are stored in the "int" column (Int64).
        let attr_namespace = attrs_eval_dict_downcast(
            AttributesIdentifier::Root,
            "code.namespace",
            col(VALUE_COLUMN_NAME).eq(lit("main")),
        );
        let attr_line = attrs_eval_dict_downcast(
            AttributesIdentifier::Root,
            "code.line.number",
            col(VALUE_COLUMN_NAME).eq(lit(42i64)),
        );
        let severity = root_eval(col(consts::SEVERITY_TEXT).eq(lit("WARN")));

        // Build: (namespace == "main" AND line == 42) AND severity == "WARN"
        let inner_and = ScopedExpr::BitmapAnd(Box::new(attr_namespace), Box::new(attr_line));
        let mut op = ScopedExpr::BitmapAnd(Box::new(inner_and), Box::new(severity));

        let mask = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 should match");
                assert!(!bitmap.contains(1), "row 1 should not match");
                assert!(!bitmap.contains(2), "row 2 should not match (line != 42)");
            }
            other => panic!("expected IdMask::Some, got {other:?}"),
        }
    }
}
