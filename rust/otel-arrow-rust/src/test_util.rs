// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use arrow::array::{
    ArrayRef, BinaryArray, BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array,
    Int64Array, Int8Array, RecordBatch, StringArray, TimestampMicrosecondArray,
    TimestampMillisecondArray, TimestampNanosecondArray, TimestampSecondArray, UInt16Array,
    UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;
use std::sync::Arc;

pub(crate) fn create_test_schema() -> Schema {
    Schema::new(vec![
        Field::new("a", DataType::UInt16, true),
        Field::new("b", DataType::Utf8, true),
        Field::new("c", DataType::Float64, true),
    ])
}

pub(crate) fn create_record_batch(schema: SchemaRef, num_rows: usize) -> RecordBatch {
    let columns = schema
        .fields
        .iter()
        .map(|f| create_array(f.data_type(), num_rows))
        .collect::<Vec<_>>();
    RecordBatch::try_new(schema, columns).unwrap()
}

pub(crate) fn create_array(dt: &DataType, num_rows: usize) -> ArrayRef {
    let mut r = rand::thread_rng();
    match dt {
        DataType::Boolean => Arc::new(
            (0..num_rows)
                .map(|_| Some(r.gen_bool(1.0 / 2.0)))
                .collect::<BooleanArray>(),
        ) as Arc<_>,
        DataType::Int8 => {
            Arc::new(Int8Array::from_iter((0..num_rows).map(|_| r.gen::<i8>()))) as Arc<_>
        }
        DataType::Int16 => {
            Arc::new(Int16Array::from_iter((0..num_rows).map(|_| r.gen::<i16>()))) as Arc<_>
        }
        DataType::Int32 => {
            Arc::new(Int32Array::from_iter((0..num_rows).map(|_| r.gen::<i32>()))) as Arc<_>
        }
        DataType::Int64 => {
            Arc::new(Int64Array::from_iter((0..num_rows).map(|_| r.gen::<i64>()))) as Arc<_>
        }
        DataType::UInt8 => {
            Arc::new(UInt8Array::from_iter((0..num_rows).map(|_| r.gen::<u8>()))) as Arc<_>
        }
        DataType::UInt16 => Arc::new(UInt16Array::from_iter(
            (0..num_rows).map(|_| r.gen::<u16>()),
        )) as Arc<_>,
        DataType::UInt32 => Arc::new(UInt32Array::from_iter(
            (0..num_rows).map(|_| r.gen::<u32>()),
        )) as Arc<_>,
        DataType::UInt64 => Arc::new(UInt64Array::from_iter(
            (0..num_rows).map(|_| r.gen::<u64>()),
        )) as Arc<_>,
        DataType::Float32 => Arc::new(Float32Array::from_iter(
            (0..num_rows).map(|_| r.gen::<f32>()),
        )) as Arc<_>,
        DataType::Float64 => Arc::new(Float64Array::from_iter(
            (0..num_rows).map(|_| r.gen::<f64>()),
        )) as Arc<_>,
        DataType::Timestamp(unit, _) => match unit {
            TimeUnit::Second => Arc::new(TimestampSecondArray::from_iter(&Int64Array::from_iter(
                (0..num_rows).map(|_| r.gen::<i64>()),
            ))) as Arc<_>,
            TimeUnit::Millisecond => Arc::new(TimestampMillisecondArray::from_iter(
                &Int64Array::from_iter((0..num_rows).map(|_| r.gen::<i64>())),
            )) as Arc<_>,
            TimeUnit::Microsecond => Arc::new(TimestampMicrosecondArray::from_iter(
                &Int64Array::from_iter((0..num_rows).map(|_| r.gen::<i64>())),
            )) as Arc<_>,

            TimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from_iter(
                &Int64Array::from_iter((0..num_rows).map(|_| r.gen::<i64>())),
            )) as Arc<_>,
        },
        DataType::Binary | DataType::LargeBinary => Arc::new(BinaryArray::from_iter(
            (0..num_rows).map(|_| Some(Alphanumeric.sample_string(&mut r, 10))),
        )) as Arc<_>,
        DataType::Utf8 => Arc::new(StringArray::from_iter(
            (0..num_rows).map(|_| Some(Alphanumeric.sample_string(&mut r, 10))),
        )) as Arc<_>,
        _ => {
            unimplemented!()
        }
    }
}
