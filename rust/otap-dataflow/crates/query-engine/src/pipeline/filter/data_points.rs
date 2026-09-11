// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for applying filters to datapoints

use arrow::array::BooleanArray;
use arrow::compute::filter_record_batch;
use arrow::datatypes::UInt32Type;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::Metrics;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmap, filter_child_batch};

use crate::error::Result;
use crate::pipeline::expr::types::MetricDatapointType;

/// Filter the datapoints record batch, identified by the datapoint type, by keeping only the
/// rows in positions identified by the selection vector.
///
/// This function also handles automatically filtering the child record batches (attributes,
/// exemplars, etc.) of these datapoint types to remove any orphaned children rows (e.g.
/// remove any attributes/exemplars associated with  discarded datapoints).
///
/// If after discarding the filtered rows some record batch turns out to be empty, it will
/// be removed entirely from the OTAP batch.
pub fn filter_metric_data_points(
    otap_batch: &mut OtapArrowRecords,
    datapoint_type: &MetricDatapointType,
    data_point_selection_vec: &BooleanArray,
    id_bitmap: &mut IdBitmap,
) -> Result<()> {
    let Some(data_point_batch) = otap_batch.get(datapoint_type.payload_type()) else {
        // nothing to do
        return Ok(());
    };

    if data_point_selection_vec.true_count() == 0 {
        // discard everything
        remove_all_metric_data_points(otap_batch, datapoint_type);
        return Ok(());
    }

    // filter and replace datapoint record batch
    let filtered_data_point_batch =
        filter_record_batch(data_point_batch, data_point_selection_vec)?;
    otap_batch.set(datapoint_type.payload_type(), filtered_data_point_batch)?;

    // filter the child record batches...
    //
    // We're going to use the `filter_child_batch` utility which expects to take the child batch
    // from some input, filter it by its parent, and append it to some new output batch. Since we
    // want to modify the OTAP batch in place, we temporarily swap the child record batch being
    // filtered to this tmp OTAP batch to be used as the input
    let mut tmp = OtapArrowRecords::Metrics(Metrics::default());
    if let Some(batch) = otap_batch.remove(datapoint_type.dp_attrs_payload_type()) {
        tmp.set(datapoint_type.dp_attrs_payload_type(), batch)?
    }
    if let Some(exemplar_payload_type) = datapoint_type.exemplar_payload_type()
        && let Some(batch) = otap_batch.remove(exemplar_payload_type)
    {
        tmp.set(exemplar_payload_type, batch)?;
    }
    if let Some(exemplar_attr_payload_type) = datapoint_type.exemplar_attr_payload_type()
        && let Some(batch) = otap_batch.remove(exemplar_attr_payload_type)
    {
        tmp.set(exemplar_attr_payload_type, batch)?;
    }

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
