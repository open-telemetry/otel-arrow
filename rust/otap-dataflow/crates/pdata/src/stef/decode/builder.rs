// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! High-level OTAP Arrow record builder used by the STEF decoder.
//!
//! The stream decoder reconstructs logical resources, scopes, metrics, attributes, and points.
//! This builder turns that logical state into the direct OTAP record batches consumed by the
//! rest of the engine.

use super::super::Error;
use super::entities::{
    DecPoint, DecPointValue, DecodedAttribute, DirectDecMetric, DirectDecResource, DirectDecScope,
    RootModified,
};
use super::record_builders::{
    DirectNumberDataPointsRecordBatchBuilder, DirectNumberDpAttrsRecordBatchBuilder,
    append_decoded_attribute_with_parent,
};
use crate::OtapArrowRecords;
use crate::encode::record::{
    attributes::AttributesRecordBatchBuilder, metrics::MetricsRecordBatchBuilder,
};
use crate::otap::Metrics;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata_views::views::metrics as metrics_view;

pub(super) struct DirectOtapMetricsBuilder<'a> {
    resource_attrs: AttributesRecordBatchBuilder<u16>,
    scope_attrs: AttributesRecordBatchBuilder<u16>,
    metric_attrs: AttributesRecordBatchBuilder<u16>,
    pub(super) ndp_attrs: DirectNumberDpAttrsRecordBatchBuilder<'a>,
    metrics: MetricsRecordBatchBuilder,
    ndp: DirectNumberDataPointsRecordBatchBuilder,
    next_resource_id: u16,
    next_scope_id: u16,
    next_metric_id: u16,
    next_ndp_id: u32,
    current_resource_id: Option<u16>,
    current_scope_id: Option<u16>,
    current_metric_id: Option<u16>,
}

impl<'a> Default for DirectOtapMetricsBuilder<'a> {
    fn default() -> Self {
        Self {
            resource_attrs: AttributesRecordBatchBuilder::<u16>::new(),
            scope_attrs: AttributesRecordBatchBuilder::<u16>::new(),
            metric_attrs: AttributesRecordBatchBuilder::<u16>::new(),
            ndp_attrs: DirectNumberDpAttrsRecordBatchBuilder::default(),
            metrics: MetricsRecordBatchBuilder::new(),
            ndp: DirectNumberDataPointsRecordBatchBuilder::default(),
            next_resource_id: 0,
            next_scope_id: 0,
            next_metric_id: 0,
            next_ndp_id: 0,
            current_resource_id: None,
            current_scope_id: None,
            current_metric_id: None,
        }
    }
}

impl<'a> DirectOtapMetricsBuilder<'a> {
    pub(super) fn reserve_number_points(&mut self, additional: usize) {
        self.ndp.reserve(additional);
        self.ndp_attrs.begin_frame(additional);
    }

    pub(super) fn prepare_record(
        &mut self,
        modified: RootModified,
        resource: &DirectDecResource<'a>,
        scope: &DirectDecScope<'a>,
        metric: &DirectDecMetric<'a>,
    ) -> Result<u16, Error> {
        if self.current_resource_id.is_none() || modified.resource {
            self.start_resource(resource)?;
        }
        if self.current_scope_id.is_none() || modified.scope {
            self.start_scope(scope)?;
        }
        if self.current_metric_id.is_none() || modified.metric {
            self.start_metric(resource, scope, metric)?;
        }

        self.current_metric_id
            .ok_or(Error::InvalidFrame("metric id is initialized"))
    }

    pub(super) fn append_number_point_row(
        &mut self,
        metric_id: u16,
        point_id: u32,
        point: &DecPoint,
    ) {
        self.ndp.append_id(point_id);
        self.ndp.append_parent_id(metric_id);
        self.ndp
            .append_start_time_unix_nano(point.start_timestamp as i64);
        self.ndp.append_time_unix_nano(point.timestamp as i64);
        match point.value {
            DecPointValue::None => {
                self.ndp.append_double_value(None);
                self.ndp.append_int_value(None);
                self.ndp.append_flags(1);
            }
            DecPointValue::Int64(value) => {
                self.ndp.append_double_value(None);
                self.ndp.append_int_value(Some(value));
                self.ndp.append_flags(0);
            }
            DecPointValue::Float64(value) => {
                self.ndp.append_double_value(Some(value));
                self.ndp.append_int_value(None);
                self.ndp.append_flags(0);
            }
        }
    }

    pub(super) fn append_number_point_attrs(
        &mut self,
        point_id: u32,
        attrs: &[DecodedAttribute<'a>],
    ) {
        self.ndp_attrs.append_all(point_id, attrs);
    }

    pub(super) fn start_resource(&mut self, resource: &DirectDecResource<'a>) -> Result<(), Error> {
        let resource_id = self.allocate_resource_id()?;
        self.current_resource_id = Some(resource_id);
        self.current_scope_id = None;
        self.current_metric_id = None;

        for kv in &resource.attributes {
            append_decoded_attribute_with_parent(&mut self.resource_attrs, &resource_id, kv);
        }
        Ok(())
    }

    pub(super) fn start_scope(&mut self, scope: &DirectDecScope<'a>) -> Result<(), Error> {
        let scope_id = self.allocate_scope_id()?;
        self.current_scope_id = Some(scope_id);
        self.current_metric_id = None;

        for kv in &scope.attributes {
            append_decoded_attribute_with_parent(&mut self.scope_attrs, &scope_id, kv);
        }
        Ok(())
    }

    pub(super) fn start_metric(
        &mut self,
        resource: &DirectDecResource<'a>,
        scope: &DirectDecScope<'a>,
        metric: &DirectDecMetric<'a>,
    ) -> Result<(), Error> {
        let metric_id = self.allocate_metric_id()?;
        let resource_id = self
            .current_resource_id
            .expect("resource id is initialized before metric");
        let scope_id = self
            .current_scope_id
            .expect("scope id is initialized before metric");

        self.metrics.append_id(metric_id);
        self.metrics.resource.append_id(Some(resource_id));
        self.metrics.resource.append_schema_url(
            (!resource.schema_url.is_empty()).then_some(resource.schema_url.as_bytes()),
        );
        self.metrics
            .resource
            .append_dropped_attributes_count(resource.dropped_attributes_count);
        self.metrics.scope.append_id(Some(scope_id));
        self.metrics
            .scope
            .append_name((!scope.name.is_empty()).then_some(scope.name.as_bytes()));
        self.metrics
            .scope
            .append_version((!scope.version.is_empty()).then_some(scope.version.as_bytes()));
        self.metrics
            .scope
            .append_dropped_attributes_count(scope.dropped_attributes_count);
        self.metrics
            .append_scope_schema_url(scope.schema_url.as_bytes());
        self.metrics.append_metric_type(match metric.r#type {
            0 => metrics_view::DataType::Gauge as u8,
            1 => metrics_view::DataType::Sum as u8,
            _ => return Err(Error::UnsupportedStefValue("metric type")),
        });
        self.metrics.append_name(metric.name.as_bytes());
        self.metrics
            .append_description(metric.description.as_bytes());
        self.metrics.append_unit(metric.unit.as_bytes());
        if metric.r#type == 1 {
            self.metrics
                .append_aggregation_temporality(Some(metric.aggregation_temporality as i32));
            self.metrics.append_is_monotonic(Some(metric.monotonic));
        } else {
            self.metrics.append_aggregation_temporality(None);
            self.metrics.append_is_monotonic(None);
        }

        for kv in &metric.metadata {
            append_decoded_attribute_with_parent(&mut self.metric_attrs, &metric_id, kv);
        }

        self.current_metric_id = Some(metric_id);
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<OtapArrowRecords, Error> {
        if self.next_metric_id == 0 {
            return Ok(OtapArrowRecords::Metrics(Metrics::default()));
        }

        let mut otap_batch = OtapArrowRecords::Metrics(Metrics::default());
        let pairs = [
            (
                self.metrics
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::UnivariateMetrics,
            ),
            (
                self.metric_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::MetricAttrs,
            ),
            (
                self.resource_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::ResourceAttrs,
            ),
            (
                self.scope_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::ScopeAttrs,
            ),
            (
                self.ndp
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::NumberDataPoints,
            ),
            (
                self.ndp_attrs
                    .finish()
                    .map_err(|e| Error::OtapEncode(e.to_string()))?,
                ArrowPayloadType::NumberDpAttrs,
            ),
        ];
        for (record_batch, payload_type) in pairs {
            if record_batch.num_rows() > 0 {
                otap_batch
                    .set(payload_type, record_batch)
                    .map_err(|e| Error::OtapEncode(e.to_string()))?;
            }
        }
        Ok(otap_batch)
    }

    pub(super) fn allocate_resource_id(&mut self) -> Result<u16, Error> {
        let id = self.next_resource_id;
        self.next_resource_id = self
            .next_resource_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("resource id"))?;
        Ok(id)
    }

    pub(super) fn allocate_scope_id(&mut self) -> Result<u16, Error> {
        let id = self.next_scope_id;
        self.next_scope_id = self
            .next_scope_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("scope id"))?;
        Ok(id)
    }

    pub(super) fn allocate_metric_id(&mut self) -> Result<u16, Error> {
        let id = self.next_metric_id;
        self.next_metric_id = self
            .next_metric_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("metric id"))?;
        Ok(id)
    }

    pub(super) fn allocate_number_point_id(&mut self) -> Result<u32, Error> {
        let id = self.next_ndp_id;
        self.next_ndp_id = self
            .next_ndp_id
            .checked_add(1)
            .ok_or(Error::ValueOutOfRange("number data point id"))?;
        Ok(id)
    }
}
