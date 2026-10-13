// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projection utilities for extracting required columns from RecordBatches

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, NullArray, RecordBatch, RecordBatchOptions, StructArray};
use arrow::compute::cast;

use arrow::datatypes::{DataType, Field, FieldRef, Schema};
use arrow::error::ArrowError;
use datafusion::common::tree_node::{TreeNode, TreeNodeRecursion, TreeNodeVisitor};
use datafusion::common::{HashMap, HashSet};
use datafusion::error::DataFusionError;
use datafusion::functions::core::getfield::GetFieldFunc;
use datafusion::logical_expr::Expr;
use datafusion::scalar::ScalarValue;
use otel_arrow_dfe_pdata::arrays::sanitize::sanitize_column;

use otel_arrow_dfe_pdata::schema::consts;

use crate::error::Result;

pub mod anyval;

#[derive(Debug, Default)]
pub struct ProjectionOptions {
    /// Whether or not to downcast dictionary arrays to the native type. Some types of expressions,
    /// arithmetic operations for example, do not work on dictionary encoded columns.
    pub downcast_dicts: bool,

    /// Whether or not to sanitize dictionaries. Often we may filter a record batch containing
    /// dictionary columns and then evaluate an expression on the result. The filtering operation
    /// may leave orphaned keys in dictionary column's values array. If there's some kind of
    /// operation that operates directly on dictionary values indiscriminantly of whether they are
    /// orphaned and it may fail for the orphaned keys in particular, set this option to remove
    /// sanitize the columns before evaluation
    pub sanitize_dicts: bool,

    /// Whether to create null placeholders when some column does not exist. A column not being
    /// present means it is optional in the OTAP model, or it represents the value of an attribute
    /// that is not present. Ordinarily, when any column is not present the projection return
    /// `None` and lets the caller handle it, however this option can be used in cases where the
    /// caller is feeding the resulting projected batch into an expression that has some specific
    /// handling for null columns
    pub default_null_columns: bool,
}

/// Projection helper that can project a RecordBatch to only the columns needed by an expression
#[derive(Debug)]
pub struct Projection {
    schema: ProjectedSchema,
}

impl From<Vec<String>> for Projection {
    fn from(columns: Vec<String>) -> Self {
        Self {
            schema: columns
                .into_iter()
                .map(ProjectedSchemaColumn::Root)
                .collect(),
        }
    }
}

impl Projection {
    /// Attempt to create a new instance of [`FilterProjection`]. It will return an error if
    /// there is some form of [`Expr`] tree which is not recognized
    pub(crate) fn try_new(logical_expr: &Expr) -> Result<Self> {
        let mut visitor = ProjectedSchemaExprVisitor::default();
        _ = logical_expr.visit(&mut visitor)?;

        Ok(Self {
            schema: visitor.into(),
        })
    }

    pub(crate) fn references_column(&self, col_name: &str) -> bool {
        self.schema.iter().any(|col| {
            if let ProjectedSchemaColumn::Root(schema_col_name) = col {
                schema_col_name.as_str() == col_name
            } else {
                false
            }
        })
    }

    /// Apply this projection to a record batch containing OTAP attributes.
    ///
    /// This is most appropriately invoked during evaluation of a nested pipeline applied to
    /// attributes such as in the OPL query `logs | apply attributes { where <expr> }`. In this
    /// case, we'd evaluate the expression (which would be planned as a datafusion expression) on
    /// the attributes record batch. This evaluation requires special treatment because there is a
    /// virtual "value" column that can be used in expressions, which must be replaced with the
    /// actual column that contains the attribute values.
    ///
    pub fn project_attrs_record_batch(
        &self,
        attrs_record_batch: &RecordBatch,
        options: &ProjectionOptions,
    ) -> Result<Option<RecordBatch>> {
        let mut projection_cols = ProjectionColumns::from(attrs_record_batch);

        let empty_projection = self.schema.is_empty();
        let keys_only = self.schema.len() == 1 && self.references_column(consts::ATTRIBUTE_KEY);
        if !empty_projection && !keys_only {
            self.ensure_attrs_value_column_present(&mut projection_cols)?;
        }

        self.project_with_options(projection_cols, options)
    }

    pub fn project_with_options<T: Into<ProjectionColumns>>(
        &self,
        input: T,
        options: &ProjectionOptions,
    ) -> Result<Option<RecordBatch>> {
        let mut projection_cols = input.into();
        if !self.apply_to_columns(&mut projection_cols, options.default_null_columns) {
            return Ok(None);
        }

        if options.downcast_dicts {
            Self::try_downcast_dicts(&mut projection_cols.fields, &mut projection_cols.columns)?;
        }

        if options.sanitize_dicts {
            Self::try_sanitize_columns(&mut projection_cols.columns)?;
        }

        Ok(Some(projection_cols.try_into()?))
    }

    /// Apply the projection, keeping only the columns specified by this projection, and having
    /// them organized in the specific column order.
    ///
    /// This returns `true` if the projection was successfully applied. Otherwise it returns
    /// `false` indicating that some desired column was not present.
    fn apply_to_columns(
        &self,
        projection_cols: &mut ProjectionColumns,
        default_nulls: bool,
    ) -> bool {
        for (expected_idx, projected_col) in self.schema.iter().enumerate() {
            match projected_col {
                ProjectedSchemaColumn::Root(desired_col_name) => {
                    if let Some((found_idx, _)) = projection_cols.find(desired_col_name) {
                        projection_cols.swap(found_idx, expected_idx);
                    } else if default_nulls {
                        // default nulls
                        let last_idx = projection_cols.fields.len();
                        projection_cols.append_column(
                            Field::new(desired_col_name, DataType::Null, true),
                            Arc::new(NullArray::new(projection_cols.num_rows())),
                        );
                        projection_cols.swap(last_idx, expected_idx);
                    } else {
                        return false;
                    };
                }
                ProjectedSchemaColumn::Struct(desired_struct_name, desired_struct_fields) => {
                    let struct_index = projection_cols.find(desired_struct_name).map(|(i, _)| i);
                    if struct_index.is_none() && !default_nulls {
                        return false;
                    }

                    let struct_col = struct_index
                        .and_then(|i| projection_cols.columns.get(i))
                        .and_then(|col| col.as_any().downcast_ref::<StructArray>());

                    let mut struct_fields = Vec::new();
                    let mut struct_field_defs = Vec::new();

                    for field_name in desired_struct_fields {
                        let Some((struct_field, field_def)) = struct_col
                            .and_then(|struct_col| struct_col.fields().find(field_name))
                            .map(|(field_index, field)| {
                                (
                                    // safety: we know struct_col is not none here because
                                    // otherwise this callback would not be invoked
                                    struct_col.expect("not none").column(field_index).clone(),
                                    field.clone(),
                                )
                            })
                            .or(if default_nulls {
                                Some((
                                    Arc::new(NullArray::new(projection_cols.num_rows()))
                                        as ArrayRef,
                                    Arc::new(Field::new(field_name, DataType::Null, true)),
                                ))
                            } else {
                                None
                            })
                        else {
                            return false;
                        };

                        struct_fields.push(struct_field);
                        struct_field_defs.push(field_def)
                    }

                    // safety: `try_new` will return an error here if the types of arrays we pass
                    // for the fields do not match the field definitions, or if the arrays have
                    // different lengths. Based on the way we've constructed inputs, this should
                    // not happen because we've taken them from the input struct column in order
                    let projected_struct_arr = Arc::new(
                        StructArray::try_new(
                            struct_field_defs.into(),
                            struct_fields,
                            struct_col.map(|arr| arr.nulls().cloned()).unwrap_or(None),
                        )
                        .expect("can init StructArray"),
                    );

                    if let Some(struct_index) = struct_index {
                        projection_cols.replace_column_at_index(struct_index, projected_struct_arr);
                        projection_cols.swap(struct_index, expected_idx);
                    } else {
                        let last_idx = projection_cols.fields.len();
                        projection_cols.append_column(
                            Field::new(
                                desired_struct_name,
                                projected_struct_arr.data_type().clone(),
                                true,
                            ),
                            projected_struct_arr,
                        );
                        projection_cols.swap(last_idx, expected_idx);
                    };
                }
            }
        }

        projection_cols.fields.truncate(self.schema.len());
        projection_cols.columns.truncate(self.schema.len());

        true
    }

    pub fn try_downcast_dicts(fields: &mut [Arc<Field>], columns: &mut [ArrayRef]) -> Result<()> {
        for i in 0..fields.len() {
            let field = &fields[i];
            if let DataType::Dictionary(_, v) = field.data_type() {
                let new_field = Arc::new(field.as_ref().clone().with_data_type(v.as_ref().clone()));
                let new_column = cast(&columns[i], v.as_ref())?;
                fields[i] = new_field;
                columns[i] = new_column;
            }
        }

        Ok(())
    }

    fn try_sanitize_columns(columns: &mut [ArrayRef]) -> Result<()> {
        for column in columns.iter_mut() {
            if let Some(new_column) = sanitize_column(column.as_ref()) {
                *column = new_column;
            }
        }

        Ok(())
    }
}

/// Columns and fields that will be operated by [`Projection`].
///
/// This provides a wrapper around mutable vecs of Field and ArrayRef with a variety of helper
/// methods for organizing projected columns.
pub(crate) struct ProjectionColumns {
    num_rows: usize,
    fields: Vec<FieldRef>,
    columns: Vec<ArrayRef>,
}

impl From<&RecordBatch> for ProjectionColumns {
    fn from(rb: &RecordBatch) -> Self {
        let num_rows = rb.num_rows();
        let fields = rb.schema_ref().fields().to_vec();
        let columns = rb.columns().to_vec();
        Self {
            fields,
            columns,
            num_rows,
        }
    }
}

impl TryFrom<ProjectionColumns> for RecordBatch {
    type Error = ArrowError;

    fn try_from(columns: ProjectionColumns) -> std::result::Result<Self, Self::Error> {
        RecordBatch::try_new_with_options(
            Arc::new(Schema::new(columns.fields)),
            columns.columns,
            &RecordBatchOptions::new().with_row_count(Some(columns.num_rows)),
        )
    }
}

impl ProjectionColumns {
    pub(crate) fn columns(&self) -> &[ArrayRef] {
        &self.columns
    }

    /// append a new column
    pub(crate) fn append_column(&mut self, field: Field, column: ArrayRef) {
        self.fields.push(FieldRef::new(field));
        self.columns.push(column);
    }

    /// swap the location of two columns. If either index is out of bounds, this does nothing.
    pub(crate) fn swap(&mut self, pos_1: usize, pos_2: usize) {
        if pos_1 < self.fields.len() && pos_2 < self.fields.len() {
            self.fields.swap(pos_1, pos_2);
            self.columns.swap(pos_1, pos_2);
        }
    }

    /// find the index and definition of a column/field by name. returns None if no field exists
    /// with the passed name.
    pub(crate) fn find(&self, column_name: &str) -> Option<(usize, &FieldRef)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, b)| b.name() == column_name)
    }

    /// replace the column at a given index with the passed array. The field at this index will
    /// also have its datatype updated. If the index is out of bounds, this does nothing.
    fn replace_column_at_index(&mut self, index: usize, new_column: ArrayRef) {
        if let Some(field) = self.fields.get(index) {
            let new_field = field
                .as_ref()
                .clone()
                .with_data_type(new_column.data_type().clone());
            self.fields[index] = FieldRef::new(new_field);
            self.columns[index] = new_column;
        }
    }

    /// change the name of a column at the given index. noop if index is out of bounds.
    pub(crate) fn rename_column_at_index(&mut self, index: usize, new_column_name: &str) {
        if let Some(field) = self.fields.get(index) {
            let new_field = field.as_ref().clone().with_name(new_column_name);
            self.fields[index] = FieldRef::new(new_field);
        }
    }

    fn num_rows(&self) -> usize {
        self.num_rows
    }
}

/// Defines that the record batch should be projected as when the filter is applied.
///
/// Note that the only thing that matters when applying the filter's `PhysicalExpr` is that the
/// columns are all present and in the correct order, which is why this is implemented as a lists
/// of column names without regard to types.
type ProjectedSchema = Vec<ProjectedSchemaColumn>;

/// Definition of column in the projected schema
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd)]
pub(crate) enum ProjectedSchemaColumn {
    /// Simply column in the [`RecordBatch`] being filtered that should be in the projected schema
    Root(String),

    /// Columns that should be projected from a nested struct. For example on a Logs record batch
    /// this could be things like `resource.name`, or `body.str`.
    Struct(String, Vec<String>),
}

/// Implementation of [`TreeNodeVisitor`] that will visit the [`Expr`] defining the filter
/// predicate to determine which columns are referenced in the filter predicate. This information
/// can then be used to determine how to project the input batches before evaluating the filter's
/// [`PhysicalExpr`]
#[derive(Debug, Default)]
struct ProjectedSchemaExprVisitor {
    root_columns: HashSet<String>,

    // this is used to keep track of fields in some nested struct which are referenced by the expr.
    // the map is keyed by struct name, and the set contains the fields within the struct.
    struct_columns: HashMap<String, HashSet<String>>,
}

impl<'a> TreeNodeVisitor<'a> for ProjectedSchemaExprVisitor {
    type Node = Expr;

    fn f_down(&mut self, node: &'a Self::Node) -> datafusion::error::Result<TreeNodeRecursion> {
        if let Expr::Column(col) = node {
            _ = self.root_columns.insert(col.name.clone());
        }

        // here we're checking if the expression we're visiting references a field within a struct
        // column. The way we reference these in the plans we build is using an expression like
        // `col("scope").field("name")` which produces a ScalarFunction expression invoking the
        // `GetFieldFunc` function with arguments ("scope", "name").
        if let Expr::ScalarFunction(scalar_udf) = node
            && scalar_udf.func.as_ref().inner().is::<GetFieldFunc>()
        {
            let source = scalar_udf.args.first();
            let field = scalar_udf.args.get(1);
            match (source, field) {
                (
                    Some(Expr::Column(col)),
                    Some(Expr::Literal(ScalarValue::Utf8(Some(nested_col)), _)),
                ) => {
                    let struct_fields = self
                        .struct_columns
                        .entry(col.name.clone())
                        .or_insert(HashSet::new());
                    _ = struct_fields.insert(nested_col.clone());

                    // don't continue as we've found a column. Otherwise this will continue
                    // down the expression tree and we'll visit the Column expression twice.
                    return Ok(TreeNodeRecursion::Jump);
                }
                unexpected_args => {
                    let err_msg = format!(
                        "Found unexpected arguments to `GetFieldFunc`. Expected (Col, Literal(Utf8)) found {:?}",
                        unexpected_args
                    );
                    return Err(DataFusionError::Plan(err_msg));
                }
            }
        }

        Ok(TreeNodeRecursion::Continue)
    }
}

impl From<ProjectedSchemaExprVisitor> for ProjectedSchema {
    fn from(visitor: ProjectedSchemaExprVisitor) -> Self {
        let num_cols = visitor.root_columns.len()
            + visitor
                .struct_columns
                .values()
                .map(|cols| cols.len())
                .sum::<usize>();
        let mut schema = Vec::with_capacity(num_cols);

        for col in visitor.root_columns {
            schema.push(ProjectedSchemaColumn::Root(col))
        }

        for (struct_name, cols) in visitor.struct_columns {
            schema.push(ProjectedSchemaColumn::Struct(
                struct_name,
                cols.into_iter().collect(),
            ));
        }

        schema
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{Float64Array, Int64Array, StringArray};

    /// Scenario: projection references columns in a different order than they appear in the
    /// input batch.
    /// Guarantees: result has columns in the projection's declared order.
    #[test]
    fn test_projection_reorders_columns_to_match_schema() {
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("a", DataType::Int64, false),
                Field::new("b", DataType::Utf8, false),
                Field::new("c", DataType::Float64, false),
            ])),
            vec![
                Arc::new(Int64Array::from(vec![1, 2])),
                Arc::new(StringArray::from(vec!["x", "y"])),
                Arc::new(Float64Array::from(vec![10.0, 20.0])),
            ],
        )
        .unwrap();

        let projection = Projection::from(vec!["c".into(), "a".into()]);
        let result = projection
            .project_with_options(&batch, &ProjectionOptions::default())
            .unwrap()
            .unwrap();

        assert_eq!(result.num_columns(), 2);
        assert_eq!(result.schema().field(0).name(), "c");
        assert_eq!(result.schema().field(1).name(), "a");

        // verify actual data came along correctly
        let c_col = result
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(c_col.values(), &[10.0, 20.0]);

        let a_col = result
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(a_col.values(), &[1, 2]);
    }

    /// Scenario: apply projection to two input batches have the same columns but in different order
    /// Guarantees: Both projected batches have identical columns order (same column names at the
    /// same expected indices),
    #[test]
    fn test_projection_produces_stable_schema_across_batches_with_different_column_order() {
        let projection = Projection::from(vec!["b".into(), "a".into()]);
        let batch1 = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("a", DataType::Int64, false),
                Field::new("b", DataType::Utf8, false),
            ])),
            vec![
                Arc::new(Int64Array::from(vec![1])),
                Arc::new(StringArray::from(vec!["x"])),
            ],
        )
        .unwrap();
        let batch2 = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("b", DataType::Utf8, false),
                Field::new("a", DataType::Int64, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec!["y"])),
                Arc::new(Int64Array::from(vec![2])),
            ],
        )
        .unwrap();

        let result1 = projection
            .project_with_options(&batch1, &ProjectionOptions::default())
            .unwrap()
            .unwrap();
        let result2 = projection
            .project_with_options(&batch2, &ProjectionOptions::default())
            .unwrap()
            .unwrap();

        // both must have schema ["b", "a"] regardless of input order
        assert_eq!(result1.schema().field(0).name(), "b");
        assert_eq!(result1.schema().field(1).name(), "a");
        assert_eq!(result2.schema().field(0).name(), "b");
        assert_eq!(result2.schema().field(1).name(), "a");

        // verify data is correct in each
        let b1 = result1
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(b1.value(0), "x");

        let b2 = result2
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(b2.value(0), "y");
    }
}
