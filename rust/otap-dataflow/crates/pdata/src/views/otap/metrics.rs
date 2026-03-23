// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Zero-copy view implementation for OTAP Arrow RecordBatches (Metrics)

use std::collections::BTreeMap;

#[allow(unused_imports)]
use arrow::array::Array;

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays};
use crate::otlp::common::{ResourceArrays, ScopeArrays};
use crate::otlp::metrics::data_points::exp_histogram::{
    ExpHistogramDpArrays, PositiveNegativeArrayAccess,
};
use crate::otlp::metrics::data_points::histogram::HistogramDpArrays;
use crate::otlp::metrics::data_points::number::NumberDpArrays;
use crate::otlp::metrics::data_points::summary::{QuantileArrays, SummaryDpArrays};
use crate::otlp::metrics::exemplar::ExemplarArrays;
use crate::otlp::metrics::{MetricType, MetricsArrays};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::{SpanId, TraceId};
use crate::views::otap::common::{
    Otap32AttributeIter, OtapAttributeIter, OtapAttributeView, RowGroup, RowGroupIter,
    build_attribute_index, group_by_resource_id, group_by_scope_id,
};
use otap_df_pdata_views::views::common::{InstrumentationScopeView, Str};
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, BucketsView, DataPointFlags, DataType, DataView, ExemplarView,
    ExponentialHistogramDataPointView, ExponentialHistogramView, GaugeView, HistogramDataPointView,
    HistogramView, MetricView, MetricsView, NumberDataPointView, ResourceMetricsView,
    ScopeMetricsView, SumView, SummaryDataPointView, SummaryView, Value, ValueAtQuantileView,
};
use otap_df_pdata_views::views::resource::ResourceView;

// ===== Index Helpers =====

fn build_u16_parent_index(parent_id: &arrow::array::UInt16Array) -> BTreeMap<u16, Vec<usize>> {
    let mut index: BTreeMap<u16, Vec<usize>> = BTreeMap::new();
    for i in 0..parent_id.len() {
        if parent_id.is_valid(i) {
            index.entry(parent_id.value(i)).or_default().push(i);
        }
    }
    index
}

fn build_u32_parent_index(
    parent_id: &MaybeDictArrayAccessor<'_, arrow::array::UInt32Array>,
) -> BTreeMap<u32, Vec<usize>> {
    let mut index: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for i in 0..parent_id.len() {
        if let Some(pid) = parent_id.value_at(i) {
            index.entry(pid).or_default().push(i);
        }
    }
    index
}

fn build_attr32_index(attrs: &Attribute32Arrays<'_>) -> BTreeMap<u32, Vec<usize>> {
    build_u32_parent_index(&attrs.parent_id)
}

// ===== Main View =====

/// Zero-copy view over OTAP metrics Arrow RecordBatches.
pub struct OtapMetricsView<'a> {
    metrics_arrays: MetricsArrays<'a>,
    resource_columns: ResourceArrays<'a>,
    scope_columns: ScopeArrays<'a>,

    resource_attrs: Option<Attribute16Arrays<'a>>,
    scope_attrs: Option<Attribute16Arrays<'a>>,
    metric_attrs: Option<Attribute16Arrays<'a>>,

    number_dp_arrays: Option<NumberDpArrays<'a>>,
    number_dp_attrs: Option<Attribute32Arrays<'a>>,
    number_dp_exemplar_arrays: Option<ExemplarArrays<'a>>,
    number_dp_exemplar_attrs: Option<Attribute32Arrays<'a>>,

    hist_dp_arrays: Option<HistogramDpArrays<'a>>,
    hist_dp_attrs: Option<Attribute32Arrays<'a>>,
    hist_dp_exemplar_arrays: Option<ExemplarArrays<'a>>,
    hist_dp_exemplar_attrs: Option<Attribute32Arrays<'a>>,

    exp_hist_dp_arrays: Option<ExpHistogramDpArrays<'a>>,
    exp_hist_dp_attrs: Option<Attribute32Arrays<'a>>,
    exp_hist_dp_exemplar_arrays: Option<ExemplarArrays<'a>>,
    exp_hist_dp_exemplar_attrs: Option<Attribute32Arrays<'a>>,

    summary_dp_arrays: Option<SummaryDpArrays<'a>>,
    summary_dp_attrs: Option<Attribute32Arrays<'a>>,

    // Pre-computed hierarchy indices
    resource_groups: Vec<(u16, RowGroup)>,
    scope_groups_map: BTreeMap<u16, Vec<(u16, RowGroup)>>,
    resource_attrs_map: BTreeMap<u16, Vec<usize>>,
    scope_attrs_map: BTreeMap<u16, Vec<usize>>,
    metric_attrs_map: BTreeMap<u16, Vec<usize>>,

    // Data-point parent indexes: metric_id (u16) -> [dp_row_idx]
    number_dp_index: BTreeMap<u16, Vec<usize>>,
    hist_dp_index: BTreeMap<u16, Vec<usize>>,
    exp_hist_dp_index: BTreeMap<u16, Vec<usize>>,
    summary_dp_index: BTreeMap<u16, Vec<usize>>,

    // Data-point attribute indexes: dp_id (u32) -> [attr_row_idx]
    number_dp_attrs_index: BTreeMap<u32, Vec<usize>>,
    hist_dp_attrs_index: BTreeMap<u32, Vec<usize>>,
    exp_hist_dp_attrs_index: BTreeMap<u32, Vec<usize>>,
    summary_dp_attrs_index: BTreeMap<u32, Vec<usize>>,

    // Exemplar parent indexes: dp_id (u32) -> [exemplar_row_idx]
    number_exemplar_index: BTreeMap<u32, Vec<usize>>,
    hist_exemplar_index: BTreeMap<u32, Vec<usize>>,
    exp_hist_exemplar_index: BTreeMap<u32, Vec<usize>>,

    // Exemplar attribute indexes: exemplar_id (u32) -> [attr_row_idx]
    number_exemplar_attrs_index: BTreeMap<u32, Vec<usize>>,
    hist_exemplar_attrs_index: BTreeMap<u32, Vec<usize>>,
    exp_hist_exemplar_attrs_index: BTreeMap<u32, Vec<usize>>,
}

impl<'a> TryFrom<&'a OtapArrowRecords> for OtapMetricsView<'a> {
    type Error = Error;

    fn try_from(records: &'a OtapArrowRecords) -> Result<Self, Self::Error> {
        let metrics_batch = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .or_else(|| records.get(ArrowPayloadType::MultivariateMetrics))
            .ok_or(Error::MetricRecordNotFound)?;

        let metrics_arrays = MetricsArrays::try_from(metrics_batch)?;
        let resource_columns = ResourceArrays::try_from(metrics_batch)?;
        let scope_columns = ScopeArrays::try_from(metrics_batch)?;

        let resource_attrs_batch = records.get(ArrowPayloadType::ResourceAttrs);
        let scope_attrs_batch = records.get(ArrowPayloadType::ScopeAttrs);
        let metric_attrs_batch = records.get(ArrowPayloadType::MetricAttrs);

        // Resource/scope hierarchy
        let resource_groups = group_by_resource_id(metrics_batch);
        let mut scope_groups_map = BTreeMap::new();
        for (resource_id, row_indices) in &resource_groups {
            let scope_groups = group_by_scope_id(metrics_batch, row_indices);
            let _ = scope_groups_map.insert(*resource_id, scope_groups);
        }

        // Attribute indexes (u16 parent)
        let resource_attrs_map = resource_attrs_batch
            .map(build_attribute_index)
            .unwrap_or_default();
        let scope_attrs_map = scope_attrs_batch
            .map(build_attribute_index)
            .unwrap_or_default();
        let metric_attrs_map = metric_attrs_batch
            .map(build_attribute_index)
            .unwrap_or_default();

        // Data point arrays
        let number_dp_arrays = records
            .get(ArrowPayloadType::NumberDataPoints)
            .map(NumberDpArrays::try_from)
            .transpose()?;
        let number_dp_attrs = records
            .get(ArrowPayloadType::NumberDpAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;
        let number_dp_exemplar_arrays = records
            .get(ArrowPayloadType::NumberDpExemplars)
            .map(ExemplarArrays::try_from)
            .transpose()?;
        let number_dp_exemplar_attrs = records
            .get(ArrowPayloadType::NumberDpExemplarAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;

        let hist_dp_arrays = records
            .get(ArrowPayloadType::HistogramDataPoints)
            .map(HistogramDpArrays::try_from)
            .transpose()?;
        let hist_dp_attrs = records
            .get(ArrowPayloadType::HistogramDpAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;
        let hist_dp_exemplar_arrays = records
            .get(ArrowPayloadType::HistogramDpExemplars)
            .map(ExemplarArrays::try_from)
            .transpose()?;
        let hist_dp_exemplar_attrs = records
            .get(ArrowPayloadType::HistogramDpExemplarAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;

        let exp_hist_dp_arrays = records
            .get(ArrowPayloadType::ExpHistogramDataPoints)
            .map(ExpHistogramDpArrays::try_from)
            .transpose()?;
        let exp_hist_dp_attrs = records
            .get(ArrowPayloadType::ExpHistogramDpAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;
        let exp_hist_dp_exemplar_arrays = records
            .get(ArrowPayloadType::ExpHistogramDpExemplars)
            .map(ExemplarArrays::try_from)
            .transpose()?;
        let exp_hist_dp_exemplar_attrs = records
            .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;

        let summary_dp_arrays = records
            .get(ArrowPayloadType::SummaryDataPoints)
            .map(SummaryDpArrays::try_from)
            .transpose()?;
        let summary_dp_attrs = records
            .get(ArrowPayloadType::SummaryDpAttrs)
            .map(Attribute32Arrays::try_from)
            .transpose()?;

        // Data point parent indexes
        let number_dp_index = number_dp_arrays
            .as_ref()
            .map(|a| build_u16_parent_index(a.parent_id))
            .unwrap_or_default();
        let hist_dp_index = hist_dp_arrays
            .as_ref()
            .map(|a| build_u16_parent_index(a.parent_id))
            .unwrap_or_default();
        let exp_hist_dp_index = exp_hist_dp_arrays
            .as_ref()
            .map(|a| build_u16_parent_index(a.parent_id))
            .unwrap_or_default();
        let summary_dp_index = summary_dp_arrays
            .as_ref()
            .map(|a| build_u16_parent_index(a.parent_id))
            .unwrap_or_default();

        // Data point attribute indexes
        let number_dp_attrs_index = number_dp_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();
        let hist_dp_attrs_index = hist_dp_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();
        let exp_hist_dp_attrs_index = exp_hist_dp_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();
        let summary_dp_attrs_index = summary_dp_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();

        // Exemplar parent indexes
        let number_exemplar_index = number_dp_exemplar_arrays
            .as_ref()
            .map(|a| build_u32_parent_index(&a.parent_id))
            .unwrap_or_default();
        let hist_exemplar_index = hist_dp_exemplar_arrays
            .as_ref()
            .map(|a| build_u32_parent_index(&a.parent_id))
            .unwrap_or_default();
        let exp_hist_exemplar_index = exp_hist_dp_exemplar_arrays
            .as_ref()
            .map(|a| build_u32_parent_index(&a.parent_id))
            .unwrap_or_default();

        // Exemplar attribute indexes
        let number_exemplar_attrs_index = number_dp_exemplar_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();
        let hist_exemplar_attrs_index = hist_dp_exemplar_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();
        let exp_hist_exemplar_attrs_index = exp_hist_dp_exemplar_attrs
            .as_ref()
            .map(build_attr32_index)
            .unwrap_or_default();

        Ok(Self {
            metrics_arrays,
            resource_columns,
            scope_columns,
            resource_attrs: resource_attrs_batch
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            scope_attrs: scope_attrs_batch
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            metric_attrs: metric_attrs_batch
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            number_dp_arrays,
            number_dp_attrs,
            number_dp_exemplar_arrays,
            number_dp_exemplar_attrs,
            hist_dp_arrays,
            hist_dp_attrs,
            hist_dp_exemplar_arrays,
            hist_dp_exemplar_attrs,
            exp_hist_dp_arrays,
            exp_hist_dp_attrs,
            exp_hist_dp_exemplar_arrays,
            exp_hist_dp_exemplar_attrs,
            summary_dp_arrays,
            summary_dp_attrs,
            resource_groups,
            scope_groups_map,
            resource_attrs_map,
            scope_attrs_map,
            metric_attrs_map,
            number_dp_index,
            hist_dp_index,
            exp_hist_dp_index,
            summary_dp_index,
            number_dp_attrs_index,
            hist_dp_attrs_index,
            exp_hist_dp_attrs_index,
            summary_dp_attrs_index,
            number_exemplar_index,
            hist_exemplar_index,
            exp_hist_exemplar_index,
            number_exemplar_attrs_index,
            hist_exemplar_attrs_index,
            exp_hist_exemplar_attrs_index,
        })
    }
}

// ===== MetricsView impl =====

impl<'a> MetricsView for OtapMetricsView<'a> {
    type ResourceMetrics<'res>
        = OtapResourceMetricsView<'res>
    where
        Self: 'res;
    type ResourceMetricsIter<'res>
        = OtapResourceMetricsIter<'res>
    where
        Self: 'res;

    #[inline]
    fn resources(&self) -> <OtapMetricsView<'a> as MetricsView>::ResourceMetricsIter<'_> {
        OtapResourceMetricsIter {
            view: self,
            iter: self.resource_groups.iter(),
        }
    }
}

// ===== Resource Metrics =====

pub struct OtapResourceMetricsIter<'a> {
    view: &'a OtapMetricsView<'a>,
    iter: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> Iterator for OtapResourceMetricsIter<'a> {
    type Item = OtapResourceMetricsView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapResourceMetricsIter<'a> as Iterator>::Item> {
        let (resource_id, row_indices) = self.iter.next()?;
        Some(OtapResourceMetricsView {
            view: self.view,
            resource_id: *resource_id,
            row_indices,
        })
    }
}

pub struct OtapResourceMetricsView<'a> {
    view: &'a OtapMetricsView<'a>,
    resource_id: u16,
    row_indices: &'a RowGroup,
}

impl<'a> ResourceMetricsView for OtapResourceMetricsView<'a> {
    type Resource<'res>
        = OtapMetricsResourceView<'res>
    where
        Self: 'res;
    type ScopeMetrics<'scp>
        = OtapScopeMetricsView<'scp>
    where
        Self: 'scp;
    type ScopesIter<'scp>
        = OtapScopeMetricsIter<'scp>
    where
        Self: 'scp;

    #[inline]
    fn resource(
        &self,
    ) -> Option<<OtapResourceMetricsView<'a> as ResourceMetricsView>::Resource<'_>> {
        Some(OtapMetricsResourceView {
            view: self.view,
            resource_id: self.resource_id,
        })
    }

    #[inline]
    fn scopes(&self) -> <OtapResourceMetricsView<'a> as ResourceMetricsView>::ScopesIter<'_> {
        let scope_groups = self
            .view
            .scope_groups_map
            .get(&self.resource_id)
            .map(|v| v.iter())
            .unwrap_or_default();
        OtapScopeMetricsIter {
            view: self.view,
            iter: scope_groups,
        }
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        let first_row = self.row_indices.iter().next()?;
        self.view
            .resource_columns
            .schema_url
            .as_ref()
            .and_then(|c| c.str_at(first_row))
            .map(|s| s.as_bytes())
    }
}

// ===== Resource View (metrics-specific) =====

pub struct OtapMetricsResourceView<'a> {
    view: &'a OtapMetricsView<'a>,
    resource_id: u16,
}

impl<'a> ResourceView for OtapMetricsResourceView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributesIter<'att>
        = OtapAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn attributes(&self) -> <OtapMetricsResourceView<'a> as ResourceView>::AttributesIter<'_> {
        let matching_rows = self
            .view
            .resource_attrs_map
            .get(&self.resource_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapAttributeIter {
            attrs: self.view.resource_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        let first_row = self.find_first_row_for_resource().unwrap_or(0);
        self.view
            .resource_columns
            .dropped_attributes_count
            .as_ref()
            .map(|col| {
                if col.is_valid(first_row) {
                    col.value(first_row)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }
}

impl<'a> OtapMetricsResourceView<'a> {
    /// Find the first row in the metrics batch that belongs to this resource.
    /// All rows with the same resource_id share the same resource metadata.
    fn find_first_row_for_resource(&self) -> Option<usize> {
        for (rid, row_group) in &self.view.resource_groups {
            if *rid == self.resource_id {
                return row_group.iter().next();
            }
        }
        None
    }
}

// ===== Scope Metrics =====

pub struct OtapScopeMetricsIter<'a> {
    view: &'a OtapMetricsView<'a>,
    iter: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> Iterator for OtapScopeMetricsIter<'a> {
    type Item = OtapScopeMetricsView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapScopeMetricsIter<'a> as Iterator>::Item> {
        let (scope_id, row_indices) = self.iter.next()?;
        Some(OtapScopeMetricsView {
            view: self.view,
            scope_id: *scope_id,
            row_indices,
        })
    }
}

pub struct OtapScopeMetricsView<'a> {
    view: &'a OtapMetricsView<'a>,
    scope_id: u16,
    row_indices: &'a RowGroup,
}

impl<'a> ScopeMetricsView for OtapScopeMetricsView<'a> {
    type Scope<'scp>
        = OtapMetricsScopeView<'scp>
    where
        Self: 'scp;
    type Metric<'met>
        = OtapMetricView<'met>
    where
        Self: 'met;
    type MetricIter<'met>
        = OtapMetricIter<'met>
    where
        Self: 'met;

    #[inline]
    fn scope(&self) -> Option<<OtapScopeMetricsView<'a> as ScopeMetricsView>::Scope<'_>> {
        Some(OtapMetricsScopeView {
            view: self.view,
            scope_id: self.scope_id,
        })
    }

    #[inline]
    fn metrics(&self) -> <OtapScopeMetricsView<'a> as ScopeMetricsView>::MetricIter<'_> {
        OtapMetricIter {
            view: self.view,
            row_indices: self.row_indices.iter(),
        }
    }

    #[inline]
    fn schema_url(&self) -> Str<'_> {
        self.row_indices
            .iter()
            .next()
            .and_then(|first_row| {
                self.view
                    .metrics_arrays
                    .schema_url
                    .as_ref()
                    .and_then(|col| col.str_at(first_row).map(|s| s.as_bytes()))
            })
            .unwrap_or(b"")
    }
}

// ===== Scope View (metrics-specific) =====

pub struct OtapMetricsScopeView<'a> {
    view: &'a OtapMetricsView<'a>,
    scope_id: u16,
}

impl<'a> InstrumentationScopeView for OtapMetricsScopeView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = OtapAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn name(&self) -> Option<Str<'_>> {
        let first_row = self.find_first_row_for_scope()?;
        self.view
            .scope_columns
            .name
            .as_ref()
            .and_then(|col| col.str_at(first_row).map(|s| s.as_bytes()))
    }

    #[inline]
    fn version(&self) -> Option<Str<'_>> {
        let first_row = self.find_first_row_for_scope()?;
        self.view
            .scope_columns
            .version
            .as_ref()
            .and_then(|col| col.str_at(first_row).map(|s| s.as_bytes()))
    }

    #[inline]
    fn attributes(
        &self,
    ) -> <OtapMetricsScopeView<'a> as InstrumentationScopeView>::AttributeIter<'_> {
        let matching_rows = self
            .view
            .scope_attrs_map
            .get(&self.scope_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapAttributeIter {
            attrs: self.view.scope_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        let first_row = self.find_first_row_for_scope().unwrap_or(0);
        self.view
            .scope_columns
            .dropped_attributes_count
            .as_ref()
            .map(|col| {
                if col.is_valid(first_row) {
                    col.value(first_row)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }
}

impl<'a> OtapMetricsScopeView<'a> {
    /// Find the first row in the metrics batch that belongs to this scope.
    /// All rows with the same scope_id share the same scope name/version.
    fn find_first_row_for_scope(&self) -> Option<usize> {
        for scope_list in self.view.scope_groups_map.values() {
            for (sid, row_group) in scope_list {
                if *sid == self.scope_id {
                    return row_group.iter().next();
                }
            }
        }
        None
    }
}

// ===== Individual Metric =====

pub struct OtapMetricIter<'a> {
    view: &'a OtapMetricsView<'a>,
    row_indices: RowGroupIter<'a>,
}

impl<'a> Iterator for OtapMetricIter<'a> {
    type Item = OtapMetricView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapMetricIter<'a> as Iterator>::Item> {
        let row_idx = self.row_indices.next()?;
        Some(OtapMetricView {
            view: self.view,
            row_idx,
        })
    }
}

pub struct OtapMetricView<'a> {
    view: &'a OtapMetricsView<'a>,
    row_idx: usize,
}

impl<'a> MetricView for OtapMetricView<'a> {
    type Data<'dat>
        = OtapDataView<'dat>
    where
        Self: 'dat;
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = OtapAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn name(&self) -> Str<'_> {
        self.view
            .metrics_arrays
            .name
            .str_at(self.row_idx)
            .map(|s| s.as_bytes())
            .unwrap_or(b"")
    }

    #[inline]
    fn description(&self) -> Str<'_> {
        self.view
            .metrics_arrays
            .description
            .as_ref()
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
            .unwrap_or(b"")
    }

    #[inline]
    fn unit(&self) -> Str<'_> {
        self.view
            .metrics_arrays
            .unit
            .as_ref()
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
            .unwrap_or(b"")
    }

    #[inline]
    fn data(&self) -> Option<<OtapMetricView<'a> as MetricView>::Data<'_>> {
        let metric_type_val = self
            .view
            .metrics_arrays
            .metric_type
            .value_at(self.row_idx)?;
        let metric_type = MetricType::try_from(metric_type_val).ok()?;
        let metric_id = self.view.metrics_arrays.id.value_at(self.row_idx)?;

        let aggregation_temporality = self
            .view
            .metrics_arrays
            .aggregation_temporality
            .as_ref()
            .and_then(|col| col.value_at(self.row_idx))
            .map(|v| AggregationTemporality::from(v as u32))
            .unwrap_or(AggregationTemporality::Unspecified);

        let is_monotonic = self
            .view
            .metrics_arrays
            .is_monotonic
            .value_at(self.row_idx)
            .unwrap_or(false);

        match metric_type {
            MetricType::Empty => None,
            MetricType::Gauge => Some(OtapDataView::Gauge(OtapGaugeView {
                view: self.view,
                metric_id,
            })),
            MetricType::Sum => Some(OtapDataView::Sum(OtapSumView {
                view: self.view,
                metric_id,
                aggregation_temporality,
                is_monotonic,
            })),
            MetricType::Histogram => Some(OtapDataView::Histogram(OtapHistogramView {
                view: self.view,
                metric_id,
                aggregation_temporality,
            })),
            MetricType::ExponentialHistogram => {
                Some(OtapDataView::ExponentialHistogram(OtapExpHistogramView {
                    view: self.view,
                    metric_id,
                    aggregation_temporality,
                }))
            }
            MetricType::Summary => Some(OtapDataView::Summary(OtapSummaryView {
                view: self.view,
                metric_id,
            })),
        }
    }

    #[inline]
    fn metadata(&self) -> <OtapMetricView<'a> as MetricView>::AttributeIter<'_> {
        let metric_id = self.view.metrics_arrays.id.value_at(self.row_idx);
        let matching_rows = metric_id
            .and_then(|id| self.view.metric_attrs_map.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapAttributeIter {
            attrs: self.view.metric_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }
}

// ===== DataView =====

pub enum OtapDataView<'a> {
    Gauge(OtapGaugeView<'a>),
    Sum(OtapSumView<'a>),
    Histogram(OtapHistogramView<'a>),
    ExponentialHistogram(OtapExpHistogramView<'a>),
    Summary(OtapSummaryView<'a>),
}

impl<'a> DataView<'a> for OtapDataView<'a> {
    type Gauge<'g>
        = OtapGaugeView<'g>
    where
        Self: 'g;
    type Sum<'s>
        = OtapSumView<'s>
    where
        Self: 's;
    type Histogram<'h>
        = OtapHistogramView<'h>
    where
        Self: 'h;
    type ExponentialHistogram<'e>
        = OtapExpHistogramView<'e>
    where
        Self: 'e;
    type Summary<'su>
        = OtapSummaryView<'su>
    where
        Self: 'su;

    #[inline]
    fn value_type(&self) -> DataType {
        match self {
            Self::Gauge(_) => DataType::Gauge,
            Self::Sum(_) => DataType::Sum,
            Self::Histogram(_) => DataType::Histogram,
            Self::ExponentialHistogram(_) => DataType::ExponentialHistogram,
            Self::Summary(_) => DataType::Summary,
        }
    }

    #[inline]
    fn as_gauge(&self) -> Option<<Self as DataView<'a>>::Gauge<'_>> {
        match self {
            Self::Gauge(g) => Some(OtapGaugeView {
                view: g.view,
                metric_id: g.metric_id,
            }),
            _ => None,
        }
    }

    #[inline]
    fn as_sum(&self) -> Option<<Self as DataView<'a>>::Sum<'_>> {
        match self {
            Self::Sum(s) => Some(OtapSumView {
                view: s.view,
                metric_id: s.metric_id,
                aggregation_temporality: s.aggregation_temporality,
                is_monotonic: s.is_monotonic,
            }),
            _ => None,
        }
    }

    #[inline]
    fn as_histogram(&self) -> Option<<Self as DataView<'a>>::Histogram<'_>> {
        match self {
            Self::Histogram(h) => Some(OtapHistogramView {
                view: h.view,
                metric_id: h.metric_id,
                aggregation_temporality: h.aggregation_temporality,
            }),
            _ => None,
        }
    }

    #[inline]
    fn as_exponential_histogram(&self) -> Option<<Self as DataView<'a>>::ExponentialHistogram<'_>> {
        match self {
            Self::ExponentialHistogram(e) => Some(OtapExpHistogramView {
                view: e.view,
                metric_id: e.metric_id,
                aggregation_temporality: e.aggregation_temporality,
            }),
            _ => None,
        }
    }

    #[inline]
    fn as_summary(&self) -> Option<<Self as DataView<'a>>::Summary<'_>> {
        match self {
            Self::Summary(s) => Some(OtapSummaryView {
                view: s.view,
                metric_id: s.metric_id,
            }),
            _ => None,
        }
    }
}

// ===== Gauge =====

pub struct OtapGaugeView<'a> {
    view: &'a OtapMetricsView<'a>,
    metric_id: u16,
}

impl<'a> GaugeView for OtapGaugeView<'a> {
    type NumberDataPoint<'dp>
        = OtapNumberDataPointView<'dp>
    where
        Self: 'dp;
    type NumberDataPointIter<'dp>
        = OtapNumberDpIter<'dp>
    where
        Self: 'dp;

    #[inline]
    fn data_points(&self) -> <OtapGaugeView<'a> as GaugeView>::NumberDataPointIter<'_> {
        let rows = self
            .view
            .number_dp_index
            .get(&self.metric_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapNumberDpIter {
            view: self.view,
            rows,
            pos: 0,
        }
    }
}

// ===== Sum =====

pub struct OtapSumView<'a> {
    view: &'a OtapMetricsView<'a>,
    metric_id: u16,
    aggregation_temporality: AggregationTemporality,
    is_monotonic: bool,
}

impl<'a> SumView for OtapSumView<'a> {
    type NumberDataPoint<'dp>
        = OtapNumberDataPointView<'dp>
    where
        Self: 'dp;
    type NumberDataPointIter<'dp>
        = OtapNumberDpIter<'dp>
    where
        Self: 'dp;

    #[inline]
    fn data_points(&self) -> <OtapSumView<'a> as SumView>::NumberDataPointIter<'_> {
        let rows = self
            .view
            .number_dp_index
            .get(&self.metric_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapNumberDpIter {
            view: self.view,
            rows,
            pos: 0,
        }
    }

    #[inline]
    fn aggregation_temporality(&self) -> AggregationTemporality {
        self.aggregation_temporality
    }

    #[inline]
    fn is_monotonic(&self) -> bool {
        self.is_monotonic
    }
}

// ===== Histogram =====

pub struct OtapHistogramView<'a> {
    view: &'a OtapMetricsView<'a>,
    metric_id: u16,
    aggregation_temporality: AggregationTemporality,
}

impl<'a> HistogramView for OtapHistogramView<'a> {
    type HistogramDataPoint<'dp>
        = OtapHistogramDpView<'dp>
    where
        Self: 'dp;
    type HistogramDataPointIter<'dp>
        = OtapHistogramDpIter<'dp>
    where
        Self: 'dp;

    #[inline]
    fn data_points(&self) -> <OtapHistogramView<'a> as HistogramView>::HistogramDataPointIter<'_> {
        let rows = self
            .view
            .hist_dp_index
            .get(&self.metric_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapHistogramDpIter {
            view: self.view,
            rows,
            pos: 0,
        }
    }

    #[inline]
    fn aggregation_temporality(&self) -> AggregationTemporality {
        self.aggregation_temporality
    }
}

// ===== ExponentialHistogram =====

pub struct OtapExpHistogramView<'a> {
    view: &'a OtapMetricsView<'a>,
    metric_id: u16,
    aggregation_temporality: AggregationTemporality,
}

impl<'a> ExponentialHistogramView for OtapExpHistogramView<'a> {
    type ExponentialHistogramDataPoint<'dp>
        = OtapExpHistogramDpView<'dp>
    where
        Self: 'dp;
    type ExponentialHistogramDataPointIter<'dp>
        = OtapExpHistogramDpIter<'dp>
    where
        Self: 'dp;

    #[inline]
    fn data_points(
        &self,
    ) -> <OtapExpHistogramView<'a> as ExponentialHistogramView>::ExponentialHistogramDataPointIter<'_>
    {
        let rows = self
            .view
            .exp_hist_dp_index
            .get(&self.metric_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapExpHistogramDpIter {
            view: self.view,
            rows,
            pos: 0,
        }
    }

    #[inline]
    fn aggregation_temporality(&self) -> AggregationTemporality {
        self.aggregation_temporality
    }
}

// ===== Summary =====

pub struct OtapSummaryView<'a> {
    view: &'a OtapMetricsView<'a>,
    metric_id: u16,
}

impl<'a> SummaryView for OtapSummaryView<'a> {
    type SummaryDataPoint<'dp>
        = OtapSummaryDpView<'dp>
    where
        Self: 'dp;
    type SummaryDataPointIter<'dp>
        = OtapSummaryDpIter<'dp>
    where
        Self: 'dp;

    #[inline]
    fn data_points(&self) -> <OtapSummaryView<'a> as SummaryView>::SummaryDataPointIter<'_> {
        let rows = self
            .view
            .summary_dp_index
            .get(&self.metric_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapSummaryDpIter {
            view: self.view,
            rows,
            pos: 0,
        }
    }
}

// ===== Number Data Point =====

pub struct OtapNumberDpIter<'a> {
    view: &'a OtapMetricsView<'a>,
    rows: &'a [usize],
    pos: usize,
}

impl<'a> Iterator for OtapNumberDpIter<'a> {
    type Item = OtapNumberDataPointView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapNumberDpIter<'a> as Iterator>::Item> {
        if self.pos >= self.rows.len() {
            return None;
        }
        let row_idx = self.rows[self.pos];
        self.pos += 1;
        Some(OtapNumberDataPointView {
            view: self.view,
            row_idx,
        })
    }
}

pub struct OtapNumberDataPointView<'a> {
    view: &'a OtapMetricsView<'a>,
    row_idx: usize,
}

impl<'a> NumberDataPointView for OtapNumberDataPointView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = Otap32AttributeIter<'att>
    where
        Self: 'att;
    type Exemplar<'ex>
        = OtapExemplarView<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = OtapExemplarIter<'ex>
    where
        Self: 'ex;

    #[inline]
    fn start_time_unix_nano(&self) -> u64 {
        self.view
            .number_dp_arrays
            .as_ref()
            .and_then(|a| a.start_time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn time_unix_nano(&self) -> u64 {
        self.view
            .number_dp_arrays
            .as_ref()
            .and_then(|a| a.time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn value(&self) -> Option<Value> {
        let arrays = self.view.number_dp_arrays.as_ref()?;
        // Prefer double over int (same logic as proto encoder)
        if let Some(col) = arrays.double_value {
            if let Some(val) = col.value_at(self.row_idx) {
                return Some(Value::Double(val));
            }
        }
        if let Some(col) = arrays.int_value {
            if let Some(val) = col.value_at(self.row_idx) {
                return Some(Value::Integer(val));
            }
        }
        None
    }

    #[inline]
    fn attributes(
        &self,
    ) -> <OtapNumberDataPointView<'a> as NumberDataPointView>::AttributeIter<'_> {
        let dp_id = self
            .view
            .number_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.number_dp_attrs_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        Otap32AttributeIter {
            attrs: self.view.number_dp_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn exemplars(&self) -> <OtapNumberDataPointView<'a> as NumberDataPointView>::ExemplarIter<'_> {
        let dp_id = self
            .view
            .number_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.number_exemplar_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapExemplarIter {
            exemplar_arrays: self.view.number_dp_exemplar_arrays.as_ref(),
            exemplar_attrs: self.view.number_dp_exemplar_attrs.as_ref(),
            exemplar_attrs_index: &self.view.number_exemplar_attrs_index,
            matching_rows,
            pos: 0,
        }
    }

    #[inline]
    fn flags(&self) -> DataPointFlags {
        let val = self
            .view
            .number_dp_arrays
            .as_ref()
            .and_then(|a| a.flags)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0);
        DataPointFlags::new(val)
    }
}

// ===== Histogram Data Point =====

pub struct OtapHistogramDpIter<'a> {
    view: &'a OtapMetricsView<'a>,
    rows: &'a [usize],
    pos: usize,
}

impl<'a> Iterator for OtapHistogramDpIter<'a> {
    type Item = OtapHistogramDpView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapHistogramDpIter<'a> as Iterator>::Item> {
        if self.pos >= self.rows.len() {
            return None;
        }
        let row_idx = self.rows[self.pos];
        self.pos += 1;
        Some(OtapHistogramDpView {
            view: self.view,
            row_idx,
        })
    }
}

pub struct OtapHistogramDpView<'a> {
    view: &'a OtapMetricsView<'a>,
    row_idx: usize,
}

impl<'a> HistogramDataPointView for OtapHistogramDpView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = Otap32AttributeIter<'att>
    where
        Self: 'att;
    type BucketCountIter<'bc>
        = std::vec::IntoIter<u64>
    where
        Self: 'bc;
    type ExplicitBoundsIter<'eb>
        = std::vec::IntoIter<f64>
    where
        Self: 'eb;
    type Exemplar<'ex>
        = OtapExemplarView<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = OtapExemplarIter<'ex>
    where
        Self: 'ex;

    #[inline]
    fn attributes(&self) -> <OtapHistogramDpView<'a> as HistogramDataPointView>::AttributeIter<'_> {
        let dp_id = self
            .view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.hist_dp_attrs_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        Otap32AttributeIter {
            attrs: self.view.hist_dp_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn start_time_unix_nano(&self) -> u64 {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.start_time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn time_unix_nano(&self) -> u64 {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn count(&self) -> u64 {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_count)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }

    #[inline]
    fn sum(&self) -> Option<f64> {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_sum)
            .and_then(|col| col.value_at(self.row_idx))
    }

    #[inline]
    fn bucket_counts(
        &self,
    ) -> <OtapHistogramDpView<'a> as HistogramDataPointView>::BucketCountIter<'_> {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_bucket_counts.as_ref())
            .and_then(|lva| lva.value_at_opt(self.row_idx))
            .unwrap_or_default()
            .into_iter()
    }

    #[inline]
    fn explicit_bounds(
        &self,
    ) -> <OtapHistogramDpView<'a> as HistogramDataPointView>::ExplicitBoundsIter<'_> {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_explicit_bounds.as_ref())
            .and_then(|lva| lva.value_at_opt(self.row_idx))
            .unwrap_or_default()
            .into_iter()
    }

    #[inline]
    fn exemplars(&self) -> <OtapHistogramDpView<'a> as HistogramDataPointView>::ExemplarIter<'_> {
        let dp_id = self
            .view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.hist_exemplar_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapExemplarIter {
            exemplar_arrays: self.view.hist_dp_exemplar_arrays.as_ref(),
            exemplar_attrs: self.view.hist_dp_exemplar_attrs.as_ref(),
            exemplar_attrs_index: &self.view.hist_exemplar_attrs_index,
            matching_rows,
            pos: 0,
        }
    }

    #[inline]
    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(
            self.view
                .hist_dp_arrays
                .as_ref()
                .and_then(|a| a.flags)
                .and_then(|col| col.value_at(self.row_idx))
                .unwrap_or(0),
        )
    }

    #[inline]
    fn min(&self) -> Option<f64> {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_min)
            .and_then(|col| col.value_at(self.row_idx))
    }

    #[inline]
    fn max(&self) -> Option<f64> {
        self.view
            .hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_max)
            .and_then(|col| col.value_at(self.row_idx))
    }
}

// ===== Exponential Histogram Data Point =====

pub struct OtapExpHistogramDpIter<'a> {
    view: &'a OtapMetricsView<'a>,
    rows: &'a [usize],
    pos: usize,
}

impl<'a> Iterator for OtapExpHistogramDpIter<'a> {
    type Item = OtapExpHistogramDpView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapExpHistogramDpIter<'a> as Iterator>::Item> {
        if self.pos >= self.rows.len() {
            return None;
        }
        let row_idx = self.rows[self.pos];
        self.pos += 1;
        Some(OtapExpHistogramDpView {
            view: self.view,
            row_idx,
        })
    }
}

pub struct OtapExpHistogramDpView<'a> {
    view: &'a OtapMetricsView<'a>,
    row_idx: usize,
}

impl<'a> ExponentialHistogramDataPointView for OtapExpHistogramDpView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = Otap32AttributeIter<'att>
    where
        Self: 'att;
    type Buckets<'b>
        = OtapBucketsView<'b>
    where
        Self: 'b;
    type Exemplar<'ex>
        = OtapExemplarView<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = OtapExemplarIter<'ex>
    where
        Self: 'ex;

    #[inline]
    fn attributes(
        &self,
    ) -> <OtapExpHistogramDpView<'a> as ExponentialHistogramDataPointView>::AttributeIter<'_> {
        let dp_id = self
            .view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.exp_hist_dp_attrs_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        Otap32AttributeIter {
            attrs: self.view.exp_hist_dp_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn start_time_unix_nano(&self) -> u64 {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.start_time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn time_unix_nano(&self) -> u64 {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn count(&self) -> u64 {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_count)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }

    #[inline]
    fn sum(&self) -> Option<f64> {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_sum)
            .and_then(|col| col.value_at(self.row_idx))
    }

    #[inline]
    fn scale(&self) -> i32 {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.exp_histogram_scale)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }

    #[inline]
    fn zero_count(&self) -> u64 {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.exp_histogram_zero_count)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }

    #[inline]
    fn positive(
        &self,
    ) -> Option<<OtapExpHistogramDpView<'a> as ExponentialHistogramDataPointView>::Buckets<'_>>
    {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.exp_histogram_positive.as_ref())
            .map(|b| OtapBucketsView {
                buckets: b,
                row_idx: self.row_idx,
            })
    }

    #[inline]
    fn negative(
        &self,
    ) -> Option<<OtapExpHistogramDpView<'a> as ExponentialHistogramDataPointView>::Buckets<'_>>
    {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.exp_histogram_negative.as_ref())
            .map(|b| OtapBucketsView {
                buckets: b,
                row_idx: self.row_idx,
            })
    }

    #[inline]
    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(
            self.view
                .exp_hist_dp_arrays
                .as_ref()
                .and_then(|a| a.flags)
                .and_then(|col| col.value_at(self.row_idx))
                .unwrap_or(0),
        )
    }

    #[inline]
    fn exemplars(
        &self,
    ) -> <OtapExpHistogramDpView<'a> as ExponentialHistogramDataPointView>::ExemplarIter<'_> {
        let dp_id = self
            .view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.exp_hist_exemplar_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        OtapExemplarIter {
            exemplar_arrays: self.view.exp_hist_dp_exemplar_arrays.as_ref(),
            exemplar_attrs: self.view.exp_hist_dp_exemplar_attrs.as_ref(),
            exemplar_attrs_index: &self.view.exp_hist_exemplar_attrs_index,
            matching_rows,
            pos: 0,
        }
    }

    #[inline]
    fn min(&self) -> Option<f64> {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_min)
            .and_then(|col| col.value_at(self.row_idx))
    }

    #[inline]
    fn max(&self) -> Option<f64> {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.histogram_max)
            .and_then(|col| col.value_at(self.row_idx))
    }

    #[inline]
    fn zero_threshold(&self) -> f64 {
        self.view
            .exp_hist_dp_arrays
            .as_ref()
            .and_then(|a| a.zero_threshold)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0.0)
    }
}

// ===== Summary Data Point =====

pub struct OtapSummaryDpIter<'a> {
    view: &'a OtapMetricsView<'a>,
    rows: &'a [usize],
    pos: usize,
}

impl<'a> Iterator for OtapSummaryDpIter<'a> {
    type Item = OtapSummaryDpView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapSummaryDpIter<'a> as Iterator>::Item> {
        if self.pos >= self.rows.len() {
            return None;
        }
        let row_idx = self.rows[self.pos];
        self.pos += 1;
        Some(OtapSummaryDpView {
            view: self.view,
            row_idx,
        })
    }
}

pub struct OtapSummaryDpView<'a> {
    view: &'a OtapMetricsView<'a>,
    row_idx: usize,
}

impl<'a> SummaryDataPointView for OtapSummaryDpView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = Otap32AttributeIter<'att>
    where
        Self: 'att;
    type ValueAtQuantile<'vaq>
        = OtapValueAtQuantileView<'vaq>
    where
        Self: 'vaq;
    type ValueAtQuantileIter<'vaq>
        = OtapQuantileIter<'vaq>
    where
        Self: 'vaq;

    #[inline]
    fn attributes(&self) -> <OtapSummaryDpView<'a> as SummaryDataPointView>::AttributeIter<'_> {
        let dp_id = self
            .view
            .summary_dp_arrays
            .as_ref()
            .and_then(|a| a.id.value_at(self.row_idx));
        let matching_rows = dp_id
            .and_then(|id| self.view.summary_dp_attrs_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        Otap32AttributeIter {
            attrs: self.view.summary_dp_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn start_time_unix_nano(&self) -> u64 {
        self.view
            .summary_dp_arrays
            .as_ref()
            .and_then(|a| a.start_time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn time_unix_nano(&self) -> u64 {
        self.view
            .summary_dp_arrays
            .as_ref()
            .and_then(|a| a.time_unix_nano)
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn count(&self) -> u64 {
        self.view
            .summary_dp_arrays
            .as_ref()
            .and_then(|a| a.summary_count)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }

    #[inline]
    fn sum(&self) -> f64 {
        self.view
            .summary_dp_arrays
            .as_ref()
            .and_then(|a| a.summary_sum)
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0.0)
    }

    #[inline]
    fn quantile_values(
        &self,
    ) -> <OtapSummaryDpView<'a> as SummaryDataPointView>::ValueAtQuantileIter<'_> {
        let quantile_arrays = self
            .view
            .summary_dp_arrays
            .as_ref()
            .and_then(|a| a.summary_quantile_values.as_ref());
        OtapQuantileIter::new(quantile_arrays, self.row_idx)
    }

    #[inline]
    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(
            self.view
                .summary_dp_arrays
                .as_ref()
                .and_then(|a| a.flags)
                .and_then(|col| col.value_at(self.row_idx))
                .unwrap_or(0),
        )
    }
}

// ===== ValueAtQuantile =====

pub struct OtapQuantileIter<'a> {
    quantile_arrays: Option<&'a QuantileArrays<'a>>,
    end: usize,
    current: usize,
}

impl<'a> OtapQuantileIter<'a> {
    fn new(quantile_arrays: Option<&'a QuantileArrays<'a>>, row_idx: usize) -> Self {
        let (start, end) = quantile_arrays
            .and_then(|qa| {
                if qa.list_array.is_valid(row_idx) {
                    let offsets = qa.list_array.value_offsets();
                    Some((offsets[row_idx] as usize, offsets[row_idx + 1] as usize))
                } else {
                    None
                }
            })
            .unwrap_or((0, 0));
        Self {
            quantile_arrays,
            end,
            current: start,
        }
    }
}

impl<'a> Iterator for OtapQuantileIter<'a> {
    type Item = OtapValueAtQuantileView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapQuantileIter<'a> as Iterator>::Item> {
        if self.current >= self.end {
            return None;
        }
        let idx = self.current;
        self.current += 1;
        Some(OtapValueAtQuantileView {
            quantile_arrays: self.quantile_arrays?,
            idx,
        })
    }
}

pub struct OtapValueAtQuantileView<'a> {
    quantile_arrays: &'a QuantileArrays<'a>,
    idx: usize,
}

impl<'a> ValueAtQuantileView for OtapValueAtQuantileView<'a> {
    #[inline]
    fn quantile(&self) -> f64 {
        self.quantile_arrays
            .quantile_array
            .value_at(self.idx)
            .unwrap_or(0.0)
    }
    #[inline]
    fn value(&self) -> f64 {
        self.quantile_arrays
            .value_array
            .value_at(self.idx)
            .unwrap_or(0.0)
    }
}

// ===== Buckets =====

pub struct OtapBucketsView<'a> {
    buckets: &'a PositiveNegativeArrayAccess<'a>,
    row_idx: usize,
}

impl<'a> BucketsView for OtapBucketsView<'a> {
    type BucketCountIter<'bc>
        = std::vec::IntoIter<u64>
    where
        Self: 'bc;

    #[inline]
    fn offset(&self) -> i32 {
        self.buckets
            .offset_array
            .value_at(self.row_idx)
            .unwrap_or(0)
    }

    #[inline]
    fn bucket_counts(&self) -> <OtapBucketsView<'a> as BucketsView>::BucketCountIter<'_> {
        self.buckets
            .bucket_count
            .value_at_opt(self.row_idx)
            .unwrap_or_default()
            .into_iter()
    }
}

// ===== Exemplar =====

pub struct OtapExemplarIter<'a> {
    exemplar_arrays: Option<&'a ExemplarArrays<'a>>,
    exemplar_attrs: Option<&'a Attribute32Arrays<'a>>,
    exemplar_attrs_index: &'a BTreeMap<u32, Vec<usize>>,
    matching_rows: &'a [usize],
    pos: usize,
}

impl<'a> Iterator for OtapExemplarIter<'a> {
    type Item = OtapExemplarView<'a>;
    #[inline]
    fn next(&mut self) -> Option<<OtapExemplarIter<'a> as Iterator>::Item> {
        if self.pos >= self.matching_rows.len() {
            return None;
        }
        let row_idx = self.matching_rows[self.pos];
        self.pos += 1;
        Some(OtapExemplarView {
            exemplar_arrays: self.exemplar_arrays?,
            exemplar_attrs: self.exemplar_attrs,
            exemplar_attrs_index: self.exemplar_attrs_index,
            row_idx,
        })
    }
}

pub struct OtapExemplarView<'a> {
    exemplar_arrays: &'a ExemplarArrays<'a>,
    exemplar_attrs: Option<&'a Attribute32Arrays<'a>>,
    exemplar_attrs_index: &'a BTreeMap<u32, Vec<usize>>,
    row_idx: usize,
}

impl<'a> ExemplarView for OtapExemplarView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = Otap32AttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn filtered_attributes(&self) -> <OtapExemplarView<'a> as ExemplarView>::AttributeIter<'_> {
        let exemplar_id = self.exemplar_arrays.id.value_at(self.row_idx);
        let matching_rows = exemplar_id
            .and_then(|id| self.exemplar_attrs_index.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        Otap32AttributeIter {
            attrs: self.exemplar_attrs,
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn time_unix_nano(&self) -> u64 {
        self.exemplar_arrays
            .time_unix_nano
            .and_then(|col| col.value_at(self.row_idx).map(|v| v as u64))
            .unwrap_or(0)
    }

    #[inline]
    fn value(&self) -> Option<Value> {
        if let Some(col) = self.exemplar_arrays.double_value {
            if let Some(val) = col.value_at(self.row_idx) {
                return Some(Value::Double(val));
            }
        }
        if let Some(col) = self.exemplar_arrays.int_value {
            if let Some(val) = col.value_at(self.row_idx) {
                return Some(Value::Integer(val));
            }
        }
        None
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.exemplar_arrays
            .span_id
            .as_ref()
            .and_then(|col| col.slice_at(self.row_idx))
            .and_then(|bytes| bytes.try_into().ok())
    }

    #[inline]
    fn trace_id(&self) -> Option<&TraceId> {
        self.exemplar_arrays
            .trace_id
            .as_ref()
            .and_then(|col| col.slice_at(self.row_idx))
            .and_then(|bytes| bytes.try_into().ok())
    }
}
