// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projection utilities for extracting required columns from RecordBatches

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, RecordBatch, RecordBatchOptions, StructArray};
use arrow::compute::cast;
use arrow::datatypes::{DataType, Field, Schema};
use datafusion::common::tree_node::{TreeNode, TreeNodeRecursion, TreeNodeVisitor};
use datafusion::common::{HashMap, HashSet};
use datafusion::error::DataFusionError;
use datafusion::functions::core::getfield::GetFieldFunc;
use datafusion::logical_expr::Expr;
use datafusion::scalar::ScalarValue;

use crate::error::Result;

#[derive(Default)]
pub struct ProjectionOptions {
    /// Whether or not to downcast dictionary arrays to the native type. Some types of expressions,
    /// arithmetic operations for example, do not work on dictionary encoded columns.
    pub downcast_dicts: bool,
}

/// Projection helper that can project a RecordBatch to only the columns needed by an expression
#[derive(Debug)]
pub struct Projection {
    pub schema: ProjectedSchema,
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

    /// Project the record batch to the expected schema. If there are some expected columns in
    /// the passed [`RecordBatch`] which are missing, this will return `None`.
    pub fn project(&self, record_batch: &RecordBatch) -> Result<Option<RecordBatch>> {
        self.project_with_options(record_batch, &ProjectionOptions::default())
    }

    pub fn project_with_options(
        &self,
        record_batch: &RecordBatch,
        options: &ProjectionOptions,
    ) -> Result<Option<RecordBatch>> {
        let (mut fields, mut columns) = match self.project_columns(record_batch) {
            Some(projection) => projection,
            None => return Ok(None),
        };

        if options.downcast_dicts {
            Self::try_downcast_dicts(&mut fields, &mut columns)?;
        }

        // safety: `try_new` should not return an error here unless the columns do not match the
        // fields in the schema, or if the columns are different lengths. Based on how we've
        // constructed the inputs, this should not happen because we've taken them from the input
        let rb = RecordBatch::try_new_with_options(
            Arc::new(Schema::new(fields)),
            columns,
            &RecordBatchOptions::new().with_row_count(Some(record_batch.num_rows())),
        )
        .expect("can project record batch");

        Ok(Some(rb))
    }

    fn project_columns(
        &self,
        record_batch: &RecordBatch,
    ) -> Option<(Vec<Arc<Field>>, Vec<ArrayRef>)> {
        let original_schema = record_batch.schema_ref();

        // TODO - if the heap allocations here have significant perf overhead, we could try reusing
        // these arrays between batches.
        let mut columns = Vec::new();
        let mut fields = Vec::new();

        for projected_col in &self.schema {
            match projected_col {
                ProjectedSchemaColumn::Root(desired_col_name) => {
                    let index = original_schema.index_of(desired_col_name).ok()?;
                    let column = record_batch.column(index).clone();
                    let field = original_schema.fields[index].clone();
                    columns.push(column);
                    fields.push(field)
                }
                ProjectedSchemaColumn::Struct(desired_struct_name, desired_struct_fields) => {
                    let struct_index = original_schema.index_of(desired_struct_name).ok()?;
                    let column = record_batch.column(struct_index);
                    let col_as_struct = column.as_any().downcast_ref::<StructArray>()?;

                    let mut struct_fields = Vec::new();
                    let mut struct_field_defs = Vec::new();

                    for field_name in desired_struct_fields {
                        let (field_index, field) = col_as_struct.fields().find(field_name)?;
                        struct_fields.push(col_as_struct.column(field_index).clone());
                        struct_field_defs.push(field.clone());
                    }

                    // safety: `try_new` will return an error here if the types of arrays we pass
                    // for the fields do not match the field definitions, or if the arrays have
                    // different lengths. Based on the way we've constructed inputs, this should
                    // not happen because we've taken them from the input struct column in order
                    let projected_struct_arr = StructArray::try_new(
                        struct_field_defs.into(),
                        struct_fields,
                        col_as_struct.nulls().cloned(),
                    )
                    .expect("can init StructArray");

                    let projected_field = original_schema.fields[struct_index]
                        .as_ref()
                        .clone()
                        .with_data_type(projected_struct_arr.data_type().clone());
                    fields.push(Arc::new(projected_field));
                    columns.push(Arc::new(projected_struct_arr));
                }
            }
        }

        Some((fields, columns))
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
        if let Expr::ScalarFunction(scalar_udf) = node {
            if scalar_udf
                .func
                .as_ref()
                .inner()
                .as_any()
                .is::<GetFieldFunc>()
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
