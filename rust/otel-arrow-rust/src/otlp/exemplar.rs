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

use crate::arrays::{
    ByteArrayAccessor, NullableArrayAccessor, get_f64_array_opt, get_i64_array_opt,
    get_timestamp_nanosecond_array, get_u32_array, get_u32_array_opt,
};
use crate::error;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::metric::AppendAndGet;
use crate::proto::opentelemetry::metrics::v1::Exemplar;
use crate::proto::opentelemetry::metrics::v1::exemplar::Value;
use crate::schema::consts;
use arrow::array::RecordBatch;
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ensure};
use std::collections::HashMap;

#[derive(Default)]
pub struct ExemplarsStore {
    // This field is also not used anywhere in otel-arrow: https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/exemplar.go#L49
    #[allow(unused)]
    next_id: u32,
    exemplars_by_ids: HashMap<u32, Vec<Exemplar>>,
}

impl ExemplarsStore {
    /// Gets or creates the exemplar of given id and creates a new one if not yet created.
    pub fn get_or_create_exemplar_by_id(&mut self, id: u32) -> &mut Vec<Exemplar> {
        self.exemplars_by_ids.entry(id).or_default()
    }
}

impl ExemplarsStore {
    pub fn try_from(rb: &RecordBatch, attr_store: &mut Attribute32Store) -> error::Result<Self> {
        let mut exemplars_store = Self::default();
        let mut parent_id_decoder =
            ExemplarParentIdDecoder::new(ParentIdEncoding::ParentIdDeltaGroupEncoding);

        let id_arr_opt = get_u32_array_opt(rb, consts::ID)?;
        let int_value_arr = get_i64_array_opt(rb, consts::INT_VALUE)?;
        let double_value_arr = get_f64_array_opt(rb, consts::DOUBLE_VALUE)?;
        let parent_id_arr = get_u32_array(rb, consts::PARENT_ID)?;
        let time_unix_nano_arr = get_timestamp_nanosecond_array(rb, consts::TIME_UNIX_NANO)?;
        let span_id_arr = ByteArrayAccessor::try_new(rb.column_by_name(consts::SPAN_ID).context(
            error::ColumnNotFoundSnafu {
                name: consts::SPAN_ID,
            },
        )?)?;
        let trace_id_arr = ByteArrayAccessor::try_new(
            rb.column_by_name(consts::TRACE_ID)
                .context(error::ColumnNotFoundSnafu {
                    name: consts::TRACE_ID,
                })?,
        )?;

        for idx in 0..rb.num_rows() {
            let int_value = int_value_arr.value_at(idx);
            let double_value = double_value_arr.value_at(idx);
            let parent_id = parent_id_decoder.decode(
                parent_id_arr.value_at_or_default(idx),
                int_value,
                double_value,
            );
            let existing_exemplars = exemplars_store
                .exemplars_by_ids
                .entry(parent_id)
                .or_default();
            let current_exemplar = existing_exemplars.append_and_get();

            let id_opt = id_arr_opt.value_at(idx);

            let time_unix_nano = time_unix_nano_arr.value_at_or_default(idx);
            current_exemplar.time_unix_nano = time_unix_nano as u64;

            let span_id_bytes = span_id_arr.value_at_or_default(idx);
            ensure!(span_id_bytes.len() == 8, error::InvalidSpanIdSnafu {
                message: format!("rb: {:?}", rb),
            });
            current_exemplar.span_id = span_id_bytes;

            let trace_id_bytes = trace_id_arr.value_at_or_default(idx);
            ensure!(trace_id_bytes.len() == 16, error::InvalidTraceIdSnafu {
                message: format!("rb: {:?}", rb),
            });
            current_exemplar.trace_id = trace_id_bytes;

            match (int_value, double_value) {
                (Some(int_value), None) => {
                    current_exemplar.value = Some(Value::AsInt(int_value));
                }

                (None, Some(double_value)) => {
                    current_exemplar.value = Some(Value::AsDouble(double_value))
                }
                _ => {
                    return error::InvalidExemplarDataSnafu {
                        message: format!("record batch: {:?}", rb),
                    }
                    .fail();
                }
            }

            if let Some(id) = id_opt
                && let Some(attrs) = attr_store.attribute_by_delta_id(id)
            {
                current_exemplar.filtered_attributes = attrs.to_vec();
            }
        }

        Ok(exemplars_store)
    }
}

//todo: maybe merge with [attribute_decoder::ParentIdEncoding]
#[allow(clippy::enum_variant_names)]
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
enum ParentIdEncoding {
    /// ParentIdNoEncoding stores the parent ID as is.
    ParentIdNoEncoding = 0,
    /// ParentIdDeltaEncoding stores the parent ID as a delta from the previous
    /// parent ID.
    ParentIdDeltaEncoding = 1,
    /// ParentIdDeltaGroupEncoding stores the parent ID as a delta from the
    /// previous parent ID in the same group. A group is defined by the
    /// combination Key and Value.
    ParentIdDeltaGroupEncoding = 2,
}

#[derive(Eq, PartialEq, Debug)]
enum ExemplarValueType {
    Undefined = 0,
    Int = 1,
    Double = 2,
}

struct ExemplarParentIdDecoder {
    encoding: ParentIdEncoding,
    prev_parent_id: u32,
    prev_type: ExemplarValueType,
    prev_int_value: Option<i64>,
    prev_double_value: Option<f64>,
}

impl ExemplarParentIdDecoder {
    fn new(encoding: ParentIdEncoding) -> ExemplarParentIdDecoder {
        Self {
            encoding,
            prev_parent_id: 0,
            prev_type: ExemplarValueType::Undefined,
            prev_int_value: None,
            prev_double_value: None,
        }
    }

    fn decode(
        &mut self,
        parent_id_or_delta: u32,
        int_value: Option<i64>,
        double_value: Option<f64>,
    ) -> u32 {
        match self.encoding {
            ParentIdEncoding::ParentIdNoEncoding => parent_id_or_delta,
            ParentIdEncoding::ParentIdDeltaEncoding => {
                self.prev_parent_id += parent_id_or_delta;
                self.prev_parent_id
            }
            ParentIdEncoding::ParentIdDeltaGroupEncoding => {
                if let Some(int_value) = int_value {
                    return if self.prev_type == ExemplarValueType::Int
                        && self.prev_int_value == Some(int_value)
                    {
                        self.prev_parent_id += parent_id_or_delta;
                        self.prev_parent_id
                    } else {
                        self.prev_type = ExemplarValueType::Int;
                        self.prev_int_value = Some(int_value);
                        self.prev_double_value = None;
                        self.prev_parent_id = parent_id_or_delta;
                        self.prev_parent_id
                    };
                }
                if let Some(double_value) = double_value {
                    return if self.prev_type == ExemplarValueType::Double
                        && self.prev_double_value == Some(double_value)
                    {
                        self.prev_parent_id += parent_id_or_delta;
                        self.prev_parent_id
                    } else {
                        self.prev_type = ExemplarValueType::Double;
                        self.prev_double_value = Some(double_value);
                        self.prev_int_value = None;
                        self.prev_parent_id = parent_id_or_delta;
                        self.prev_parent_id
                    };
                }

                self.prev_parent_id += parent_id_or_delta;
                self.prev_parent_id
            }
        }
    }
}
