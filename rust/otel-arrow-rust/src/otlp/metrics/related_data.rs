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

use crate::error;
use crate::otap::OtapBatch;
use crate::otlp::attributes::store::{Attribute16Store, Attribute32Store};
use crate::otlp::metrics::data_points::data_point_store::{
    EHistogramDataPointsStore, HistogramDataPointsStore, NumberDataPointsStore,
    SummaryDataPointsStore,
};
use crate::otlp::metrics::exemplar::ExemplarsStore;
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
}

impl TryFrom<&OtapBatch> for RelatedData {
    type Error = error::Error;

    fn try_from(otap_batch: &OtapBatch) -> error::Result<Self> {
        let mut related_data = RelatedData::default();

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ResourceAttrs) {
            related_data.res_attr_map_store = Attribute16Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ScopeAttrs) {
            related_data.scope_attr_map_store = Attribute16Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::NumberDpExemplarAttrs) {
            related_data.number_d_p_exemplar_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::NumberDpExemplars) {
            related_data.number_data_point_exemplars_store =
                ExemplarsStore::try_from(rb, &mut related_data.number_d_p_exemplar_attrs_store)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::NumberDpAttrs) {
            related_data.number_d_p_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::NumberDataPoints) {
            related_data.number_data_points_store = NumberDataPointsStore::from_record_batch(
                rb,
                &mut related_data.number_data_point_exemplars_store,
                &related_data.number_d_p_attrs_store,
            )?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::SummaryDpAttrs) {
            related_data.summary_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::SummaryDataPoints) {
            related_data.summary_data_points_store = SummaryDataPointsStore::from_record_batch(
                rb,
                &mut related_data.summary_attrs_store,
            )?
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::HistogramDpExemplarAttrs) {
            related_data.histogram_exemplar_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::HistogramDpExemplars) {
            related_data.histogram_data_point_exemplars_store =
                ExemplarsStore::try_from(rb, &mut related_data.histogram_exemplar_attrs_store)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::HistogramDataPoints) {
            related_data.histogram_data_points_store = HistogramDataPointsStore::from_record_batch(
                rb,
                &mut related_data.histogram_data_point_exemplars_store,
                &related_data.histogram_attrs_store,
            )?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::HistogramDpAttrs) {
            related_data.histogram_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ExpHistogramDpAttrs) {
            related_data.exp_histogram_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ExpHistogramDpExemplarAttrs) {
            related_data.exp_histogram_exemplar_attrs_store = Attribute32Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ExpHistogramDpExemplars) {
            related_data.e_histogram_data_point_exemplars_store =
                ExemplarsStore::try_from(rb, &mut related_data.exp_histogram_exemplar_attrs_store)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ExpHistogramDataPoints) {
            related_data.e_histogram_data_points_store =
                EHistogramDataPointsStore::from_record_batch(
                    rb,
                    &mut related_data.e_histogram_data_point_exemplars_store,
                    &related_data.exp_histogram_attrs_store,
                )?;
        }

        Ok(related_data)
    }
}
