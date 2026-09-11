// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::BooleanArray;
use arrow::compute::filter_record_batch;
use arrow::datatypes::UInt32Type;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::Metrics;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmap, filter_child_batch};

use crate::error::Result;
use crate::pipeline::expr::types::MetricDatapointType;

// TODO - in the parent module we use "datapoint" and here we use "data_point"

pub fn filter_metric_datapoints(
    otap_batch: &mut OtapArrowRecords,
    datapoint_type: &MetricDatapointType,
    data_point_selection_vec: &BooleanArray,
    id_bitmap: &mut IdBitmap,
) -> Result<()> {
    let Some(data_point_batch) = otap_batch.get(datapoint_type.payload_type()) else {
        // hmmm --- that's weird
        todo!("remove all the child batches I guess?") // it's not really our res
    };

    let filtered_data_point_batch =
        filter_record_batch(data_point_batch, data_point_selection_vec)?;
    // TODO - if there are no rows, remove all the children I guess

    otap_batch.set(datapoint_type.payload_type(), filtered_data_point_batch)?;

    // TODO comment about what we're doing here
    let mut tmp = OtapArrowRecords::Metrics(Metrics::default());
    shuffle_metric_dp_child_batches(datapoint_type, otap_batch, &mut tmp)?;

    filter_child_batch::<UInt32Type>(
        &tmp,
        otap_batch,
        datapoint_type.dp_attrs_payload_type(),
        id_bitmap,
    )?;

    if let Some(payload_type) = datapoint_type.exemplar_payload_type() {
        filter_child_batch::<UInt32Type>(&tmp, otap_batch, payload_type, id_bitmap)?;
    }
    if let Some(payload_type) = datapoint_type.exemplar_attr_payload_type() {
        filter_child_batch::<UInt32Type>(&tmp, otap_batch, payload_type, id_bitmap)?;
    }

    Ok(())
}

fn shuffle_metric_dp_child_batches(
    data_point_type: &MetricDatapointType,
    from: &mut OtapArrowRecords,
    to: &mut OtapArrowRecords,
) -> Result<()> {
    if let Some(exemplar_attr_payload_type) = data_point_type.exemplar_attr_payload_type() {
        if let Some(batch) = from.remove(exemplar_attr_payload_type) {
            to.set(exemplar_attr_payload_type, batch)?;
        }
    }

    if let Some(exemplar_payload_type) = data_point_type.exemplar_payload_type() {
        if let Some(batch) = from.remove(exemplar_payload_type) {
            to.set(exemplar_payload_type, batch)?;
        }
    }

    if let Some(batch) = from.remove(data_point_type.dp_attrs_payload_type()) {
        to.set(data_point_type.dp_attrs_payload_type(), batch)?
    }

    Ok(())
}

/// removes all the metric datapoints of the given type and any associated child records such as
/// attributes, exemplars, and exemplar attributes.
pub fn remove_all_metric_data_points(
    otap_batch: &mut OtapArrowRecords,
    datapoint_type: &MetricDatapointType,
) {
    if let Some(exemplar_attr_payload_type) = datapoint_type.exemplar_attr_payload_type() {
        _ = otap_batch.remove(exemplar_attr_payload_type);
    }

    if let Some(exemplar_payload_type) = datapoint_type.exemplar_payload_type() {
        _ = otap_batch.remove(exemplar_payload_type)
    }

    _ = otap_batch.remove(datapoint_type.dp_attrs_payload_type());
    _ = otap_batch.remove(datapoint_type.payload_type());
}
