// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utility macros for creating Arrow RecordBatch instances with dictionary support.

/// Create a record batch with support for dictionary-encoded arrays.
///
/// This macro extends the standard Arrow record batch creation to support dictionary types.
/// For dictionary types, use a tuple `(KeyType, ValueType)` for the type and
/// `([keys...], [values...])` for the data.
///
/// Both inline literals and pre-defined variables are supported for data values.
///
/// # Examples
///
/// ```ignore
/// // Regular array with inline data
/// let batch = record_batch!(("a", Int32, [1, 2, 3])).unwrap();
///
/// // Regular array with pre-defined data
/// let values = vec![1, 2, 3];
/// let batch = record_batch!(("a", Int32, values)).unwrap();
///
/// // Dictionary array with inline data
/// let batch = record_batch!(
///     ("a", (Int32, Utf8), ([0, 1, 2], ["foo", "bar", "baz"]))
/// ).unwrap();
///
/// // Dictionary array with pre-defined data
/// let keys = vec![0u8, 1, 2];
/// let values = vec!["foo", "bar", "baz"];
/// let batch = record_batch!(
///     ("a", (UInt8, Utf8), (keys, values))
/// ).unwrap();
///
/// ```
///
/// # Future enhancements:
///
/// - TODO: Support for nullable arrays using [None, Some(1)]
/// - TODO: Support for structs using something similar to:
///   ("a", Struct, {
///   ("b", Int32, [1, 2, 3]),
///   })
#[macro_export]
macro_rules! record_batch {
    ($(($name:expr, $($rest:tt)*)),* $(,)?) => {
        {
            let schema = std::sync::Arc::new(arrow_schema::Schema::new(vec![
                $(
                    record_batch!(@field_def $name, $($rest)*),
                )*
            ]));

            let batch = arrow::array::RecordBatch::try_new(
                schema,
                vec![$(
                    $crate::record_batch!(@array $($rest)*),
                )*]
            );

            batch
        }
    };

    // Field definition patterns
    (@field_def $name:expr, $type:ident, [$($values:expr),*]) => {
        arrow_schema::Field::new($name, arrow_schema::DataType::$type, true)
    };

    (@field_def $name:expr, $type:ident, $values:expr) => {
        arrow_schema::Field::new($name, arrow_schema::DataType::$type, true)
    };

    (@field_def $name:expr, ($key_type:ident, $value_type:ident), ([$($keys:expr),*], [$($values:expr),*])) => {
        arrow_schema::Field::new(
            $name,
            arrow_schema::DataType::Dictionary(
                Box::new(arrow_schema::DataType::$key_type),
                Box::new(arrow_schema::DataType::$value_type),
            ),
            true
        )
    };

    (@field_def $name:expr, ($key_type:ident, $value_type:ident), ($keys:expr, $values:expr)) => {
        arrow_schema::Field::new(
            $name,
            arrow_schema::DataType::Dictionary(
                Box::new(arrow_schema::DataType::$key_type),
                Box::new(arrow_schema::DataType::$value_type),
            ),
            true
        )
    };

    // Array creation patterns
    (@array $type:ident, [$($values:expr),*]) => {
        std::sync::Arc::new(record_batch!(@create_array $type, [$($values),*])) as std::sync::Arc<dyn arrow::array::Array>
    };

    (@array $type:ident, $values:expr) => {
        std::sync::Arc::new(record_batch!(@create_array_from_expr $type, $values)) as std::sync::Arc<dyn arrow::array::Array>
    };

    (@array ($key_type:ident, $value_type:ident), ([$($keys:expr),*], [$($values:expr),*])) => {
        {
            use arrow::array::DictionaryArray;
            let keys = record_batch!(@create_array $key_type, [$($keys),*]);
            let values = std::sync::Arc::new(
                record_batch!(@create_array $value_type, [$($values),*])
            );
            std::sync::Arc::new(DictionaryArray::new(keys, values)) as std::sync::Arc<dyn arrow::array::Array>
        }
    };

    (@array ($key_type:ident, $value_type:ident), ($keys:expr, $values:expr)) => {
        {
            use arrow::array::DictionaryArray;
            let keys = record_batch!(@create_array_from_expr $key_type, $keys);
            let values = std::sync::Arc::new(
                record_batch!(@create_array_from_expr $value_type, $values)
            );
            std::sync::Arc::new(DictionaryArray::new(keys, values)) as std::sync::Arc<dyn arrow::array::Array>
        }
    };

    // Create array implementations for inline literals
    (@create_array Boolean, [$($values:expr),*]) => {
        arrow::array::BooleanArray::from(vec![$($values),*])
    };
    (@create_array Int8, [$($values:expr),*]) => {
        arrow::array::Int8Array::from(vec![$($values),*])
    };
    (@create_array Int16, [$($values:expr),*]) => {
        arrow::array::Int16Array::from(vec![$($values),*])
    };
    (@create_array Int32, [$($values:expr),*]) => {
        arrow::array::Int32Array::from(vec![$($values),*])
    };
    (@create_array Int64, [$($values:expr),*]) => {
        arrow::array::Int64Array::from(vec![$($values),*])
    };
    (@create_array UInt8, [$($values:expr),*]) => {
        arrow::array::UInt8Array::from(vec![$($values),*])
    };
    (@create_array UInt16, [$($values:expr),*]) => {
        arrow::array::UInt16Array::from(vec![$($values),*])
    };
    (@create_array UInt32, [$($values:expr),*]) => {
        arrow::array::UInt32Array::from(vec![$($values),*])
    };
    (@create_array UInt64, [$($values:expr),*]) => {
        arrow::array::UInt64Array::from(vec![$($values),*])
    };
    (@create_array Float16, [$($values:expr),*]) => {
        arrow::array::Float16Array::from(vec![$($values),*])
    };
    (@create_array Float32, [$($values:expr),*]) => {
        arrow::array::Float32Array::from(vec![$($values),*])
    };
    (@create_array Float64, [$($values:expr),*]) => {
        arrow::array::Float64Array::from(vec![$($values),*])
    };
    (@create_array Utf8, [$($values:expr),*]) => {
        arrow::array::StringArray::from(vec![$($values),*])
    };

    // Create array implementations for expressions (variables)
    (@create_array_from_expr Boolean, $values:expr) => {
        arrow::array::BooleanArray::from($values)
    };
    (@create_array_from_expr Int8, $values:expr) => {
        arrow::array::Int8Array::from($values)
    };
    (@create_array_from_expr Int16, $values:expr) => {
        arrow::array::Int16Array::from($values)
    };
    (@create_array_from_expr Int32, $values:expr) => {
        arrow::array::Int32Array::from($values)
    };
    (@create_array_from_expr Int64, $values:expr) => {
        arrow::array::Int64Array::from($values)
    };
    (@create_array_from_expr UInt8, $values:expr) => {
        arrow::array::UInt8Array::from($values)
    };
    (@create_array_from_expr UInt16, $values:expr) => {
        arrow::array::UInt16Array::from($values)
    };
    (@create_array_from_expr UInt32, $values:expr) => {
        arrow::array::UInt32Array::from($values)
    };
    (@create_array_from_expr UInt64, $values:expr) => {
        arrow::array::UInt64Array::from($values)
    };
    (@create_array_from_expr Float16, $values:expr) => {
        arrow::array::Float16Array::from($values)
    };
    (@create_array_from_expr Float32, $values:expr) => {
        arrow::array::Float32Array::from($values)
    };
    (@create_array_from_expr Float64, $values:expr) => {
        arrow::array::Float64Array::from($values)
    };
    (@create_array_from_expr Utf8, $values:expr) => {
        arrow::array::StringArray::from($values)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_record_batch() {
        use arrow::array::Int32Array;
        let batch = record_batch!(("a", Int32, [1, 2, 3])).unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 1);
        assert!(
            batch
                .column(0)
                .as_any()
                .downcast_ref::<Int32Array>()
                .is_some()
        );
    }

    #[test]
    fn test_dict() {
        use arrow::array::DictionaryArray;
        use arrow::datatypes::Int32Type;
        let batch =
            record_batch!(("a", (Int32, Utf8), ([0, 1, 2], ["foo", "bar", "baz"]))).unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 1);

        // Verify it's a dictionary array
        let dict_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<DictionaryArray<Int32Type>>()
            .unwrap();

        // Verify the values
        assert_eq!(dict_array.len(), 3);
    }

    #[test]
    fn test_mixed_dict_and_regular() {
        use arrow::array::{DictionaryArray, Int64Array, StringArray};
        use arrow::datatypes::UInt8Type;

        let batch = record_batch!(
            ("id", Int64, [1, 2, 3, 4]),
            (
                "status",
                (UInt8, Utf8),
                ([0, 1, 0, 2], ["active", "pending", "closed"])
            ),
            ("name", Utf8, ["alice", "bob", "charlie", "dave"])
        )
        .unwrap();

        assert_eq!(batch.num_rows(), 4);
        assert_eq!(batch.num_columns(), 3);

        // Verify the regular Int64 column
        let id_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(id_array.value(0), 1);
        assert_eq!(id_array.value(3), 4);

        // Verify the dictionary column
        let status_array = batch
            .column(1)
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        assert_eq!(status_array.len(), 4);

        // Verify the regular String column
        let name_array = batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(name_array.value(0), "alice");
        assert_eq!(name_array.value(3), "dave");
    }

    #[test]
    fn test_multiple_dict_columns() {
        use arrow::array::DictionaryArray;
        use arrow::datatypes::{Int32Type, UInt16Type};

        let batch = record_batch!(
            (
                "category",
                (UInt16, Utf8),
                ([0, 1, 0, 2], ["books", "electronics", "clothing"])
            ),
            (
                "priority",
                (Int32, Utf8),
                ([0, 1, 2, 1], ["low", "medium", "high"])
            )
        )
        .unwrap();

        assert_eq!(batch.num_rows(), 4);
        assert_eq!(batch.num_columns(), 2);

        // Verify first dictionary column
        let category_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .unwrap();
        assert_eq!(category_array.len(), 4);

        // Verify second dictionary column
        let priority_array = batch
            .column(1)
            .as_any()
            .downcast_ref::<DictionaryArray<Int32Type>>()
            .unwrap();
        assert_eq!(priority_array.len(), 4);
    }

    #[test]
    fn test_dict_with_numeric_values() {
        use arrow::array::DictionaryArray;
        use arrow::datatypes::UInt8Type;

        let batch = record_batch!((
            "price_tier",
            (UInt8, Int32),
            ([0, 1, 2, 1, 0], [100, 500, 1000])
        ))
        .unwrap();

        assert_eq!(batch.num_rows(), 5);
        assert_eq!(batch.num_columns(), 1);

        // Verify it's a dictionary array with Int32 values
        let dict_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        assert_eq!(dict_array.len(), 5);
    }

    #[test]
    fn test_complex_mixed_types() {
        use arrow::array::{BooleanArray, DictionaryArray, Float64Array, Int32Array, UInt64Array};
        use arrow::datatypes::{Int32Type, UInt16Type};

        let batch = record_batch!(
            ("id", UInt64, [100, 200, 300]),
            ("enabled", Boolean, [true, false, true]),
            ("category", (UInt16, Utf8), ([0, 1, 2], ["A", "B", "C"])),
            ("score", Float64, [95.5, 87.3, 92.1]),
            ("status_code", (Int32, Int32), ([0, 1, 0], [200, 404, 500])),
            ("count", Int32, [10, 20, 30])
        )
        .unwrap();

        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 6);

        // Verify UInt64 column
        assert!(
            batch
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .is_some()
        );

        // Verify Boolean column
        let enabled_array = batch
            .column(1)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap();
        assert!(enabled_array.value(0));
        assert!(!enabled_array.value(1));

        // Verify first dictionary column (string values)
        assert!(
            batch
                .column(2)
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                .is_some()
        );

        // Verify Float64 column
        let score_array = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(score_array.value(0), 95.5);

        // Verify second dictionary column (numeric values)
        assert!(
            batch
                .column(4)
                .as_any()
                .downcast_ref::<DictionaryArray<Int32Type>>()
                .is_some()
        );

        // Verify Int32 column
        let count_array = batch
            .column(5)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(count_array.value(2), 30);
    }

    #[test]
    fn test_predefined_variables() {
        use arrow::array::{DictionaryArray, Int32Array, StringArray, UInt32Array};
        use arrow::datatypes::UInt8Type;

        // Test with pre-defined vectors
        let int_values: Vec<i32> = vec![10, 20, 30, 40];
        let uint_values: Vec<u32> = vec![100, 200, 300, 400];
        let string_values: Vec<&str> = vec!["alice", "bob", "charlie", "dave"];

        // Dictionary data
        let dict_keys: Vec<u8> = vec![0, 1, 2, 1];
        let dict_values: Vec<&str> = vec!["red", "green", "blue"];

        let batch = record_batch!(
            ("numbers", Int32, int_values),
            ("counters", UInt32, uint_values),
            ("names", Utf8, string_values),
            ("colors", (UInt8, Utf8), (dict_keys, dict_values))
        )
        .unwrap();

        assert_eq!(batch.num_rows(), 4);
        assert_eq!(batch.num_columns(), 4);

        // Verify Int32 column
        let numbers_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(numbers_array.value(0), 10);
        assert_eq!(numbers_array.value(3), 40);

        // Verify UInt32 column
        let counters_array = batch
            .column(1)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(counters_array.value(1), 200);

        // Verify String column
        let names_array = batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(names_array.value(2), "charlie");

        // Verify Dictionary column
        let colors_array = batch
            .column(3)
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        assert_eq!(colors_array.len(), 4);
    }

    #[test]
    fn test_inline_expressions() {
        use arrow::array::{DictionaryArray, Int64Array, UInt8Array};
        use arrow::datatypes::UInt16Type;

        // Test with inline expressions - all columns must have same length (5 rows)
        let batch = record_batch!(
            ("range8", UInt8, (0..5).collect::<Vec<u8>>()),
            (
                "range64",
                Int64,
                (10..15).map(|x| x as i64).collect::<Vec<i64>>()
            ),
            (
                "computed",
                (UInt16, Int32),
                (
                    (0..5).map(|x| (x % 3) as u16).collect::<Vec<u16>>(),
                    vec![100, 200, 300]
                )
            )
        )
        .unwrap();

        assert_eq!(batch.num_rows(), 5);
        assert_eq!(batch.num_columns(), 3);

        // Verify UInt8 column with range expression
        let range8_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(range8_array.value(0), 0);
        assert_eq!(range8_array.value(4), 4);

        // Verify Int64 column with mapped expression
        let range64_array = batch
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(range64_array.value(0), 10);
        assert_eq!(range64_array.value(4), 14);

        // Verify Dictionary column with inline expressions
        let computed_array = batch
            .column(2)
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .unwrap();
        assert_eq!(computed_array.len(), 5);
    }
}
