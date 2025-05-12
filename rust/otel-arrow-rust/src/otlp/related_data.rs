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

use crate::decode::record_message::RecordMessage;
use crate::error;
use crate::otlp::attributes::store::{Attribute16Store, Attribute32Store};
use crate::otlp::data_points::data_point_store::{
    EHistogramDataPointsStore, HistogramDataPointsStore, NumberDataPointsStore,
    SummaryDataPointsStore,
};
use crate::otlp::exemplar::ExemplarsStore;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

#[derive(Default)]
pub struct RelatedData {
    pub(crate) metric_id: u16,

    // Resource attributes.
    pub(crate) res_attr_map_store: Attribute16Store,
    pub(crate) scope_attr_map_store: Attribute16Store,
    pub(crate) number_d_p_attrs_store: Attribute32Store,
    pub(crate) summary_attrs_store: Attribute32Store,
    pub(crate) histogram_attrs_store: Attribute32Store,
    pub(crate) exp_histogram_attrs_store: Attribute32Store,

    // Exemplars
    pub(crate) number_data_point_exemplars_store: ExemplarsStore,
    pub(crate) histogram_data_point_exemplars_store: ExemplarsStore,
    pub(crate) e_histogram_data_point_exemplars_store: ExemplarsStore,

    // Exemplar attributes store
    pub(crate) number_d_p_exemplar_attrs_store: Attribute32Store,
    pub(crate) histogram_exemplar_attrs_store: Attribute32Store,
    pub(crate) exp_histogram_exemplar_attrs_store: Attribute32Store,

    // Data points
    pub(crate) number_data_points_store: NumberDataPointsStore,
    pub(crate) summary_data_points_store: SummaryDataPointsStore,
    pub(crate) histogram_data_points_store: HistogramDataPointsStore,
    pub(crate) e_histogram_data_points_store: EHistogramDataPointsStore,
}

impl RelatedData {
    pub fn metric_id_from_delta(&mut self, delta: u16) -> u16 {
        self.metric_id += delta;
        self.metric_id
    }

    pub fn from_record_messages(
        rbs: &[RecordMessage],
    ) -> error::Result<(RelatedData, Option<usize>)> {
        let mut related_data = RelatedData::default();

        // index for main metrics record.
        let mut metrics_record_idx: Option<usize> = None;

        let mut number_dp_idx: Option<usize> = None;
        let mut summary_dp_idx: Option<usize> = None;
        let mut histogram_dp_idx: Option<usize> = None;
        let mut exp_histogram_dp_idx: Option<usize> = None;
        let mut number_dp_ex_idx: Option<usize> = None;
        let mut histogram_dp_ex_idx: Option<usize> = None;
        let mut exp_histogram_dp_ex_idx: Option<usize> = None;

        for (idx, rm) in rbs.iter().enumerate() {
            match rm.payload_type {
                ArrowPayloadType::ResourceAttrs => {
                    related_data.res_attr_map_store = Attribute16Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::ScopeAttrs => {
                    related_data.scope_attr_map_store = Attribute16Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::UnivariateMetrics => {
                    // this record is the main metrics record.
                    metrics_record_idx = Some(idx);
                }
                ArrowPayloadType::NumberDataPoints => {
                    number_dp_idx = Some(idx);
                }
                ArrowPayloadType::SummaryDataPoints => {
                    summary_dp_idx = Some(idx);
                }
                ArrowPayloadType::HistogramDataPoints => {
                    histogram_dp_idx = Some(idx);
                }
                ArrowPayloadType::ExpHistogramDataPoints => {
                    exp_histogram_dp_idx = Some(idx);
                }
                ArrowPayloadType::NumberDpAttrs => {
                    related_data.number_d_p_attrs_store = Attribute32Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::SummaryDpAttrs => {
                    related_data.summary_attrs_store = Attribute32Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::HistogramDpAttrs => {
                    related_data.histogram_attrs_store = Attribute32Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::ExpHistogramDpAttrs => {
                    related_data.exp_histogram_attrs_store =
                        Attribute32Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::NumberDpExemplars => {
                    number_dp_ex_idx = Some(idx);
                }
                ArrowPayloadType::HistogramDpExemplars => {
                    histogram_dp_ex_idx = Some(idx);
                }
                ArrowPayloadType::ExpHistogramDpExemplars => {
                    exp_histogram_dp_ex_idx = Some(idx);
                }
                ArrowPayloadType::NumberDpExemplarAttrs => {
                    related_data.number_d_p_exemplar_attrs_store =
                        Attribute32Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::HistogramDpExemplarAttrs => {
                    related_data.histogram_exemplar_attrs_store =
                        Attribute32Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::ExpHistogramDpExemplarAttrs => {
                    related_data.exp_histogram_exemplar_attrs_store =
                        Attribute32Store::try_from(&rm.record)?;
                }
                _ => {
                    //todo: support logs/trace/span
                    return error::UnsupportedPayloadTypeSnafu {
                        actual: rm.payload_type,
                    }
                    .fail();
                }
            }
        }

        // Process exemplars.
        if let Some(number_dp_ex_rec_idx) = number_dp_ex_idx {
            let record = &rbs[number_dp_ex_rec_idx].record;
            related_data.number_data_point_exemplars_store = ExemplarsStore::try_from(
                record,
                &mut related_data.number_d_p_exemplar_attrs_store,
            )?;
        }

        if let Some(histogram_dp_ex_rec_idx) = histogram_dp_ex_idx {
            let record = &rbs[histogram_dp_ex_rec_idx].record;
            related_data.histogram_data_point_exemplars_store =
                ExemplarsStore::try_from(record, &mut related_data.histogram_exemplar_attrs_store)?
        }

        if let Some(exp_histogram_dp_ex_rec_idx) = exp_histogram_dp_ex_idx {
            let record = &rbs[exp_histogram_dp_ex_rec_idx].record;
            related_data.e_histogram_data_point_exemplars_store = ExemplarsStore::try_from(
                record,
                &mut related_data.exp_histogram_exemplar_attrs_store,
            )?;
        }

        // Process data points
        if let Some(number_data_point_record_idx) = number_dp_idx {
            let number_data_point_record = &rbs[number_data_point_record_idx];
            related_data.number_data_points_store = NumberDataPointsStore::from_record_batch(
                &number_data_point_record.record,
                &mut related_data.number_data_point_exemplars_store,
                &related_data.number_d_p_attrs_store,
            )?;
        }
        if let Some(summary_data_point_rec_idx) = summary_dp_idx {
            let record = &rbs[summary_data_point_rec_idx].record;
            related_data.summary_data_points_store = SummaryDataPointsStore::from_record_batch(
                record,
                &mut related_data.summary_attrs_store,
            )?;
        }
        if let Some(histogram_dp_idx) = histogram_dp_idx {
            let record = &rbs[histogram_dp_idx].record;
            related_data.histogram_data_points_store = HistogramDataPointsStore::from_record_batch(
                record,
                &mut related_data.histogram_data_point_exemplars_store,
                &related_data.histogram_attrs_store,
            )?;
        }

        if let Some(exp_histogram_data_point_idx) = exp_histogram_dp_idx {
            let record = &rbs[exp_histogram_data_point_idx].record;
            related_data.e_histogram_data_points_store =
                EHistogramDataPointsStore::from_record_batch(
                    record,
                    &mut related_data.e_histogram_data_point_exemplars_store,
                    &related_data.exp_histogram_attrs_store,
                )?;
        }

        Ok((related_data, metrics_record_idx))
    }
}
