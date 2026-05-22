// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, AsArray, Datum, UInt8Array, make_array};
use arrow::buffer::NullBuffer;
use arrow::compute::kernels::cmp::{distinct, eq, gt, gt_eq, lt, lt_eq, neq, not_distinct};
use arrow::compute::{and, not, or};
use arrow::datatypes::DataType;
use arrow::error::ArrowError;
use arrow::{array::BooleanArray, buffer::BooleanBuffer};
use datafusion::logical_expr::{ColumnarValue, Operator};
use datafusion::scalar::ScalarValue;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::project::anyval::{
    arrow_type_to_any_value_type, extract_type_from_any_value_struct, is_any_value_data_type,
};

/// This kernel implements comparing two [`ColumnarValue`]s using the given [`Operation`], with
/// some additional edge case handling beyond what is found in [`arrow::compute::kernels:cmp`].
/// The motivation is that this can be used in cases where we're comparing OTAP columns which:
///
/// - may resolve to different types that are not known at compile time (for example, comparing two
///   attributes in expressions like `attributes["x"] == attributes["y"]`). In such cases, if the
///   types do not match, the expression will always resolve to false unless the operator is
///   [`NotEq`](Operator::NotEq). The exception to this is if one or both sides are a struct
///   representing an `AnyValue`, in which case it will compare the appropriate values columns
///
/// - may resolve to `null` on one or both sides. Arrow's `cmp` kernels typically return `null` for
///   these cases. By contrast, this function will always return `false` for comparisons where at
///   least one side is null, except for special cases of `null == null` and `<not null> != null`
///   in which cases it will return `true`
///
/// Not all [`Operator`] variants are supported. Currently this only supports:
/// - [`Eq`](Operator::Eq)
/// - [`Gt`](Operator::Gt)
/// - [`GtEq`](Operator::GtEq)
/// - [`Lt`](Operator::Lt)
/// - [`LtEq`](Operator::LtEq)
/// - [`NotEq`](Operator::NotEq)
///
/// Passing any other operator variant will cause this to return an error.
///
/// This function also does not, currently, perform any type coercion internally. It is the
/// responsibility of the caller to have already coerced the types for comparison if necessary.
///
pub(crate) fn compare(
    left: &ColumnarValue,
    operator: Operator,
    right: &ColumnarValue,
) -> Result<BooleanArray> {
    let left_data_type = left.data_type();
    let right_data_type = right.data_type();

    let num_rows = match (left, right) {
        (ColumnarValue::Array(arr), _) | (_, ColumnarValue::Array(arr)) => arr.len(),
        _ => 1,
    };

    match (
        is_any_value_data_type(&left_data_type),
        is_any_value_data_type(&right_data_type),
    ) {
        (false, false) => {
            match (&left_data_type, &right_data_type) {
                (DataType::Null, DataType::Null) => compare_with_null(
                    &ColumnarValue::Scalar(ScalarValue::Null),
                    operator,
                    num_rows,
                ),
                (_, DataType::Null) => compare_with_null(left, operator, num_rows),
                (DataType::Null, _) => compare_with_null(right, operator, num_rows),
                _ => {
                    if !are_concrete_types_comparable(&left_data_type, &right_data_type) {
                        // the types are not the same - the comparison is false for all rows
                        return Ok(all_false(num_rows));
                    }

                    let mut result_bool_arr = match operator {
                        // for `Eq` and `NotEq` we are using `not_distinct` and `distinct`
                        // respectively here because these are effectively the same behaviour
                        // as `eq` and `neq` kernels, except internally they handle nulls with the
                        // behaviour we want which is `null == null` produces true and
                        // `null != null` produces false.
                        Operator::Eq => apply_cmp(left, right, not_distinct),
                        Operator::NotEq => apply_cmp(left, right, distinct),

                        Operator::Gt => apply_cmp(left, right, gt),
                        Operator::GtEq => apply_cmp(left, right, gt_eq),
                        Operator::Lt => apply_cmp(left, right, lt),
                        Operator::LtEq => apply_cmp(left, right, lt_eq),
                        _ => {
                            return Err(Error::InvalidPipelineError {
                                cause: format!("Unsupported operation for compare: {operator}"),
                                query_location: None,
                            });
                        }
                    }?;

                    // for any comparison operations other than `eq` and `neq`, if there is a null
                    // on one side, we evaluate the comparison as false. The compute kernels we've
                    // used to produce the result in these cases will have inserted nulls into
                    // positions where one side is null, so here we combine the null buffer with
                    // the values buffer to get the desired comparison behaviour.
                    if !matches!(operator, Operator::Eq | Operator::NotEq) {
                        result_bool_arr = nulls_as_false(result_bool_arr);
                    }

                    Ok(result_bool_arr)
                }
            }
        }
        (true, false) => {
            let left_arr = left.to_array(num_rows)?;

            if right_data_type == DataType::Null {
                return compare_anyvalue_with_null(&left_arr, operator, num_rows);
            }

            let Ok((type_to_compare, value_field_name)) =
                arrow_type_to_any_value_type(&right_data_type)
            else {
                // type can't be in an anyvalue, so can't be compared. Always false, except for
                // NotEq case where a type mismatch is evaluated as true
                return Ok(match operator {
                    Operator::NotEq => all_true(num_rows),
                    _ => all_false(num_rows),
                });
            };

            // create the mask of which rows in the AnyValue column have a type matching the type
            // of which we're comparing
            let type_col = extract_type_from_any_value_struct(&left_arr)?;
            let type_cmp_result = eq(&type_col, &UInt8Array::new_scalar(type_to_compare as u8))?;

            let left_val_to_compare =
                extract_values_column_for_comparison(value_field_name, &left_arr)?;
            let values_cmp_result = compare(&left_val_to_compare, operator, right)?;

            let result = match operator {
                // when not equal - consider not equal if the types don't match
                Operator::NotEq => or(&not(&type_cmp_result)?, &values_cmp_result)?,
                // for all other comparisons, types must match or it evaluates to false
                _ => and(&type_cmp_result, &values_cmp_result)?,
            };

            Ok(result)
        }
        (false, true) => {
            let right_arr = right.to_array(num_rows)?;

            if left_data_type == DataType::Null {
                return compare_anyvalue_with_null(&right_arr, operator, num_rows);
            }

            let Ok((type_to_compare, value_field_name)) =
                arrow_type_to_any_value_type(&left_data_type)
            else {
                // type can't be in an anyvalue, so can't be compared. Always false, except for
                // NotEq case where a type mismatch is evaluated as true
                return Ok(match operator {
                    Operator::NotEq => all_true(num_rows),
                    _ => all_false(num_rows),
                });
            };

            // create the mask of which rows in the AnyValue column have a type matching the type
            // of which we're comparing
            let type_col = extract_type_from_any_value_struct(&right_arr)?;
            let type_cmp_result = eq(&type_col, &UInt8Array::new_scalar(type_to_compare as u8))?;

            let right_val_to_compare =
                extract_values_column_for_comparison(value_field_name, &right_arr)?;
            let values_cmp_result = compare(left, operator, &right_val_to_compare)?;

            let result = match operator {
                // when not equal - consider not equal if the types don't match
                Operator::NotEq => or(&not(&type_cmp_result)?, &values_cmp_result)?,
                // for all other comparisons, types must match or it evaluates to false
                _ => and(&type_cmp_result, &values_cmp_result)?,
            };

            // get the values column
            Ok(result)
        }
        (true, true) => {
            // Iterate through each values column and check which rows pass the comparison. We'll
            // combine the results for each type later on.
            //
            // Perf note: this isn't the most well optimized way to do this, as it does the
            // comparison for the entire length of the  array for all types, including for rows
            // that aren't of the given type. This would be inefficient if some types are present
            // sparsely. However, I expect this code path to be quite rare, as most useful
            // comparisons involving `AnyValue` would be an attribute on one or both sides and for
            // a given attribute key there's a convention that all values will have the same type.
            // In such cases where the type is homogenous, the engine will have already have
            // coerced the struct into a simple column array containing only the values and we'll
            // have already taken a different code path.
            let left_arr = left.to_array(num_rows)?;
            let left_fields = left_arr.as_struct().fields();
            let mut value_type_comparisons = Vec::with_capacity(left_fields.len() + 1);
            for (field_id, field) in left_fields.iter().enumerate() {
                if field.name() == consts::ATTRIBUTE_TYPE {
                    continue;
                }
                if field.name() == consts::ATTRIBUTE_SER {
                    return Err(Error::NotYetSupportedError {
                        message: "comparisons on complex attribute type not yet supported".into(),
                    });
                }

                let values_col = left_arr.as_struct().column(field_id);

                let type_compare_result = compare(
                    &ColumnarValue::Array(Arc::clone(values_col)),
                    operator,
                    right,
                )?;
                value_type_comparisons.push(type_compare_result);
            }

            // compare all the nulls on the left side with right side
            if let Some(nulls_and_empties) = anyval_validity(&left_arr)? {
                let right_compare_with_null =
                    compare(&ColumnarValue::Scalar(ScalarValue::Null), operator, right)?;

                let compare_nulls = match operator {
                    Operator::NotEq => or(&nulls_and_empties, &right_compare_with_null),
                    _ => and(&not(&nulls_and_empties)?, &right_compare_with_null),
                }?;

                value_type_comparisons.push(compare_nulls);
            }

            // combine the results
            let mut result = value_type_comparisons.pop().expect("not empty");
            for next_result in value_type_comparisons {
                let combine_op = match operator {
                    Operator::NotEq => and,
                    _ => or,
                };
                result = combine_op(&result, &next_result)?;
            }

            Ok(result)
        }
    }
}

/// Create a boolean array of given size that is full of true values
fn all_true(num_rows: usize) -> BooleanArray {
    BooleanArray::new(BooleanBuffer::new_set(num_rows), None)
}

/// Create a boolean array of the given size that is full of false values
fn all_false(num_rows: usize) -> BooleanArray {
    BooleanArray::new(BooleanBuffer::new_unset(num_rows), None)
}

/// Return the effective validity buffer of an `AnyValue` struct array. Validity in this case
/// means the AnyValue is not null.
///
/// Note, there are three ways to represent a null AnyValue:
/// - the struct is null at some position
/// - the struct has [`AttributeValueType::Empty`] in the `type` column
/// - the struct has a null in the values column selected by the type discriminant
///
/// The effective validity buffer will be `false` in positions where one of these criterion are met
fn anyval_validity(anyval_arr: &ArrayRef) -> Result<Option<BooleanArray>> {
    let struct_and_empty_validity = anyval_struct_or_empty_validity(anyval_arr)?;
    let value_col_validity = anyval_value_validity(anyval_arr)?;

    let validity = match (struct_and_empty_validity, value_col_validity) {
        (Some(l), Some(r)) => {
            let combined_validity = and_boolean_buffers(l.values(), r.values());
            Some(BooleanArray::new(combined_validity, None))
        }
        (Some(v), _) | (_, Some(v)) => Some(v),
        _ => None,
    };

    Ok(validity)
}

/// Returns the combined validity buffer for the selected values columns from an `AnyValue` struct
/// array. This means it returns a boolean array that is `false` if the type column selects some
/// values column and the selected values column is null in this position.
///
/// For example if we had a struct array like:
/// ```text
/// type | str  | int
/// -----|------|-----
/// Str  | "a"  | null
/// Str  | null | null
/// Int  | null |  0
/// Int  | null | null
/// ```
/// The result would be `[true, false, true, false]`
///
fn anyval_value_validity(anyval_arr: &ArrayRef) -> Result<Option<BooleanArray>> {
    let type_col = extract_type_from_any_value_struct(anyval_arr)?;
    let anyval_struct_arr = anyval_arr.as_struct();

    let mut result = all_false(anyval_arr.len());

    for (field_name, attr_type) in [
        (consts::ATTRIBUTE_STR, AttributeValueType::Str),
        (consts::ATTRIBUTE_BOOL, AttributeValueType::Bool),
        (consts::ATTRIBUTE_INT, AttributeValueType::Int),
        (consts::ATTRIBUTE_DOUBLE, AttributeValueType::Double),
        (consts::ATTRIBUTE_BYTES, AttributeValueType::Bytes),
    ] {
        if let Some(value_col) = anyval_struct_arr.column_by_name(field_name) {
            let mut valid_for_type = eq(&type_col, &UInt8Array::new_scalar(attr_type as u8))?;
            if let Some(nulls) = value_col.nulls() {
                valid_for_type = BooleanArray::new(
                    and_boolean_buffers(nulls.inner(), valid_for_type.values()),
                    None,
                );
            }
            result = or(&result, &valid_for_type)?;
        }
    }

    // handle the special serialized column
    if let Some(value_col) = anyval_struct_arr.column_by_name(consts::ATTRIBUTE_SER) {
        let valid_for_map = eq(
            &type_col,
            &UInt8Array::new_scalar(AttributeValueType::Map as u8),
        )?;
        let valid_for_slice = eq(
            &type_col,
            &UInt8Array::new_scalar(AttributeValueType::Slice as u8),
        )?;
        let mut valid_for_type = or(&valid_for_map, &valid_for_slice)?;

        if let Some(nulls) = value_col.nulls() {
            valid_for_type = BooleanArray::new(
                and_boolean_buffers(nulls.inner(), valid_for_type.values()),
                None,
            );
        }
        result = or(&result, &valid_for_type)?;
    }

    Ok(result.has_false().then_some(result))
}

/// Returns the effective validity bitmap for a struct array containing an `AnyValue` considering
/// the null buffer of the struct array and any values identified as type `Empty` by the type
/// column.
///
/// For example if we had:
/// ```text
/// nulls | type  | ...
/// ----- | ----- | ...
/// true  | Str   | ...
/// true  | Empty | ...
/// false | Str   | ...
/// ```
///
/// This would return `[true, false, false]`
///
/// This can be combined with the result of [`any_value_validity`] to get the overall effective
/// validity for the column.
fn anyval_struct_or_empty_validity(anyval_struct_arr: &ArrayRef) -> Result<Option<BooleanArray>> {
    let type_col = extract_type_from_any_value_struct(anyval_struct_arr)?;
    let type_validity = neq(
        &type_col,
        &UInt8Array::new_scalar(AttributeValueType::Empty as u8),
    )?;

    let validity = if type_validity.has_false() || anyval_struct_arr.nulls().is_some() {
        let validity = match anyval_struct_arr.nulls().map(|b| b.inner()) {
            Some(struct_validity) => {
                let type_validity = type_validity.values();
                let combined_validity = and_boolean_buffers(type_validity, struct_validity);
                BooleanArray::new(combined_validity, None)
            }
            None => type_validity,
        };

        Some(validity)
    } else {
        None
    };

    Ok(validity)
}

fn and_boolean_buffers(left: &BooleanBuffer, right: &BooleanBuffer) -> BooleanBuffer {
    BooleanBuffer::from_bitwise_binary_op(
        left.inner(),
        left.offset(),
        right.inner(),
        right.offset(),
        left.len(),
        |a, b| a & b,
    )
}

/// Helper determine if the two datatypes can be compared - returns `true` if the types represent
/// the same logical data type.
fn are_concrete_types_comparable(left: &DataType, right: &DataType) -> bool {
    match (left, right) {
        (DataType::Dictionary(_, v), _) => are_concrete_types_comparable(v.as_ref(), right),
        (_, DataType::Dictionary(_, v)) => are_concrete_types_comparable(left, v.as_ref()),
        (_, _) => left == right,
    }
}

fn apply_cmp<F>(left: &ColumnarValue, right: &ColumnarValue, cmp: F) -> Result<BooleanArray>
where
    F: Fn(&dyn Datum, &dyn Datum) -> std::result::Result<BooleanArray, ArrowError>,
{
    let cmp_result = match (left, right) {
        (ColumnarValue::Array(left_arr), ColumnarValue::Array(right_arr)) => {
            cmp(left_arr, right_arr)
        }
        (ColumnarValue::Scalar(left_scalar), ColumnarValue::Array(right_arr)) => {
            cmp(&left_scalar.to_scalar()?, right_arr)
        }
        (ColumnarValue::Array(left_arr), ColumnarValue::Scalar(right_scalar)) => {
            cmp(left_arr, &right_scalar.to_scalar()?)
        }
        (ColumnarValue::Scalar(left_scalar), ColumnarValue::Scalar(right_scalar)) => {
            cmp(&left_scalar.to_scalar()?, &right_scalar.to_scalar()?)
        }
    };

    Ok(cmp_result?)
}

/// Takes any null in from the boolean array's null buffer and combines them with the values
/// to produce a new array where nulls in the input array become false values.
///
/// For example if we had:
/// ```text
/// nulls | values
/// ------|-------
/// true  | false
/// false | true
/// false | false
/// true  | true
/// ```
///
/// the result would be `[false, false, false, true]`
///  
fn nulls_as_false(arr: BooleanArray) -> BooleanArray {
    match arr.nulls() {
        Some(nulls) => {
            let selection_vec_bool_buff = arr.values();
            let validity_bool_buff = nulls.inner();
            let nulls_as_false_bool_buff =
                and_boolean_buffers(selection_vec_bool_buff, validity_bool_buff);
            BooleanArray::new(nulls_as_false_bool_buff, None)
        }
        None => {
            // no nulls - just return the original
            arr
        }
    }
}

/// Perform the comparison with value column where the value on the other side is null.
///
/// When the operator is `==`, this will return `true`` for all indices where the input column is
/// also null. Conversely, when the operator is `!=`, this returns `true` for all indices where the
/// input value is not null. For all other operators, it produces an all `false` result.
fn compare_with_null(
    value: &ColumnarValue,
    operator: Operator,
    num_rows: usize,
) -> Result<BooleanArray> {
    match value {
        ColumnarValue::Scalar(scalar) => {
            let set = match operator {
                Operator::Eq => scalar.is_null(),
                Operator::NotEq => !scalar.is_null(),
                // Other operations (>, >=, <, <=, etc.) evaluate to false when either side null:
                _ => false,
            };

            Ok(if set {
                all_true(num_rows)
            } else {
                all_false(num_rows)
            })
        }
        ColumnarValue::Array(arr) => {
            match operator {
                Operator::Eq => {
                    let validity_bitmap_buffer = arr.nulls().map(|b| b.inner()).cloned();
                    let inverted_validity = match validity_bitmap_buffer {
                        // invert to get which rows are null:
                        Some(validity_bitmap_buffer) => BooleanBuffer::from_bitwise_unary_op(
                            validity_bitmap_buffer.inner(),
                            validity_bitmap_buffer.offset(),
                            validity_bitmap_buffer.len(),
                            |v| !v,
                        ),
                        None => {
                            // all false, because there are no nulls ...
                            BooleanBuffer::new_unset(arr.len())
                        }
                    };

                    Ok(BooleanArray::new(inverted_validity, None))
                }
                Operator::NotEq => {
                    let validity_bitmap =
                        arr.nulls().map(|b| b.inner()).cloned().unwrap_or_else(|| {
                            // all true because there are no null rows
                            BooleanBuffer::new_set(arr.len())
                        });
                    Ok(BooleanArray::new(validity_bitmap, None))
                }
                // Other operations (>, >=, <, <=, etc.) evaluate to false when either side null:
                _ => Ok(all_false(arr.len())),
            }
        }
    }
}

fn compare_anyvalue_with_null(
    anyval_arr: &ArrayRef,
    operator: Operator,
    num_rows: usize,
) -> Result<BooleanArray> {
    let validity = anyval_validity(anyval_arr)?;

    Ok(match operator {
        Operator::Eq => {
            // null == null -> true, false otherwise, so we invert the validity bitmap
            if let Some(validity) = validity {
                not(&validity)?
            } else {
                // nothing null - all false
                all_false(num_rows)
            }
        }
        Operator::NotEq => {
            if let Some(validity) = validity {
                // null != null -> true
                validity
            } else {
                // nothing null, so everything is not equal to null
                all_true(num_rows)
            }
        }
        // Other operations (>, >=, <, <=, etc.) evaluate to false when either side null:
        _ => all_false(num_rows),
    })
}

/// take the field identified by `value_field_name` from the struct, and possibly adjust the nulls
/// to account for any effectively null attributes that may be marked as null based on either the
/// struct validity, or the type column with a value of `Empty`. The goal is the produce a values
/// column that can be compared another non-struct column value with a concrete type.
fn extract_values_column_for_comparison(
    value_field_name: Option<&'static str>,
    anyvalue_struct_arr: &ArrayRef,
) -> Result<ColumnarValue> {
    match value_field_name
        .and_then(|field_name| anyvalue_struct_arr.as_struct().column_by_name(field_name))
    {
        None => Ok(ColumnarValue::Scalar(ScalarValue::Null)),
        Some(value_column) => {
            // combine any nulls resulting from null rows or nulls in the column's validity
            // bitmap into the nulls for the value column before comparing
            let extra_validity = anyval_struct_or_empty_validity(anyvalue_struct_arr)?;
            let value_column = merge_validity(extra_validity.as_ref(), value_column)?;
            Ok(ColumnarValue::Array(value_column))
        }
    }
}

/// Merges the extra validity buffer (if not `None`) into the existing columns validity buffer
/// and returns the original values_column with new validity. This can be used to mark some
/// additional rows in values_column as null.
fn merge_validity(
    extra_validity: Option<&BooleanArray>,
    value_column: &ArrayRef,
) -> Result<ArrayRef> {
    let result = match (
        extra_validity.as_ref().map(|b| b.values()),
        value_column.nulls().map(|b| b.inner()),
    ) {
        (Some(extra), Some(values)) => {
            // combine the validity buffers and replace
            let nulls = and_boolean_buffers(extra, values);
            let new_data = value_column
                .to_data()
                .into_builder()
                .nulls(Some(NullBuffer::new(nulls)))
                .build()?;
            make_array(new_data)
        }
        (Some(extra), None) => {
            // insert the new validity buffer
            let new_data = value_column
                .to_data()
                .into_builder()
                .nulls(Some(NullBuffer::new(extra.clone())))
                .build()?;
            make_array(new_data)
        }
        // no new nulls to insert, just return the original column:
        _ => Arc::clone(value_column),
    };

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::Arc;

    use arrow::{
        array::{
            DictionaryArray, Int64Array, NullArray, StringArray, StructArray,
            TimestampNanosecondArray, UInt16Array,
        },
        buffer::{BooleanBuffer, NullBuffer},
        compute::not,
        datatypes::Field,
    };
    use datafusion::scalar::ScalarValue;
    use otap_df_pdata::{otlp::attributes::AttributeValueType, schema::consts};

    // helper function for testing when the operation is commutative
    fn test_commutative(
        left: ColumnarValue,
        operator: Operator,
        right: ColumnarValue,
        expected: BooleanArray,
    ) {
        let result = compare(&left, operator, &right).unwrap();
        assert_eq!(&result, &expected);

        // check that it returns expected when the operators are on the other side
        let result = compare(&right, operator, &left).unwrap();
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_null_eq_concrete_column_with_nulls() {
        let left = StringArray::from_iter([Some("a"), None, Some("b")]);
        let expected = BooleanArray::new(BooleanBuffer::from(vec![false, true, false]), None);

        // test with null scalar
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Scalar(ScalarValue::Null),
            expected.clone(),
        );

        // test with null array
        test_commutative(
            ColumnarValue::Array(Arc::new(left)),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(NullArray::new(3))),
            expected,
        );
    }

    #[test]
    fn test_null_scalar_eq_concrete_column_without_nulls() {
        let left = StringArray::from_iter_values(["a", "b", "c"]);
        let expected = BooleanArray::new(BooleanBuffer::from(vec![false, false, false]), None);
        // test with null scalar
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Scalar(ScalarValue::Null),
            expected.clone(),
        );

        // test with null array
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(NullArray::new(3))),
            expected,
        );
    }

    #[test]
    fn test_null_quantity_comparison_with_concrete_column() {
        let left = StringArray::from_iter([Some("a"), None, Some("b")]);
        let expected = BooleanArray::new(BooleanBuffer::from(vec![false, false, false]), None);

        for operator in [Operator::Gt, Operator::GtEq, Operator::Lt, Operator::LtEq] {
            // test with null scalar
            let result = compare(
                &ColumnarValue::Array(Arc::new(left.clone())),
                operator,
                &ColumnarValue::Scalar(ScalarValue::Null),
            )
            .unwrap();
            assert_eq!(&result, &expected);

            // test with null array
            let result = compare(
                &ColumnarValue::Array(Arc::new(left.clone())),
                operator,
                &ColumnarValue::Array(Arc::new(NullArray::new(3))),
            )
            .unwrap();
            assert_eq!(&result, &expected);
        }
    }

    #[test]
    fn test_null_scalar_neq_concrete_column_with_nulls() {
        let left = StringArray::from_iter([Some("a"), None, Some("b")]);
        let expected = BooleanArray::new(BooleanBuffer::from(vec![true, false, true]), None);
        // test with null scalar
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Scalar(ScalarValue::Null),
            expected.clone(),
        );

        // test with null array
        test_commutative(
            ColumnarValue::Array(Arc::new(left)),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(NullArray::new(3))),
            expected,
        );
    }

    #[test]
    fn test_null_scalar_neq_concrete_column_without_nulls() {
        let left = StringArray::from_iter_values(["a", "b", "c"]);
        let expected = BooleanArray::new(BooleanBuffer::from(vec![true, true, true]), None);
        // test with null scalar
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Scalar(ScalarValue::Null),
            expected.clone(),
        );

        // test with null array
        test_commutative(
            ColumnarValue::Array(Arc::new(left)),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(NullArray::new(3))),
            expected,
        );
    }

    #[test]
    fn test_null_scalar_eq_scalar() {
        test_commutative(
            ColumnarValue::Scalar(ScalarValue::Null),
            Operator::Eq,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into()))),
            BooleanArray::new(BooleanBuffer::new_unset(1), None),
        );

        test_commutative(
            ColumnarValue::Scalar(ScalarValue::Null),
            Operator::Eq,
            ColumnarValue::Scalar(ScalarValue::Utf8(None)),
            BooleanArray::new(BooleanBuffer::new_set(1), None),
        );

        test_commutative(
            ColumnarValue::Scalar(ScalarValue::Null),
            Operator::Eq,
            ColumnarValue::Scalar(ScalarValue::Null),
            BooleanArray::new(BooleanBuffer::new_set(1), None),
        );
    }

    #[test]
    fn test_null_scalar_neq_scalar() {
        test_commutative(
            ColumnarValue::Scalar(ScalarValue::Null),
            Operator::NotEq,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into()))),
            BooleanArray::new(BooleanBuffer::new_set(1), None),
        );

        test_commutative(
            ColumnarValue::Scalar(ScalarValue::Null),
            Operator::NotEq,
            ColumnarValue::Scalar(ScalarValue::Utf8(None)),
            BooleanArray::new(BooleanBuffer::new_unset(1), None),
        );

        test_commutative(
            ColumnarValue::Scalar(ScalarValue::Null),
            Operator::NotEq,
            ColumnarValue::Scalar(ScalarValue::Null),
            BooleanArray::new(BooleanBuffer::new_unset(1), None),
        );
    }

    #[test]
    fn test_null_scalar_quantitative_compare_with_scalar() {
        for operator in [Operator::Lt, Operator::Gt, Operator::GtEq, Operator::LtEq] {
            let result = compare(
                &ColumnarValue::Scalar(ScalarValue::Null),
                operator,
                &ColumnarValue::Scalar(ScalarValue::Int64(Some(5))),
            )
            .unwrap();
            let expected = BooleanArray::from(vec![false]);
            assert_eq!(&result, &expected);

            let result = compare(
                &ColumnarValue::Scalar(ScalarValue::Int64(Some(5))),
                operator,
                &ColumnarValue::Scalar(ScalarValue::Null),
            )
            .unwrap();
            let expected = BooleanArray::from(vec![false]);
            assert_eq!(&result, &expected);
        }
    }

    #[test]
    fn test_concrete_type_arr_eq_concrete_type_array_with_nulls() {
        let left = StringArray::from_iter([Some("a"), None, Some("b"), None, Some("c")]);
        let right = StringArray::from_iter([Some("a"), Some("b"), Some("c"), None, Some("c")]);
        let expected = BooleanArray::new(
            BooleanBuffer::from(vec![true, false, false, true, true]),
            None,
        );

        test_commutative(
            ColumnarValue::Array(Arc::new(left)),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right)),
            expected,
        );
    }

    #[test]
    fn test_concrete_type_arr_neq_concrete_type_array_with_nulls() {
        let left = StringArray::from_iter([Some("a"), None, Some("b"), None, Some("c")]);
        let right = StringArray::from_iter([Some("a"), Some("b"), Some("c"), None, Some("c")]);
        let expected = BooleanArray::new(
            BooleanBuffer::from(vec![false, true, true, false, false]),
            None,
        );

        test_commutative(
            ColumnarValue::Array(Arc::new(left)),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right)),
            expected,
        );
    }

    #[test]
    fn test_concrete_type_arr_gt_concrete_type_array_with_nulls() {
        let left = Int64Array::from_iter([Some(2), Some(1), Some(1), None, None]);
        let right = Int64Array::from_iter([Some(1), Some(2), Some(1), Some(1), None]);
        let expected = BooleanArray::new(
            BooleanBuffer::from(vec![true, false, false, false, false]),
            None,
        );

        let result = compare(
            &ColumnarValue::Array(Arc::new(left)),
            Operator::Gt,
            &ColumnarValue::Array(Arc::new(right)),
        )
        .unwrap();
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_concrete_type_arr_gte_concrete_type_array_with_nulls() {
        let left = Int64Array::from_iter([Some(2), Some(1), Some(1), None, None]);
        let right = Int64Array::from_iter([Some(1), Some(2), Some(1), Some(1), None]);
        let expected = BooleanArray::new(
            BooleanBuffer::from(vec![true, false, true, false, false]),
            None,
        );
        let result = compare(
            &ColumnarValue::Array(Arc::new(left)),
            Operator::GtEq,
            &ColumnarValue::Array(Arc::new(right)),
        )
        .unwrap();
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_concrete_type_arr_lt_concrete_type_array_with_nulls() {
        let left = Int64Array::from_iter([Some(2), Some(1), Some(1), None, None]);
        let right = Int64Array::from_iter([Some(1), Some(2), Some(1), Some(1), None]);
        let expected = BooleanArray::new(
            BooleanBuffer::from(vec![false, true, false, false, false]),
            None,
        );

        let result = compare(
            &ColumnarValue::Array(Arc::new(left)),
            Operator::Lt,
            &ColumnarValue::Array(Arc::new(right)),
        )
        .unwrap();
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_concrete_type_arr_lte_concrete_type_array_with_nulls() {
        let left = Int64Array::from_iter([Some(2), Some(1), Some(1), None, None]);
        let right = Int64Array::from_iter([Some(1), Some(2), Some(1), Some(1), None]);
        let expected = BooleanArray::new(
            BooleanBuffer::from(vec![false, true, true, false, false]),
            None,
        );
        let result = compare(
            &ColumnarValue::Array(Arc::new(left)),
            Operator::LtEq,
            &ColumnarValue::Array(Arc::new(right)),
        )
        .unwrap();
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_non_matching_types_return_all_false_when_compared() {
        let result = compare(
            &ColumnarValue::Array(Arc::new(StringArray::from_iter_values(["a", "b", "c"]))),
            Operator::Eq,
            &ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 2, 3]))),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, false, false]);
        assert_eq!(&result, &expected)
    }

    #[test]
    fn test_unsupported_operator_returns_error() {
        let err = compare(
            &ColumnarValue::Array(Arc::new(StringArray::from_iter_values(["a", "b", "c"]))),
            Operator::And,
            &ColumnarValue::Array(Arc::new(StringArray::from_iter_values(["a", "b", "c"]))),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("Unsupported operation for compare:"),
            "unexpected error message {}",
            err
        )
    }

    #[test]
    fn test_all_null_eq_anyvalue_with_nulls() {
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([Some(0), Some(1), None, Some(0), Some(0)]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
            ],
            Some(NullBuffer::new(BooleanBuffer::from(vec![
                true, true, true, true, false,
            ]))),
        );

        // compare with null array
        let result = compare(
            &ColumnarValue::Array(Arc::new(NullArray::new(5))),
            Operator::Eq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, false, true, true, true]);
        assert_eq!(&result, &expected);

        // compare with null scalar
        let result = compare(
            &ColumnarValue::Scalar(ScalarValue::Null),
            Operator::Eq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        assert_eq!(&result, &expected);

        // check not equals as well, should be the opposite
        let result = compare(
            &ColumnarValue::Array(Arc::new(NullArray::new(5))),
            Operator::NotEq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = not(&expected).unwrap();
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Scalar(ScalarValue::Null),
            Operator::NotEq,
            &ColumnarValue::Array(Arc::new(right)),
        )
        .unwrap();
        assert_eq!(&result, &expected);
    }

    // TODO
    // - same test as below where left is scalar
    // - same test as below where left has nulls
    // - ^^ maybe these are already covered?
    #[test]
    fn test_non_null_string_eq_anyvalue_no_nulls() {
        let left = StringArray::from_iter_values(["a", "a", "a"]);
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 0]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
            ],
            None,
        );

        let expected = BooleanArray::from(vec![true, false, false]);
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right.clone())),
            expected.clone(),
        );

        // check not equals as well, should be the opposite
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right.clone())),
            not(&expected).unwrap(),
        );
    }

    #[test]
    fn test_non_null_string_eq_anyvalue_with_nulls() {
        let left = StringArray::from_iter_values(["a", "a", "a", "a", "a"]);
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([Some(0), Some(1), None, Some(0), Some(0)]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
            ],
            Some(NullBuffer::new(BooleanBuffer::from(vec![
                true, true, true, true, false,
            ]))),
        );

        let expected = BooleanArray::from(vec![true, false, false, false, false]);
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right.clone())),
            expected.clone(),
        );

        // check not equals as well, should be the opposite
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right.clone())),
            not(&expected).unwrap(),
        );
    }

    #[test]
    fn test_int_quantity_compare_with_anyvalue_no_nulls() {
        let left = Int64Array::from_iter_values([1, 1, 1]);
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 2]),
                    Arc::new(Int64Array::from_iter_values([0, 1, 2])),
                )),
            ],
            None,
        );

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Gt,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![true, false, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::GtEq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![true, true, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Lt,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, false, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::LtEq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, true, false]);
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_int_quantity_compare_with_anyvalue_with_nulls() {
        let left = Int64Array::from_iter_values([1, 1, 1, 1, 1]);
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([Some(0), Some(1), None, Some(2), Some(3)]),
                    Arc::new(Int64Array::from_iter_values([0, 1, 2, 3])),
                )),
            ],
            Some(NullBuffer::new(BooleanBuffer::from(vec![
                true, true, true, true, false,
            ]))),
        );

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Gt,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![true, false, false, false, false]);
        assert_eq!(&result, &expected);

        // test comparison in opposite direction
        let result = compare(
            &ColumnarValue::Array(Arc::new(right.clone())),
            Operator::Gt,
            &ColumnarValue::Array(Arc::new(left.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, false, false, false, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::GtEq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![true, true, false, false, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(right.clone())),
            Operator::GtEq,
            &ColumnarValue::Array(Arc::new(left.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, true, false, false, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Lt,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, false, false, false, false]);
        assert_eq!(&result, &expected);

        let result = compare(
            &ColumnarValue::Array(Arc::new(left.clone())),
            Operator::LtEq,
            &ColumnarValue::Array(Arc::new(right.clone())),
        )
        .unwrap();
        let expected = BooleanArray::from(vec![false, true, false, false, false]);
        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_invalid_type_compare_with_anyvalue() {
        let left = TimestampNanosecondArray::from_iter_values([1, 1]);
        let right = StructArray::new(
            vec![Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)].into(),
            vec![Arc::new(UInt8Array::from_iter_values(vec![
                AttributeValueType::Int as u8,
                AttributeValueType::Int as u8,
            ]))],
            None,
        );

        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right.clone())),
            BooleanArray::from(vec![false, false]),
        );

        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right.clone())),
            BooleanArray::from(vec![true, true]),
        );
    }

    #[test]
    fn test_compare_both_sides_anyvalues_equality_no_nulls() {
        let left = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some("a"),
                    Some("a"),
                    None,
                    None,
                    Some("a"),
                ])),
                Arc::new(Int64Array::from_iter([None, None, Some(0), Some(0), None])),
            ],
            None,
        );
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some("a"),
                    Some("b"),
                    None,
                    None,
                    Some("c"),
                ])),
                Arc::new(Int64Array::from_iter([None, None, Some(0), Some(1), None])),
            ],
            None,
        );

        let expected = BooleanArray::from(vec![true, false, true, false, false]);
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right.clone())),
            expected.clone(),
        );

        // check not equals as well, should be the opposite
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right.clone())),
            not(&expected).unwrap(),
        );
    }

    #[test]
    fn test_compare_both_sides_anyvalues_equality_some_nulls() {
        // left         Right   Equals (expected)
        // ----         -----   -----------------
        // Str(a)       Str(a)  True
        // Int(0)       Int(0)  True
        // Str(a)       Str(a)  True
        // Empty        Empty   True
        // Str(a)       Empty   False
        // Str(Null)    Empty   True
        // Str(Null)    Null    True
        // Empty        Null    True

        let left = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some("a"),
                    None,
                    Some("a"),
                    None,
                    Some("a"),
                    None,
                    None,
                    None,
                ])),
                Arc::new(Int64Array::from_iter([
                    None,
                    Some(0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
            ],
            Some(NullBuffer::new(BooleanBuffer::from(vec![
                true, true, true, true, true, true, true, false,
            ]))),
        );
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some("a"),
                    None,
                    Some("a"),
                    None,
                    None,
                    None,
                    Some("n/a"), // will be set to null in null bitmask
                    None,
                ])),
                Arc::new(Int64Array::from_iter([
                    None,
                    Some(0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
            ],
            Some(NullBuffer::new(BooleanBuffer::from(vec![
                true, true, true, true, true, true, false, true,
            ]))),
        );

        let expected = BooleanArray::from(vec![true, true, true, true, false, true, true, true]);
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right.clone())),
            expected.clone(),
        );

        // check not equals as well, should be the opposite
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right.clone())),
            not(&expected).unwrap(),
        );
    }

    #[test]
    fn test_compare_one_side_all_empty() {
        let left = StructArray::new(
            vec![Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)].into(),
            vec![Arc::new(UInt8Array::from_iter_values(vec![
                AttributeValueType::Empty as u8,
                AttributeValueType::Empty as u8,
                AttributeValueType::Empty as u8,
                AttributeValueType::Empty as u8,
                AttributeValueType::Empty as u8,
            ]))],
            None,
        );
        let right = StructArray::new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some("a"),
                    Some("b"),
                    None,
                    None,
                    Some("c"),
                ])),
                Arc::new(Int64Array::from_iter([None, None, Some(0), Some(1), None])),
            ],
            None,
        );

        let expected = BooleanArray::from(vec![false, false, false, false, true]);
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::Eq,
            ColumnarValue::Array(Arc::new(right.clone())),
            expected.clone(),
        );

        // check not equals as well, should be the opposite
        test_commutative(
            ColumnarValue::Array(Arc::new(left.clone())),
            Operator::NotEq,
            ColumnarValue::Array(Arc::new(right.clone())),
            not(&expected).unwrap(),
        );
    }
}
