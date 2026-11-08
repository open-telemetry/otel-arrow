// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric start time processor for OTAP metrics (`subtract_initial_point` strategy).
//!
//! Adjusts `start_time_unix_nano` and cumulative values on cumulative `Sum`, `Histogram`,
//! `ExponentialHistogram`, and `Summary` data points directly on columnar OTAP Arrow record
//! batches without round-tripping through OTLP or per-datapoint attribute maps.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = METRIC_START_TIME_PROCESSOR_URN,
    target = "otel.processor.metric_start_time",
);

use std::sync::Arc;
use std::time::Instant;

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBuilder, Float64Array, Int32Array, Int64Array, ListArray,
    ListBuilder, NullBufferBuilder, RecordBatch, StringArray, StructArray,
    TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array, UInt64Builder,
};
use arrow::compute::filter_record_batch;
use arrow::datatypes::{DataType, Field, Schema, UInt32Type};
use async_trait::async_trait;
use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use linkme::distributed_slice;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::config::ProcessorConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::{Error, ProcessorErrorKind, format_error_sources};
use otel_arrow_dfe_engine::local::processor as local;
use otel_arrow_dfe_engine::message::Message;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::process_duration::ComputeDuration;
use otel_arrow_dfe_engine::processor::{
    FlowMetricHook, ProcessorRuntimeRequirements, ProcessorWrapper,
};
use otel_arrow_dfe_engine::{
    ConsumerEffectHandlerExtension, MessageSourceLocalEffectHandlerExtension,
};
use otel_arrow_dfe_otap::{
    OTAP_PROCESSOR_FACTORIES,
    pdata::{NativePdataProcessor, NativeProcessorAdapter, OtapArrowPdata, OtapPdata},
};
use otel_arrow_dfe_pdata::arrays::{
    MaybeDictArrayAccessor, NullableArrayAccessor, get_required_array,
};
use otel_arrow_dfe_pdata::error::Error as PdataError;
use otel_arrow_dfe_pdata::otap::filter::{
    ChildBatchFilterIdHelper, IdBitmap, IdBitmapPool, filter_otap_batch,
};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use otel_arrow_dfe_telemetry::common_attributes::SignalAttributes;
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use serde_json::Value;

pub mod config;
mod metrics;
mod state;

use self::config::Config;
use self::metrics::MetricStartTimeMetrics;
use self::state::{
    AttrHasher, ExpBucketsState, ExpHistogramSeriesState, HistogramSeriesState, NumberSeriesState,
    SubtractInitialCache, SummarySeriesState,
};

/// The URN for the metric start time processor.
pub const METRIC_START_TIME_PROCESSOR_URN: &str = "urn:otel:processor:metric_start_time";

/// OTAP `metric_type` values from `UnivariateMetrics`.
const METRIC_TYPE_SUM: u8 = 2;
const METRIC_TYPE_HISTOGRAM: u8 = 3;
const METRIC_TYPE_EXP_HISTOGRAM: u8 = 4;
const METRIC_TYPE_SUMMARY: u8 = 5;

/// OTLP `AggregationTemporality::Cumulative` value in `UnivariateMetrics`.
const TEMPORALITY_CUMULATIVE: i32 = 2;

/// OTLP `DataPointFlags::FLAG_NO_RECORDED_VALUE` bitmask.
const FLAG_NO_RECORDED_VALUE: u32 = 1;

/// Start time offset subtracted from the current point's timestamp when a counter reset occurs
/// (1 millisecond in nanoseconds), matching Go's `subtractinitial` implementation.
const RESET_START_TIME_OFFSET_NS: i64 = 1_000_000;

/// Upper bound for direct `u16` `metric_id` flat-vector lookup.
const MAX_DENSE_METRIC_ID: usize = 65_536;

/// Counters returned by a single batch adjustment pass.
#[derive(Debug, Default, Clone, Copy)]
struct BatchAdjustStats {
    dropped_initial: u64,
    adjusted_points: u64,
    resets_detected: u64,
}

impl BatchAdjustStats {
    #[inline]
    fn merge(&mut self, other: Self) {
        self.dropped_initial += other.dropped_initial;
        self.adjusted_points += other.adjusted_points;
        self.resets_detected += other.resets_detected;
    }
}

/// Per-metric metadata resolved from `UnivariateMetrics` for the active batch.
#[derive(Debug, Clone, Copy)]
struct EligibleMetricSlot {
    resource_hash: u128,
    metric_row: usize,
    metric_type: u8,
}

/// Reusable lookup table mapping `metric_id: u16` to its [`EligibleMetricSlot`] in the current
/// `UnivariateMetrics` batch.
#[derive(Debug, Default)]
struct BatchMetricLookup {
    slots: Vec<Option<EligibleMetricSlot>>,
    populated_ids: Vec<u16>,
    eligible_row_flags: Vec<bool>,
    has_sum: bool,
    has_summary: bool,
    has_histogram: bool,
    has_exp_histogram: bool,
}

impl BatchMetricLookup {
    fn clear(&mut self) {
        for &id in &self.populated_ids {
            if let Some(slot) = self.slots.get_mut(id as usize) {
                *slot = None;
            }
        }
        self.populated_ids.clear();
        self.eligible_row_flags.clear();
        self.has_sum = false;
        self.has_summary = false;
        self.has_histogram = false;
        self.has_exp_histogram = false;
    }

    #[inline]
    fn any_eligible(&self) -> bool {
        self.has_sum || self.has_summary || self.has_histogram || self.has_exp_histogram
    }

    #[inline]
    fn get(&self, metric_id: u16, expected_type: u8) -> Option<EligibleMetricSlot> {
        let slot = self.slots.get(metric_id as usize).copied().flatten()?;
        (slot.metric_type == expected_type).then_some(slot)
    }

    #[inline]
    fn is_eligible_row(&self, row: usize) -> bool {
        self.eligible_row_flags.get(row).copied().unwrap_or(false)
    }
}

/// Stateful columnar adjustment engine for `subtract_initial_point`.
///
/// Separated into its own struct so [`MetricStartTimeProcessor::process_native`] can mutably
/// borrow the engine inside the `effect_handler.timed(&self.compute_duration, ...)` closure.
struct SubtractInitialEngine {
    cache: SubtractInitialCache,
    res_attr_hasher: AttrHasher,
    dp_attr_hasher: AttrHasher,
    metric_lookup: BatchMetricLookup,
    id_bitmap_pool: IdBitmapPool,
}

/// Processor that adjusts `start_time_unix_nano` on cumulative metrics using the
/// `subtract_initial_point` strategy.
pub struct MetricStartTimeProcessor {
    engine: SubtractInitialEngine,
    metrics: MeasurementMetricSet<MetricStartTimeMetrics>,
    compute_duration: ComputeDuration,
}

/// Factory function to create a [`MetricStartTimeProcessor`].
pub fn create_metric_start_time_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        MetricStartTimeProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register [`MetricStartTimeProcessor`] as an OTAP processor factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Processor)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static METRIC_START_TIME_PROCESSOR_FACTORY: otel_arrow_dfe_engine::ProcessorFactory<OtapPdata> =
    otel_arrow_dfe_engine::ProcessorFactory {
        name: METRIC_START_TIME_PROCESSOR_URN,
        create:
            |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig,
             _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
                create_metric_start_time_processor(pipeline_ctx, node, node_config, proc_cfg)
            },
        context_declarations: None,
        wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: |value| {
            let config: Config =
                otel_arrow_dfe_config::validation::deserialize_typed_config(value)?;
            config.validate()
        },
    };

impl MetricStartTimeProcessor {
    /// Creates a new [`MetricStartTimeProcessor`] from a typed [`Config`].
    pub fn new(config: Config, pipeline_ctx: PipelineContext) -> Result<Self, ConfigError> {
        config.validate()?;
        let metrics = MetricStartTimeMetrics::register(&pipeline_ctx);
        let compute_duration = ComputeDuration::new(&pipeline_ctx);
        Ok(Self {
            engine: SubtractInitialEngine {
                cache: SubtractInitialCache::new(config.gc_interval, Instant::now()),
                res_attr_hasher: AttrHasher::new(),
                dp_attr_hasher: AttrHasher::new(),
                metric_lookup: BatchMetricLookup::default(),
                id_bitmap_pool: IdBitmapPool::new(),
            },
            metrics,
            compute_duration,
        })
    }

    /// Creates a new [`MetricStartTimeProcessor`] from a JSON configuration value.
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        Self::new(config, pipeline_ctx)
    }
}

impl SubtractInitialEngine {
    /// Adjusts an OTAP metrics batch in place at the specified instant.
    fn process_metrics_batch(
        &mut self,
        records: &mut OtapArrowRecords,
        now: Instant,
    ) -> Result<BatchAdjustStats, PdataError> {
        let Some(metrics_rb) = records.get(ArrowPayloadType::UnivariateMetrics) else {
            self.cache.maybe_gc(now);
            return Ok(BatchAdjustStats::default());
        };
        if metrics_rb.num_rows() == 0 {
            self.cache.maybe_gc(now);
            return Ok(BatchAdjustStats::default());
        }

        // Index eligible cumulative metrics in UnivariateMetrics.
        self.populate_metric_lookup(records)?;
        if !self.metric_lookup.any_eligible() {
            self.cache.maybe_gc(now);
            return Ok(BatchAdjustStats::default());
        }

        let mut stats = BatchAdjustStats::default();

        if self.metric_lookup.has_sum {
            let s = self.adjust_number_data_points(records)?;
            stats.merge(s);
        }
        if self.metric_lookup.has_summary {
            let s = self.adjust_summary_data_points(records)?;
            stats.merge(s);
        }
        if self.metric_lookup.has_histogram {
            let s = self.adjust_histogram_data_points(records)?;
            stats.merge(s);
        }
        if self.metric_lookup.has_exp_histogram {
            let s = self.adjust_exp_histogram_data_points(records)?;
            stats.merge(s);
        }

        // If any initial data points were dropped, prune UnivariateMetrics rows whose data points
        // were all dropped so empty metric shells are not emitted downstream.
        if stats.dropped_initial > 0 {
            self.prune_empty_metrics(records)?;
        }

        self.cache.maybe_gc(now);
        Ok(stats)
    }

    fn populate_metric_lookup(&mut self, records: &OtapArrowRecords) -> Result<(), PdataError> {
        self.metric_lookup.clear();
        let Some(metrics_rb) = records.get(ArrowPayloadType::UnivariateMetrics) else {
            return Ok(());
        };
        let num_rows = metrics_rb.num_rows();

        let id_arr = get_required_array(metrics_rb, consts::ID)?;
        let ids = id_arr
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| PdataError::ColumnDataTypeMismatch {
                name: consts::ID.into(),
                actual: id_arr.data_type().clone(),
                expect: DataType::UInt16,
            })?;

        let type_arr = get_required_array(metrics_rb, consts::METRIC_TYPE)?;
        let metric_types = type_arr
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| PdataError::ColumnDataTypeMismatch {
                name: consts::METRIC_TYPE.into(),
                actual: type_arr.data_type().clone(),
                expect: DataType::UInt8,
            })?;

        let temporality = metrics_rb
            .column_by_name(consts::AGGREGATION_TEMPORALITY)
            .map(MaybeDictArrayAccessor::<Int32Array>::try_new)
            .transpose()?;

        let res_struct = metrics_rb
            .column_by_name(consts::RESOURCE)
            .and_then(|c| c.as_any().downcast_ref::<StructArray>());
        let res_ids = res_struct
            .and_then(|s| s.column_by_name(consts::ID))
            .and_then(|c| c.as_any().downcast_ref::<UInt16Array>());

        let mut resource_hashes_populated = false;
        self.metric_lookup.eligible_row_flags.reserve(num_rows);

        for row in 0..num_rows {
            if !ids.is_valid(row) || !metric_types.is_valid(row) {
                self.metric_lookup.eligible_row_flags.push(false);
                continue;
            }
            let m_type = metric_types.value(row);
            let is_eligible = match m_type {
                METRIC_TYPE_SUMMARY => true,
                METRIC_TYPE_SUM | METRIC_TYPE_HISTOGRAM | METRIC_TYPE_EXP_HISTOGRAM => temporality
                    .as_ref()
                    .and_then(|t| t.value_at(row))
                    .is_some_and(|t| t == TEMPORALITY_CUMULATIVE),
                _ => false,
            };

            self.metric_lookup.eligible_row_flags.push(is_eligible);
            if !is_eligible {
                continue;
            }

            if !resource_hashes_populated {
                self.res_attr_hasher
                    .populate_u16(records.get(ArrowPayloadType::ResourceAttrs))?;
                resource_hashes_populated = true;
            }

            let res_valid = res_struct.is_none_or(|s| s.is_valid(row));
            let resource_hash = if res_valid
                && let Some(r_ids) = res_ids
                && r_ids.is_valid(row)
            {
                self.res_attr_hasher.get(u32::from(r_ids.value(row)))
            } else {
                0
            };

            let metric_id = ids.value(row);
            let idx = metric_id as usize;
            if idx < MAX_DENSE_METRIC_ID {
                if idx >= self.metric_lookup.slots.len() {
                    self.metric_lookup.slots.resize(idx + 1, None);
                }
                self.metric_lookup.slots[idx] = Some(EligibleMetricSlot {
                    resource_hash,
                    metric_row: row,
                    metric_type: m_type,
                });
                self.metric_lookup.populated_ids.push(metric_id);
            }

            match m_type {
                METRIC_TYPE_SUM => self.metric_lookup.has_sum = true,
                METRIC_TYPE_SUMMARY => self.metric_lookup.has_summary = true,
                METRIC_TYPE_HISTOGRAM => self.metric_lookup.has_histogram = true,
                METRIC_TYPE_EXP_HISTOGRAM => self.metric_lookup.has_exp_histogram = true,
                _ => {}
            }
        }

        Ok(())
    }

    fn adjust_number_data_points(
        &mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<BatchAdjustStats, PdataError> {
        let Some(dp_rb) = records.get(ArrowPayloadType::NumberDataPoints) else {
            return Ok(BatchAdjustStats::default());
        };
        let num_rows = dp_rb.num_rows();
        if num_rows == 0 {
            return Ok(BatchAdjustStats::default());
        }

        let metrics_rb = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .expect("UnivariateMetrics checked by caller");
        let metric_names = MaybeDictArrayAccessor::<StringArray>::try_new(get_required_array(
            metrics_rb,
            consts::NAME,
        )?)?;

        self.dp_attr_hasher
            .populate_u32(records.get(ArrowPayloadType::NumberDpAttrs))?;

        let common = CommonDpColumns::try_new(dp_rb)?;
        let ints = dp_rb
            .column_by_name(consts::INT_VALUE)
            .and_then(|c| c.as_any().downcast_ref::<Int64Array>());
        let doubles = dp_rb
            .column_by_name(consts::DOUBLE_VALUE)
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>());

        let mut new_start_times: Vec<Option<i64>> = Vec::with_capacity(num_rows);
        let mut new_ints: Vec<Option<i64>> =
            Vec::with_capacity(if ints.is_some() { num_rows } else { 0 });
        let mut new_doubles: Vec<Option<f64>> =
            Vec::with_capacity(if doubles.is_some() { num_rows } else { 0 });
        let mut keep_flags: Vec<bool> = Vec::with_capacity(num_rows);
        let mut kept_count = 0usize;
        let mut stats = BatchAdjustStats::default();

        let mut active_parent_id: Option<u16> = None;
        let mut active_map: Option<&mut HashMap<u128, NumberSeriesState>> = None;

        for row in 0..num_rows {
            let parent_id = common.parent_id_at(row);
            if active_parent_id != Some(parent_id) {
                active_parent_id = Some(parent_id);
                active_map = self
                    .metric_lookup
                    .get(parent_id, METRIC_TYPE_SUM)
                    .map(|slot| {
                        let name = metric_names.str_at(slot.metric_row).unwrap_or("");
                        self.cache.number_series_mut(slot.resource_hash, name)
                    });
            }

            let orig_start_opt = common.start_time_at(row);
            let time_ns = common.time_at(row);
            let cur_int_opt = ints.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_double_opt = doubles.and_then(|c| c.is_valid(row).then(|| c.value(row)));

            let Some(series_map) = active_map.as_deref_mut() else {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if ints.is_some() {
                    new_ints.push(cur_int_opt);
                }
                if doubles.is_some() {
                    new_doubles.push(cur_double_opt);
                }
                continue;
            };

            let orig_start_ns = orig_start_opt.unwrap_or(0);
            if orig_start_ns != 0 && orig_start_ns != time_ns {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if ints.is_some() {
                    new_ints.push(cur_int_opt);
                }
                if doubles.is_some() {
                    new_doubles.push(cur_double_opt);
                }
                continue;
            }

            let dp_attr_hash = common
                .id_at(row)
                .map_or(0, |id| self.dp_attr_hasher.get(id));
            let cur_int = cur_int_opt.unwrap_or(0);
            let cur_double = cur_double_opt.unwrap_or(0.0);

            match series_map.entry(dp_attr_hash) {
                Entry::Vacant(vacant) => {
                    let _ = vacant.insert(NumberSeriesState {
                        mark: true,
                        start_time_ns: time_ns,
                        ref_double: cur_double,
                        ref_int: cur_int,
                        prev_double: cur_double,
                        prev_int: cur_int,
                    });
                    keep_flags.push(false);
                    stats.dropped_initial += 1;
                }
                Entry::Occupied(mut occupied) => {
                    let state = occupied.get_mut();
                    state.mark = true;
                    keep_flags.push(true);
                    kept_count += 1;
                    stats.adjusted_points += 1;

                    if common.is_no_recorded_value(row) {
                        new_start_times.push(Some(state.start_time_ns));
                        if ints.is_some() {
                            new_ints.push(cur_int_opt);
                        }
                        if doubles.is_some() {
                            new_doubles.push(cur_double_opt);
                        }
                        continue;
                    }

                    if cur_double < state.prev_double || cur_int < state.prev_int {
                        let reset_start = time_ns.saturating_sub(RESET_START_TIME_OFFSET_NS);
                        state.start_time_ns = reset_start;
                        state.ref_double = 0.0;
                        state.ref_int = 0;
                        state.prev_double = cur_double;
                        state.prev_int = cur_int;
                        stats.resets_detected += 1;

                        new_start_times.push(Some(reset_start));
                        if ints.is_some() {
                            new_ints.push(cur_int_opt);
                        }
                        if doubles.is_some() {
                            new_doubles.push(cur_double_opt);
                        }
                    } else {
                        state.prev_double = cur_double;
                        state.prev_int = cur_int;
                        new_start_times.push(Some(state.start_time_ns));
                        if ints.is_some() {
                            new_ints.push(cur_int_opt.map(|v| v.wrapping_sub(state.ref_int)));
                        }
                        if doubles.is_some() {
                            new_doubles.push(cur_double_opt.map(|v| v - state.ref_double));
                        }
                    }
                }
            }
        }

        if kept_count == 0 {
            let _ = records.remove(ArrowPayloadType::NumberDataPoints);
            let _ = records.remove(ArrowPayloadType::NumberDpAttrs);
            let _ = records.remove(ArrowPayloadType::NumberDpExemplars);
            let _ = records.remove(ArrowPayloadType::NumberDpExemplarAttrs);
            return Ok(stats);
        }

        let keep_filter = (kept_count < num_rows).then(|| BooleanArray::from(keep_flags));
        let start_col: ArrayRef =
            Arc::new(TimestampNanosecondArray::from(new_start_times).with_timezone_utc());
        let mut replacements: Vec<(&str, ArrayRef)> = Vec::with_capacity(2);
        if ints.is_some() {
            replacements.push((consts::INT_VALUE, Arc::new(Int64Array::from(new_ints))));
        }
        if doubles.is_some() {
            replacements.push((
                consts::DOUBLE_VALUE,
                Arc::new(Float64Array::from(new_doubles)),
            ));
        }

        let new_rb =
            rebuild_data_point_batch(dp_rb, keep_filter.as_ref(), start_col, &replacements)?;
        records.set(ArrowPayloadType::NumberDataPoints, new_rb)?;

        if kept_count < num_rows {
            self.filter_dp_children(
                records,
                ArrowPayloadType::NumberDataPoints,
                ArrowPayloadType::NumberDpAttrs,
                Some((
                    ArrowPayloadType::NumberDpExemplars,
                    ArrowPayloadType::NumberDpExemplarAttrs,
                )),
            )?;
        }

        Ok(stats)
    }

    fn adjust_summary_data_points(
        &mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<BatchAdjustStats, PdataError> {
        let Some(dp_rb) = records.get(ArrowPayloadType::SummaryDataPoints) else {
            return Ok(BatchAdjustStats::default());
        };
        let num_rows = dp_rb.num_rows();
        if num_rows == 0 {
            return Ok(BatchAdjustStats::default());
        }

        let metrics_rb = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .expect("UnivariateMetrics checked by caller");
        let metric_names = MaybeDictArrayAccessor::<StringArray>::try_new(get_required_array(
            metrics_rb,
            consts::NAME,
        )?)?;

        self.dp_attr_hasher
            .populate_u32(records.get(ArrowPayloadType::SummaryDpAttrs))?;

        let common = CommonDpColumns::try_new(dp_rb)?;
        let counts = dp_rb
            .column_by_name(consts::COUNT)
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>());
        let sums = dp_rb
            .column_by_name(consts::SUM)
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>());

        let mut new_start_times: Vec<Option<i64>> = Vec::with_capacity(num_rows);
        let mut new_counts: Vec<Option<u64>> =
            Vec::with_capacity(if counts.is_some() { num_rows } else { 0 });
        let mut new_sums: Vec<Option<f64>> =
            Vec::with_capacity(if sums.is_some() { num_rows } else { 0 });
        let mut keep_flags: Vec<bool> = Vec::with_capacity(num_rows);
        let mut kept_count = 0usize;
        let mut stats = BatchAdjustStats::default();

        let mut active_parent_id: Option<u16> = None;
        let mut active_map: Option<&mut HashMap<u128, SummarySeriesState>> = None;

        for row in 0..num_rows {
            let parent_id = common.parent_id_at(row);
            if active_parent_id != Some(parent_id) {
                active_parent_id = Some(parent_id);
                active_map = self
                    .metric_lookup
                    .get(parent_id, METRIC_TYPE_SUMMARY)
                    .map(|slot| {
                        let name = metric_names.str_at(slot.metric_row).unwrap_or("");
                        self.cache.summary_series_mut(slot.resource_hash, name)
                    });
            }

            let orig_start_opt = common.start_time_at(row);
            let time_ns = common.time_at(row);
            let cur_count_opt = counts.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_sum_opt = sums.and_then(|c| c.is_valid(row).then(|| c.value(row)));

            let Some(series_map) = active_map.as_deref_mut() else {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if counts.is_some() {
                    new_counts.push(cur_count_opt);
                }
                if sums.is_some() {
                    new_sums.push(cur_sum_opt);
                }
                continue;
            };

            let orig_start_ns = orig_start_opt.unwrap_or(0);
            if orig_start_ns != 0 && orig_start_ns != time_ns {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if counts.is_some() {
                    new_counts.push(cur_count_opt);
                }
                if sums.is_some() {
                    new_sums.push(cur_sum_opt);
                }
                continue;
            }

            let dp_attr_hash = common
                .id_at(row)
                .map_or(0, |id| self.dp_attr_hasher.get(id));
            let cur_count = cur_count_opt.unwrap_or(0);
            let cur_sum = cur_sum_opt.unwrap_or(0.0);

            match series_map.entry(dp_attr_hash) {
                Entry::Vacant(vacant) => {
                    let _ = vacant.insert(SummarySeriesState {
                        mark: true,
                        start_time_ns: time_ns,
                        ref_count: cur_count,
                        ref_sum: cur_sum,
                        prev_count: cur_count,
                        prev_sum: cur_sum,
                    });
                    keep_flags.push(false);
                    stats.dropped_initial += 1;
                }
                Entry::Occupied(mut occupied) => {
                    let state = occupied.get_mut();
                    state.mark = true;
                    keep_flags.push(true);
                    kept_count += 1;
                    stats.adjusted_points += 1;

                    if common.is_no_recorded_value(row) {
                        new_start_times.push(Some(state.start_time_ns));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt);
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt);
                        }
                        continue;
                    }

                    if cur_count < state.prev_count || cur_sum < state.prev_sum {
                        let reset_start = time_ns.saturating_sub(RESET_START_TIME_OFFSET_NS);
                        state.start_time_ns = reset_start;
                        state.ref_count = 0;
                        state.ref_sum = 0.0;
                        state.prev_count = cur_count;
                        state.prev_sum = cur_sum;
                        stats.resets_detected += 1;

                        new_start_times.push(Some(reset_start));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt);
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt);
                        }
                    } else {
                        state.prev_count = cur_count;
                        state.prev_sum = cur_sum;
                        new_start_times.push(Some(state.start_time_ns));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt.map(|v| v.wrapping_sub(state.ref_count)));
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt.map(|v| v - state.ref_sum));
                        }
                    }
                }
            }
        }

        if kept_count == 0 {
            let _ = records.remove(ArrowPayloadType::SummaryDataPoints);
            let _ = records.remove(ArrowPayloadType::SummaryDpAttrs);
            return Ok(stats);
        }

        let keep_filter = (kept_count < num_rows).then(|| BooleanArray::from(keep_flags));
        let start_col: ArrayRef =
            Arc::new(TimestampNanosecondArray::from(new_start_times).with_timezone_utc());
        let mut replacements: Vec<(&str, ArrayRef)> = Vec::with_capacity(2);
        if counts.is_some() {
            replacements.push((consts::COUNT, Arc::new(UInt64Array::from(new_counts))));
        }
        if sums.is_some() {
            replacements.push((consts::SUM, Arc::new(Float64Array::from(new_sums))));
        }

        let new_rb =
            rebuild_data_point_batch(dp_rb, keep_filter.as_ref(), start_col, &replacements)?;
        records.set(ArrowPayloadType::SummaryDataPoints, new_rb)?;

        if kept_count < num_rows {
            self.filter_dp_children(
                records,
                ArrowPayloadType::SummaryDataPoints,
                ArrowPayloadType::SummaryDpAttrs,
                None,
            )?;
        }

        Ok(stats)
    }

    fn adjust_histogram_data_points(
        &mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<BatchAdjustStats, PdataError> {
        let Some(dp_rb) = records.get(ArrowPayloadType::HistogramDataPoints) else {
            return Ok(BatchAdjustStats::default());
        };
        let num_rows = dp_rb.num_rows();
        if num_rows == 0 {
            return Ok(BatchAdjustStats::default());
        }

        let metrics_rb = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .expect("UnivariateMetrics checked by caller");
        let metric_names = MaybeDictArrayAccessor::<StringArray>::try_new(get_required_array(
            metrics_rb,
            consts::NAME,
        )?)?;

        self.dp_attr_hasher
            .populate_u32(records.get(ArrowPayloadType::HistogramDpAttrs))?;

        let common = CommonDpColumns::try_new(dp_rb)?;
        let counts = dp_rb
            .column_by_name(consts::COUNT)
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>());
        let sums = dp_rb
            .column_by_name(consts::SUM)
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>());
        let buckets_col = dp_rb
            .column_by_name(consts::HISTOGRAM_BUCKET_COUNTS)
            .map(U64ListSliceAccessor::try_new)
            .transpose()?;
        let bounds_col = dp_rb
            .column_by_name(consts::HISTOGRAM_EXPLICIT_BOUNDS)
            .map(F64ListSliceAccessor::try_new)
            .transpose()?;

        let mut new_start_times: Vec<Option<i64>> = Vec::with_capacity(num_rows);
        let mut new_counts: Vec<Option<u64>> =
            Vec::with_capacity(if counts.is_some() { num_rows } else { 0 });
        let mut new_sums: Vec<Option<f64>> =
            Vec::with_capacity(if sums.is_some() { num_rows } else { 0 });
        let mut new_buckets = ListBuilder::with_capacity(UInt64Builder::new(), num_rows);
        let has_buckets_col = buckets_col.is_some();

        let mut keep_flags: Vec<bool> = Vec::with_capacity(num_rows);
        let mut kept_count = 0usize;
        let mut stats = BatchAdjustStats::default();

        let mut active_parent_id: Option<u16> = None;
        let mut active_map: Option<&mut HashMap<u128, HistogramSeriesState>> = None;

        for row in 0..num_rows {
            let parent_id = common.parent_id_at(row);
            if active_parent_id != Some(parent_id) {
                active_parent_id = Some(parent_id);
                active_map = self
                    .metric_lookup
                    .get(parent_id, METRIC_TYPE_HISTOGRAM)
                    .map(|slot| {
                        let name = metric_names.str_at(slot.metric_row).unwrap_or("");
                        self.cache.histogram_series_mut(slot.resource_hash, name)
                    });
            }

            let orig_start_opt = common.start_time_at(row);
            let time_ns = common.time_at(row);
            let cur_count_opt = counts.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_sum_opt = sums.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_buckets_opt = buckets_col.as_ref().and_then(|c| c.slice_at(row));
            let cur_bounds = bounds_col
                .as_ref()
                .and_then(|c| c.slice_at(row))
                .unwrap_or(&[]);

            let Some(series_map) = active_map.as_deref_mut() else {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if counts.is_some() {
                    new_counts.push(cur_count_opt);
                }
                if sums.is_some() {
                    new_sums.push(cur_sum_opt);
                }
                if has_buckets_col {
                    append_opt_u64_slice(&mut new_buckets, cur_buckets_opt);
                }
                continue;
            };

            let orig_start_ns = orig_start_opt.unwrap_or(0);
            if orig_start_ns != 0 && orig_start_ns != time_ns {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if counts.is_some() {
                    new_counts.push(cur_count_opt);
                }
                if sums.is_some() {
                    new_sums.push(cur_sum_opt);
                }
                if has_buckets_col {
                    append_opt_u64_slice(&mut new_buckets, cur_buckets_opt);
                }
                continue;
            }

            let dp_attr_hash = common
                .id_at(row)
                .map_or(0, |id| self.dp_attr_hasher.get(id));
            let cur_count = cur_count_opt.unwrap_or(0);
            let cur_sum = cur_sum_opt.unwrap_or(0.0);
            let cur_buckets = cur_buckets_opt.unwrap_or(&[]);

            match series_map.entry(dp_attr_hash) {
                Entry::Vacant(vacant) => {
                    let _ = vacant.insert(HistogramSeriesState {
                        mark: true,
                        start_time_ns: time_ns,
                        ref_count: cur_count,
                        ref_sum: cur_sum,
                        prev_count: cur_count,
                        prev_sum: cur_sum,
                        explicit_bounds_bits: cur_bounds.iter().map(|b| b.to_bits()).collect(),
                        ref_buckets: cur_buckets.to_vec(),
                        prev_buckets: cur_buckets.to_vec(),
                    });
                    keep_flags.push(false);
                    stats.dropped_initial += 1;
                }
                Entry::Occupied(mut occupied) => {
                    let state = occupied.get_mut();
                    state.mark = true;
                    keep_flags.push(true);
                    kept_count += 1;
                    stats.adjusted_points += 1;

                    if common.is_no_recorded_value(row) {
                        new_start_times.push(Some(state.start_time_ns));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt);
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt);
                        }
                        if has_buckets_col {
                            append_opt_u64_slice(&mut new_buckets, cur_buckets_opt);
                        }
                        continue;
                    }

                    let is_reset = cur_count < state.prev_count
                        || cur_sum < state.prev_sum
                        || !bounds_eq_bits(cur_bounds, &state.explicit_bounds_bits)
                        || cur_buckets.len() != state.prev_buckets.len()
                        || cur_buckets
                            .iter()
                            .zip(state.prev_buckets.iter())
                            .any(|(&cur_b, &prev_b)| cur_b < prev_b);

                    if is_reset {
                        let reset_start = time_ns.saturating_sub(RESET_START_TIME_OFFSET_NS);
                        state.start_time_ns = reset_start;
                        state.ref_count = 0;
                        state.ref_sum = 0.0;
                        state.prev_count = cur_count;
                        state.prev_sum = cur_sum;
                        state.explicit_bounds_bits.clear();
                        state
                            .explicit_bounds_bits
                            .extend(cur_bounds.iter().map(|b| b.to_bits()));
                        state.ref_buckets.clear();
                        state.ref_buckets.resize(cur_buckets.len(), 0);
                        state.prev_buckets.clear();
                        state.prev_buckets.extend_from_slice(cur_buckets);
                        stats.resets_detected += 1;

                        new_start_times.push(Some(reset_start));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt);
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt);
                        }
                        if has_buckets_col {
                            append_opt_u64_slice(&mut new_buckets, cur_buckets_opt);
                        }
                    } else {
                        state.prev_count = cur_count;
                        state.prev_sum = cur_sum;
                        state.prev_buckets.clear();
                        state.prev_buckets.extend_from_slice(cur_buckets);

                        new_start_times.push(Some(state.start_time_ns));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt.map(|v| v.wrapping_sub(state.ref_count)));
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt.map(|v| v - state.ref_sum));
                        }
                        if has_buckets_col {
                            match cur_buckets_opt {
                                Some(slice) => {
                                    for (&cur_b, &ref_b) in
                                        slice.iter().zip(state.ref_buckets.iter())
                                    {
                                        new_buckets
                                            .values()
                                            .append_value(cur_b.wrapping_sub(ref_b));
                                    }
                                    new_buckets.append(true);
                                }
                                None => new_buckets.append(false),
                            }
                        }
                    }
                }
            }
        }

        if kept_count == 0 {
            let _ = records.remove(ArrowPayloadType::HistogramDataPoints);
            let _ = records.remove(ArrowPayloadType::HistogramDpAttrs);
            let _ = records.remove(ArrowPayloadType::HistogramDpExemplars);
            let _ = records.remove(ArrowPayloadType::HistogramDpExemplarAttrs);
            return Ok(stats);
        }

        let keep_filter = (kept_count < num_rows).then(|| BooleanArray::from(keep_flags));
        let start_col: ArrayRef =
            Arc::new(TimestampNanosecondArray::from(new_start_times).with_timezone_utc());
        let mut replacements: Vec<(&str, ArrayRef)> = Vec::with_capacity(3);
        if counts.is_some() {
            replacements.push((consts::COUNT, Arc::new(UInt64Array::from(new_counts))));
        }
        if sums.is_some() {
            replacements.push((consts::SUM, Arc::new(Float64Array::from(new_sums))));
        }
        if has_buckets_col {
            replacements.push((
                consts::HISTOGRAM_BUCKET_COUNTS,
                Arc::new(new_buckets.finish()),
            ));
        }

        let new_rb =
            rebuild_data_point_batch(dp_rb, keep_filter.as_ref(), start_col, &replacements)?;
        records.set(ArrowPayloadType::HistogramDataPoints, new_rb)?;

        if kept_count < num_rows {
            self.filter_dp_children(
                records,
                ArrowPayloadType::HistogramDataPoints,
                ArrowPayloadType::HistogramDpAttrs,
                Some((
                    ArrowPayloadType::HistogramDpExemplars,
                    ArrowPayloadType::HistogramDpExemplarAttrs,
                )),
            )?;
        }

        Ok(stats)
    }

    fn adjust_exp_histogram_data_points(
        &mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<BatchAdjustStats, PdataError> {
        let Some(dp_rb) = records.get(ArrowPayloadType::ExpHistogramDataPoints) else {
            return Ok(BatchAdjustStats::default());
        };
        let num_rows = dp_rb.num_rows();
        if num_rows == 0 {
            return Ok(BatchAdjustStats::default());
        }

        let metrics_rb = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .expect("UnivariateMetrics checked by caller");
        let metric_names = MaybeDictArrayAccessor::<StringArray>::try_new(get_required_array(
            metrics_rb,
            consts::NAME,
        )?)?;

        self.dp_attr_hasher
            .populate_u32(records.get(ArrowPayloadType::ExpHistogramDpAttrs))?;

        let common = CommonDpColumns::try_new(dp_rb)?;
        let counts = dp_rb
            .column_by_name(consts::COUNT)
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>());
        let sums = dp_rb
            .column_by_name(consts::SUM)
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>());
        let zero_counts = dp_rb
            .column_by_name(consts::EXP_HISTOGRAM_ZERO_COUNT)
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>());
        let scales = dp_rb
            .column_by_name(consts::EXP_HISTOGRAM_SCALE)
            .and_then(|c| c.as_any().downcast_ref::<Int32Array>());
        let pos_acc = dp_rb
            .column_by_name(consts::EXP_HISTOGRAM_POSITIVE)
            .map(|c| ExpBucketsColumnAccessor::try_new(c, consts::EXP_HISTOGRAM_POSITIVE))
            .transpose()?;
        let neg_acc = dp_rb
            .column_by_name(consts::EXP_HISTOGRAM_NEGATIVE)
            .map(|c| ExpBucketsColumnAccessor::try_new(c, consts::EXP_HISTOGRAM_NEGATIVE))
            .transpose()?;

        let mut new_start_times: Vec<Option<i64>> = Vec::with_capacity(num_rows);
        let mut new_counts: Vec<Option<u64>> =
            Vec::with_capacity(if counts.is_some() { num_rows } else { 0 });
        let mut new_sums: Vec<Option<f64>> =
            Vec::with_capacity(if sums.is_some() { num_rows } else { 0 });
        let mut new_zero_counts: Vec<Option<u64>> =
            Vec::with_capacity(if zero_counts.is_some() { num_rows } else { 0 });
        let mut pos_builder = pos_acc.as_ref().map(|a| a.new_builder(num_rows));
        let mut neg_builder = neg_acc.as_ref().map(|a| a.new_builder(num_rows));

        let mut keep_flags: Vec<bool> = Vec::with_capacity(num_rows);
        let mut kept_count = 0usize;
        let mut stats = BatchAdjustStats::default();

        let mut active_parent_id: Option<u16> = None;
        let mut active_map: Option<&mut HashMap<u128, ExpHistogramSeriesState>> = None;

        for row in 0..num_rows {
            let parent_id = common.parent_id_at(row);
            if active_parent_id != Some(parent_id) {
                active_parent_id = Some(parent_id);
                active_map = self
                    .metric_lookup
                    .get(parent_id, METRIC_TYPE_EXP_HISTOGRAM)
                    .map(|slot| {
                        let name = metric_names.str_at(slot.metric_row).unwrap_or("");
                        self.cache
                            .exp_histogram_series_mut(slot.resource_hash, name)
                    });
            }

            let orig_start_opt = common.start_time_at(row);
            let time_ns = common.time_at(row);
            let cur_count_opt = counts.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_sum_opt = sums.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_zero_opt = zero_counts.and_then(|c| c.is_valid(row).then(|| c.value(row)));
            let cur_scale = scales
                .and_then(|c| c.is_valid(row).then(|| c.value(row)))
                .unwrap_or(0);
            let cur_pos = pos_acc.as_ref().map(|a| a.row_at(row)).unwrap_or_default();
            let cur_neg = neg_acc.as_ref().map(|a| a.row_at(row)).unwrap_or_default();

            let Some(series_map) = active_map.as_deref_mut() else {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if counts.is_some() {
                    new_counts.push(cur_count_opt);
                }
                if sums.is_some() {
                    new_sums.push(cur_sum_opt);
                }
                if zero_counts.is_some() {
                    new_zero_counts.push(cur_zero_opt);
                }
                if let Some(b) = pos_builder.as_mut() {
                    b.append_unadjusted(cur_pos);
                }
                if let Some(b) = neg_builder.as_mut() {
                    b.append_unadjusted(cur_neg);
                }
                continue;
            };

            let orig_start_ns = orig_start_opt.unwrap_or(0);
            if orig_start_ns != 0 && orig_start_ns != time_ns {
                keep_flags.push(true);
                kept_count += 1;
                new_start_times.push(orig_start_opt);
                if counts.is_some() {
                    new_counts.push(cur_count_opt);
                }
                if sums.is_some() {
                    new_sums.push(cur_sum_opt);
                }
                if zero_counts.is_some() {
                    new_zero_counts.push(cur_zero_opt);
                }
                if let Some(b) = pos_builder.as_mut() {
                    b.append_unadjusted(cur_pos);
                }
                if let Some(b) = neg_builder.as_mut() {
                    b.append_unadjusted(cur_neg);
                }
                continue;
            }

            let dp_attr_hash = common
                .id_at(row)
                .map_or(0, |id| self.dp_attr_hasher.get(id));
            let cur_count = cur_count_opt.unwrap_or(0);
            let cur_sum = cur_sum_opt.unwrap_or(0.0);
            let cur_zero = cur_zero_opt.unwrap_or(0);
            let pos_counts = cur_pos.counts.unwrap_or(&[]);
            let neg_counts = cur_neg.counts.unwrap_or(&[]);

            match series_map.entry(dp_attr_hash) {
                Entry::Vacant(vacant) => {
                    let _ = vacant.insert(ExpHistogramSeriesState {
                        mark: true,
                        start_time_ns: time_ns,
                        scale: cur_scale,
                        ref_count: cur_count,
                        ref_sum: cur_sum,
                        ref_zero_count: cur_zero,
                        prev_count: cur_count,
                        prev_sum: cur_sum,
                        prev_zero_count: cur_zero,
                        ref_positive: ExpBucketsState {
                            offset: cur_pos.offset,
                            counts: pos_counts.to_vec(),
                        },
                        prev_positive: ExpBucketsState {
                            offset: cur_pos.offset,
                            counts: pos_counts.to_vec(),
                        },
                        ref_negative: ExpBucketsState {
                            offset: cur_neg.offset,
                            counts: neg_counts.to_vec(),
                        },
                        prev_negative: ExpBucketsState {
                            offset: cur_neg.offset,
                            counts: neg_counts.to_vec(),
                        },
                    });
                    keep_flags.push(false);
                    stats.dropped_initial += 1;
                }
                Entry::Occupied(mut occupied) => {
                    let state = occupied.get_mut();
                    state.mark = true;
                    keep_flags.push(true);
                    kept_count += 1;
                    stats.adjusted_points += 1;

                    if common.is_no_recorded_value(row) {
                        new_start_times.push(Some(state.start_time_ns));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt);
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt);
                        }
                        if zero_counts.is_some() {
                            new_zero_counts.push(cur_zero_opt);
                        }
                        if let Some(b) = pos_builder.as_mut() {
                            b.append_unadjusted(cur_pos);
                        }
                        if let Some(b) = neg_builder.as_mut() {
                            b.append_unadjusted(cur_neg);
                        }
                        continue;
                    }

                    let is_reset = cur_count < state.prev_count
                        || cur_sum < state.prev_sum
                        || cur_zero < state.prev_zero_count
                        || cur_scale != state.scale
                        || pos_counts.len() != state.prev_positive.counts.len()
                        || neg_counts.len() != state.prev_negative.counts.len()
                        || pos_counts
                            .iter()
                            .zip(state.prev_positive.counts.iter())
                            .any(|(&cur_b, &prev_b)| cur_b < prev_b)
                        || neg_counts
                            .iter()
                            .zip(state.prev_negative.counts.iter())
                            .any(|(&cur_b, &prev_b)| cur_b < prev_b);

                    if is_reset {
                        let reset_start = time_ns.saturating_sub(RESET_START_TIME_OFFSET_NS);
                        state.start_time_ns = reset_start;
                        state.scale = cur_scale;
                        state.ref_count = 0;
                        state.ref_sum = 0.0;
                        state.ref_zero_count = 0;
                        state.prev_count = cur_count;
                        state.prev_sum = cur_sum;
                        state.prev_zero_count = cur_zero;

                        state.ref_positive.offset = cur_pos.offset;
                        state.ref_positive.counts.clear();
                        state.ref_positive.counts.resize(pos_counts.len(), 0);
                        state.prev_positive.offset = cur_pos.offset;
                        state.prev_positive.counts.clear();
                        state.prev_positive.counts.extend_from_slice(pos_counts);

                        state.ref_negative.offset = cur_neg.offset;
                        state.ref_negative.counts.clear();
                        state.ref_negative.counts.resize(neg_counts.len(), 0);
                        state.prev_negative.offset = cur_neg.offset;
                        state.prev_negative.counts.clear();
                        state.prev_negative.counts.extend_from_slice(neg_counts);
                        stats.resets_detected += 1;

                        new_start_times.push(Some(reset_start));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt);
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt);
                        }
                        if zero_counts.is_some() {
                            new_zero_counts.push(cur_zero_opt);
                        }
                        if let Some(b) = pos_builder.as_mut() {
                            b.append_unadjusted(cur_pos);
                        }
                        if let Some(b) = neg_builder.as_mut() {
                            b.append_unadjusted(cur_neg);
                        }
                    } else {
                        state.prev_count = cur_count;
                        state.prev_sum = cur_sum;
                        state.prev_zero_count = cur_zero;
                        state.prev_positive.offset = cur_pos.offset;
                        state.prev_positive.counts.clear();
                        state.prev_positive.counts.extend_from_slice(pos_counts);
                        state.prev_negative.offset = cur_neg.offset;
                        state.prev_negative.counts.clear();
                        state.prev_negative.counts.extend_from_slice(neg_counts);

                        new_start_times.push(Some(state.start_time_ns));
                        if counts.is_some() {
                            new_counts.push(cur_count_opt.map(|v| v.wrapping_sub(state.ref_count)));
                        }
                        if sums.is_some() {
                            new_sums.push(cur_sum_opt.map(|v| v - state.ref_sum));
                        }
                        if zero_counts.is_some() {
                            new_zero_counts
                                .push(cur_zero_opt.map(|v| v.wrapping_sub(state.ref_zero_count)));
                        }
                        if let Some(b) = pos_builder.as_mut() {
                            b.append_subtracted(cur_pos, &state.ref_positive);
                        }
                        if let Some(b) = neg_builder.as_mut() {
                            b.append_subtracted(cur_neg, &state.ref_negative);
                        }
                    }
                }
            }
        }

        if kept_count == 0 {
            let _ = records.remove(ArrowPayloadType::ExpHistogramDataPoints);
            let _ = records.remove(ArrowPayloadType::ExpHistogramDpAttrs);
            let _ = records.remove(ArrowPayloadType::ExpHistogramDpExemplars);
            let _ = records.remove(ArrowPayloadType::ExpHistogramDpExemplarAttrs);
            return Ok(stats);
        }

        let keep_filter = (kept_count < num_rows).then(|| BooleanArray::from(keep_flags));
        let start_col: ArrayRef =
            Arc::new(TimestampNanosecondArray::from(new_start_times).with_timezone_utc());
        let mut replacements: Vec<(&str, ArrayRef)> = Vec::with_capacity(5);
        if counts.is_some() {
            replacements.push((consts::COUNT, Arc::new(UInt64Array::from(new_counts))));
        }
        if sums.is_some() {
            replacements.push((consts::SUM, Arc::new(Float64Array::from(new_sums))));
        }
        if zero_counts.is_some() {
            replacements.push((
                consts::EXP_HISTOGRAM_ZERO_COUNT,
                Arc::new(UInt64Array::from(new_zero_counts)),
            ));
        }
        if let (Some(acc), Some(builder)) = (pos_acc.as_ref(), pos_builder) {
            replacements.push((
                consts::EXP_HISTOGRAM_POSITIVE,
                acc.finish_column(builder, keep_filter.as_ref())?,
            ));
        }
        if let (Some(acc), Some(builder)) = (neg_acc.as_ref(), neg_builder) {
            replacements.push((
                consts::EXP_HISTOGRAM_NEGATIVE,
                acc.finish_column(builder, keep_filter.as_ref())?,
            ));
        }

        let new_rb =
            rebuild_data_point_batch(dp_rb, keep_filter.as_ref(), start_col, &replacements)?;
        records.set(ArrowPayloadType::ExpHistogramDataPoints, new_rb)?;

        if kept_count < num_rows {
            self.filter_dp_children(
                records,
                ArrowPayloadType::ExpHistogramDataPoints,
                ArrowPayloadType::ExpHistogramDpAttrs,
                Some((
                    ArrowPayloadType::ExpHistogramDpExemplars,
                    ArrowPayloadType::ExpHistogramDpExemplarAttrs,
                )),
            )?;
        }

        Ok(stats)
    }

    /// Filters data point child tables (`*DpAttrs`, `*DpExemplars`, `*DpExemplarAttrs`) when initial
    /// points are dropped from `parent_pt`.
    fn filter_dp_children(
        &mut self,
        records: &mut OtapArrowRecords,
        parent_pt: ArrowPayloadType,
        attrs_pt: ArrowPayloadType,
        exemplars_pts: Option<(ArrowPayloadType, ArrowPayloadType)>,
    ) -> Result<(), PdataError> {
        let has_attrs = records.get(attrs_pt).is_some();
        let has_exemplars = exemplars_pts.is_some_and(|(ex_pt, _)| records.get(ex_pt).is_some());
        if !has_attrs && !has_exemplars {
            if let Some((_, ex_attrs_pt)) = exemplars_pts {
                let _ = records.remove(ex_attrs_pt);
            }
            return Ok(());
        }

        let mut id_bitmap = self.id_bitmap_pool.acquire();
        let res = (|| -> Result<(), PdataError> {
            let Some(parent_rb) = records.get(parent_pt) else {
                let _ = records.remove(attrs_pt);
                if let Some((ex_pt, ex_attrs_pt)) = exemplars_pts {
                    let _ = records.remove(ex_pt);
                    let _ = records.remove(ex_attrs_pt);
                }
                return Ok(());
            };

            let Some(id_col) = UInt32Type::get_id_col_from_parent(parent_rb, attrs_pt)? else {
                let _ = records.remove(attrs_pt);
                if let Some((ex_pt, ex_attrs_pt)) = exemplars_pts {
                    let _ = records.remove(ex_pt);
                    let _ = records.remove(ex_attrs_pt);
                }
                return Ok(());
            };
            id_bitmap.populate(id_col.iter().flatten());

            filter_single_child_table(records, attrs_pt, &id_bitmap)?;

            if let Some((ex_pt, ex_attrs_pt)) = exemplars_pts {
                filter_single_child_table(records, ex_pt, &id_bitmap)?;
                if records.get(ex_attrs_pt).is_some() {
                    if let Some(ex_rb) = records.get(ex_pt)
                        && let Some(ex_id_col) =
                            UInt32Type::get_id_col_from_parent(ex_rb, ex_attrs_pt)?
                    {
                        id_bitmap.populate(ex_id_col.iter().flatten());
                        filter_single_child_table(records, ex_attrs_pt, &id_bitmap)?;
                    } else {
                        let _ = records.remove(ex_attrs_pt);
                    }
                }
            }
            Ok(())
        })();
        self.id_bitmap_pool.release(id_bitmap);
        res
    }

    /// Removes any eligible `UnivariateMetrics` rows that no longer have any surviving data points
    /// after initial points were dropped.
    fn prune_empty_metrics(&mut self, records: &mut OtapArrowRecords) -> Result<(), PdataError> {
        let Some(metrics_rb) = records.get(ArrowPayloadType::UnivariateMetrics) else {
            return Ok(());
        };
        let num_metrics = metrics_rb.num_rows();
        if num_metrics == 0 {
            return Ok(());
        }

        let mut surviving_ids = self.id_bitmap_pool.acquire();
        for dp_pt in [
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
        ] {
            if let Some(dp_rb) = records.get(dp_pt)
                && let Some(parent_col) = dp_rb.column_by_name(consts::PARENT_ID)
                && let Some(parent_ids) = parent_col.as_any().downcast_ref::<UInt16Array>()
            {
                for row in 0..parent_ids.len() {
                    if parent_ids.is_valid(row) {
                        surviving_ids.insert(u32::from(parent_ids.value(row)));
                    }
                }
            }
        }

        let id_arr = get_required_array(metrics_rb, consts::ID)?;
        let metric_ids = id_arr
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| PdataError::ColumnDataTypeMismatch {
                name: consts::ID.into(),
                actual: id_arr.data_type().clone(),
                expect: DataType::UInt16,
            })?;

        let mut selection_builder = BooleanBuilder::with_capacity(num_metrics);
        let mut kept_metrics = 0usize;
        for row in 0..num_metrics {
            let keep = !self.metric_lookup.is_eligible_row(row)
                || (metric_ids.is_valid(row)
                    && surviving_ids.contains(u32::from(metric_ids.value(row))));
            if keep {
                kept_metrics += 1;
            }
            selection_builder.append_value(keep);
        }
        self.id_bitmap_pool.release(surviving_ids);

        if kept_metrics == 0 {
            *records = OtapArrowRecords::Metrics(Metrics::default());
        } else if kept_metrics < num_metrics {
            let selection = selection_builder.finish();
            *records = filter_otap_batch(&selection, records, &mut self.id_bitmap_pool)?;
        }

        Ok(())
    }
}

fn filter_single_child_table(
    records: &mut OtapArrowRecords,
    child_pt: ArrowPayloadType,
    id_bitmap: &IdBitmap,
) -> Result<(), PdataError> {
    let Some(child_rb) = records.get(child_pt) else {
        return Ok(());
    };
    let parent_ids =
        child_rb
            .column_by_name(consts::PARENT_ID)
            .ok_or_else(|| PdataError::ColumnNotFound {
                name: consts::PARENT_ID.into(),
            })?;
    let selection = UInt32Type::build_selection_vec(parent_ids, id_bitmap)?;
    let kept = selection.true_count();
    if kept == 0 {
        let _ = records.remove(child_pt);
    } else if kept < child_rb.num_rows() {
        let filtered = filter_record_batch(child_rb, &selection)
            .map_err(|source| PdataError::ColumnLengthMismatch { source })?;
        records.set(child_pt, filtered)?;
    }
    Ok(())
}

/// Common columns shared across all four OTAP data point tables.
struct CommonDpColumns<'a> {
    parent_ids: &'a UInt16Array,
    ids: Option<&'a UInt32Array>,
    start_times: Option<&'a TimestampNanosecondArray>,
    times: Option<&'a TimestampNanosecondArray>,
    flags: Option<&'a UInt32Array>,
}

impl<'a> CommonDpColumns<'a> {
    fn try_new(rb: &'a RecordBatch) -> Result<Self, PdataError> {
        let parent_arr = get_required_array(rb, consts::PARENT_ID)?;
        let parent_ids = parent_arr
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| PdataError::ColumnDataTypeMismatch {
                name: consts::PARENT_ID.into(),
                actual: parent_arr.data_type().clone(),
                expect: DataType::UInt16,
            })?;

        let ids = rb
            .column_by_name(consts::ID)
            .and_then(|c| c.as_any().downcast_ref::<UInt32Array>());
        let start_times = rb
            .column_by_name(consts::START_TIME_UNIX_NANO)
            .and_then(|c| c.as_any().downcast_ref::<TimestampNanosecondArray>());
        let times = rb
            .column_by_name(consts::TIME_UNIX_NANO)
            .and_then(|c| c.as_any().downcast_ref::<TimestampNanosecondArray>());
        let flags = rb
            .column_by_name(consts::FLAGS)
            .and_then(|c| c.as_any().downcast_ref::<UInt32Array>());

        Ok(Self {
            parent_ids,
            ids,
            start_times,
            times,
            flags,
        })
    }

    #[inline]
    fn parent_id_at(&self, row: usize) -> u16 {
        self.parent_ids.value(row)
    }

    #[inline]
    fn id_at(&self, row: usize) -> Option<u32> {
        self.ids.and_then(|c| c.is_valid(row).then(|| c.value(row)))
    }

    #[inline]
    fn start_time_at(&self, row: usize) -> Option<i64> {
        self.start_times
            .and_then(|c| c.is_valid(row).then(|| c.value(row)))
    }

    #[inline]
    fn time_at(&self, row: usize) -> i64 {
        self.times
            .and_then(|c| c.is_valid(row).then(|| c.value(row)))
            .unwrap_or(0)
    }

    #[inline]
    fn is_no_recorded_value(&self, row: usize) -> bool {
        self.flags
            .and_then(|c| c.is_valid(row).then(|| c.value(row)))
            .is_some_and(|f| (f & FLAG_NO_RECORDED_VALUE) != 0)
    }
}

/// Zero-allocation slice accessor for `List<UInt64>` columns (`bucket_counts`).
struct U64ListSliceAccessor<'a> {
    list: &'a ListArray,
    values: &'a [u64],
}

impl<'a> U64ListSliceAccessor<'a> {
    fn try_new(col: &'a ArrayRef) -> Result<Self, PdataError> {
        let list = col.as_any().downcast_ref::<ListArray>().ok_or_else(|| {
            PdataError::InvalidListArray {
                expect_oneof: vec![DataType::UInt64],
                actual: col.data_type().clone(),
            }
        })?;
        let u64_arr = list
            .values()
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or_else(|| PdataError::InvalidListArray {
                expect_oneof: vec![DataType::UInt64],
                actual: list.values().data_type().clone(),
            })?;
        Ok(Self {
            list,
            values: u64_arr.values(),
        })
    }

    #[inline]
    fn slice_at(&self, row: usize) -> Option<&'a [u64]> {
        if !self.list.is_valid(row) {
            return None;
        }
        let offsets = self.list.value_offsets();
        let start = offsets[row] as usize;
        let end = offsets[row + 1] as usize;
        Some(&self.values[start..end])
    }
}

/// Zero-allocation slice accessor for `List<Float64>` columns (`explicit_bounds`).
struct F64ListSliceAccessor<'a> {
    list: &'a ListArray,
    values: &'a [f64],
}

impl<'a> F64ListSliceAccessor<'a> {
    fn try_new(col: &'a ArrayRef) -> Result<Self, PdataError> {
        let list = col.as_any().downcast_ref::<ListArray>().ok_or_else(|| {
            PdataError::InvalidListArray {
                expect_oneof: vec![DataType::Float64],
                actual: col.data_type().clone(),
            }
        })?;
        let f64_arr = list
            .values()
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| PdataError::InvalidListArray {
                expect_oneof: vec![DataType::Float64],
                actual: list.values().data_type().clone(),
            })?;
        Ok(Self {
            list,
            values: f64_arr.values(),
        })
    }

    #[inline]
    fn slice_at(&self, row: usize) -> Option<&'a [f64]> {
        if !self.list.is_valid(row) {
            return None;
        }
        let offsets = self.list.value_offsets();
        let start = offsets[row] as usize;
        let end = offsets[row + 1] as usize;
        Some(&self.values[start..end])
    }
}

#[inline]
fn bounds_eq_bits(bounds: &[f64], expected_bits: &[u64]) -> bool {
    bounds.len() == expected_bits.len()
        && bounds
            .iter()
            .zip(expected_bits.iter())
            .all(|(b, &bits)| b.to_bits() == bits)
}

#[inline]
fn append_opt_u64_slice(builder: &mut ListBuilder<UInt64Builder>, opt_slice: Option<&[u64]>) {
    match opt_slice {
        Some(slice) => {
            builder.values().append_slice(slice);
            builder.append(true);
        }
        None => builder.append(false),
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ExpBucketsRow<'a> {
    struct_valid: bool,
    offset: i32,
    counts: Option<&'a [u64]>,
}

/// Accessor for the `positive` or `negative` `StructArray` column on `ExpHistogramDataPoints`.
struct ExpBucketsColumnAccessor<'a> {
    struct_arr: &'a StructArray,
    offsets: Option<&'a Int32Array>,
    bucket_counts: Option<U64ListSliceAccessor<'a>>,
}

impl<'a> ExpBucketsColumnAccessor<'a> {
    fn try_new(col: &'a ArrayRef, col_name: &str) -> Result<Self, PdataError> {
        let struct_arr = col.as_any().downcast_ref::<StructArray>().ok_or_else(|| {
            PdataError::ColumnDataTypeMismatch {
                name: col_name.into(),
                actual: col.data_type().clone(),
                expect: DataType::Struct(arrow::datatypes::Fields::empty()),
            }
        })?;
        let offsets = struct_arr
            .column_by_name(consts::EXP_HISTOGRAM_OFFSET)
            .and_then(|c| c.as_any().downcast_ref::<Int32Array>());
        let bucket_counts = struct_arr
            .column_by_name(consts::EXP_HISTOGRAM_BUCKET_COUNTS)
            .map(U64ListSliceAccessor::try_new)
            .transpose()?;
        Ok(Self {
            struct_arr,
            offsets,
            bucket_counts,
        })
    }

    #[inline]
    fn row_at(&self, row: usize) -> ExpBucketsRow<'a> {
        if !self.struct_arr.is_valid(row) {
            return ExpBucketsRow {
                struct_valid: false,
                offset: 0,
                counts: None,
            };
        }
        let offset = self
            .offsets
            .and_then(|c| c.is_valid(row).then(|| c.value(row)))
            .unwrap_or(0);
        let counts = self.bucket_counts.as_ref().and_then(|c| c.slice_at(row));
        ExpBucketsRow {
            struct_valid: true,
            offset,
            counts,
        }
    }

    fn new_builder(&self, capacity: usize) -> ExpBucketsColumnBuilder {
        ExpBucketsColumnBuilder {
            has_bucket_counts: self.bucket_counts.is_some(),
            has_struct_nulls: self.struct_arr.nulls().is_some(),
            bucket_counts: ListBuilder::with_capacity(UInt64Builder::new(), capacity),
            struct_nulls: NullBufferBuilder::new(capacity),
        }
    }

    fn finish_column(
        &self,
        mut builder: ExpBucketsColumnBuilder,
        keep_filter: Option<&BooleanArray>,
    ) -> Result<ArrayRef, PdataError> {
        let new_buckets_col: Option<ArrayRef> = builder
            .has_bucket_counts
            .then(|| Arc::new(builder.bucket_counts.finish()) as ArrayRef);

        let mut fields = Vec::with_capacity(self.struct_arr.fields().len());
        let mut columns = Vec::with_capacity(self.struct_arr.columns().len());

        for (field, col) in self
            .struct_arr
            .fields()
            .iter()
            .zip(self.struct_arr.columns())
        {
            if field.name().as_str() == consts::EXP_HISTOGRAM_BUCKET_COUNTS
                && let Some(new_col) = new_buckets_col.as_ref()
            {
                fields.push(Arc::new(
                    field
                        .as_ref()
                        .clone()
                        .with_data_type(new_col.data_type().clone()),
                ));
                columns.push(Arc::clone(new_col));
            } else {
                fields.push(Arc::clone(field));
                let filtered_col = match keep_filter {
                    Some(filter) => arrow::compute::filter(col, filter)
                        .map_err(|source| PdataError::ColumnLengthMismatch { source })?,
                    None => Arc::clone(col),
                };
                columns.push(filtered_col);
            }
        }

        let nulls = if keep_filter.is_some() {
            builder.struct_nulls.finish()
        } else {
            self.struct_arr.nulls().cloned()
        };

        let struct_arr = StructArray::try_new(fields.into(), columns, nulls)
            .map_err(|source| PdataError::ColumnLengthMismatch { source })?;
        Ok(Arc::new(struct_arr))
    }
}

struct ExpBucketsColumnBuilder {
    has_bucket_counts: bool,
    has_struct_nulls: bool,
    bucket_counts: ListBuilder<UInt64Builder>,
    struct_nulls: NullBufferBuilder,
}

impl ExpBucketsColumnBuilder {
    #[inline]
    fn append_unadjusted(&mut self, row: ExpBucketsRow<'_>) {
        if self.has_struct_nulls {
            self.struct_nulls.append(row.struct_valid);
        }
        if self.has_bucket_counts {
            if row.struct_valid {
                append_opt_u64_slice(&mut self.bucket_counts, row.counts);
            } else {
                self.bucket_counts.append(false);
            }
        }
    }

    #[inline]
    fn append_subtracted(&mut self, row: ExpBucketsRow<'_>, reference: &ExpBucketsState) {
        if self.has_struct_nulls {
            self.struct_nulls.append(row.struct_valid);
        }
        if !self.has_bucket_counts {
            return;
        }
        if !row.struct_valid {
            self.bucket_counts.append(false);
            return;
        }
        let Some(cur_counts) = row.counts else {
            self.bucket_counts.append(false);
            return;
        };

        let offset_diff = (row.offset as isize) - (reference.offset as isize);
        let ref_counts = &reference.counts;
        for (i, &cur_val) in cur_counts.iter().enumerate() {
            let ref_idx = (i as isize) + offset_diff;
            let adjusted = if ref_idx >= 0 && (ref_idx as usize) < ref_counts.len() {
                cur_val.wrapping_sub(ref_counts[ref_idx as usize])
            } else {
                cur_val
            };
            self.bucket_counts.values().append_value(adjusted);
        }
        self.bucket_counts.append(true);
    }
}

/// Rebuilds a data point [`RecordBatch`] by replacing `start_time_unix_nano` (or inserting it right
/// after `parent_id` if absent) and replacing adjusted value columns, while either sharing
/// unchanged columns via `Arc::clone` (when `keep_filter` is `None`) or filtering them by
/// `keep_filter`.
fn rebuild_data_point_batch(
    rb: &RecordBatch,
    keep_filter: Option<&BooleanArray>,
    start_time_col: ArrayRef,
    replacements: &[(&str, ArrayRef)],
) -> Result<RecordBatch, PdataError> {
    let old_schema = rb.schema();
    let num_cols = old_schema.fields().len() + 1;
    let mut fields: Vec<Arc<Field>> = Vec::with_capacity(num_cols);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(num_cols);
    let mut has_start_time = false;

    for (col_idx, field) in old_schema.fields().iter().enumerate() {
        let name = field.name().as_str();
        if name == consts::START_TIME_UNIX_NANO {
            has_start_time = true;
            fields.push(Arc::new(
                field
                    .as_ref()
                    .clone()
                    .with_data_type(start_time_col.data_type().clone())
                    .with_nullable(true),
            ));
            columns.push(Arc::clone(&start_time_col));
        } else if let Some((_, rep_col)) = replacements.iter().find(|(n, _)| *n == name) {
            fields.push(Arc::new(
                field
                    .as_ref()
                    .clone()
                    .with_data_type(rep_col.data_type().clone()),
            ));
            columns.push(Arc::clone(rep_col));
        } else {
            fields.push(Arc::clone(field));
            let col = rb.column(col_idx);
            let out_col = match keep_filter {
                Some(filter) => arrow::compute::filter(col, filter)
                    .map_err(|source| PdataError::ColumnLengthMismatch { source })?,
                None => Arc::clone(col),
            };
            columns.push(out_col);
        }
    }

    if !has_start_time {
        let insert_idx = fields
            .iter()
            .position(|f| f.name().as_str() == consts::PARENT_ID)
            .map_or(0, |idx| idx + 1);
        fields.insert(
            insert_idx,
            Arc::new(Field::new(
                consts::START_TIME_UNIX_NANO,
                start_time_col.data_type().clone(),
                true,
            )),
        );
        columns.insert(insert_idx, start_time_col);
    }

    let new_schema = Arc::new(Schema::new_with_metadata(
        fields,
        old_schema.metadata().clone(),
    ));
    RecordBatch::try_new(new_schema, columns)
        .map_err(|source| PdataError::ColumnLengthMismatch { source })
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for MetricStartTimeProcessor {
    fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        NativePdataProcessor::runtime_requirements(self)
    }

    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        NativeProcessorAdapter::process(self, msg, effect_handler).await
    }
}

#[async_trait(?Send)]
impl NativePdataProcessor for MetricStartTimeProcessor {
    fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        ProcessorRuntimeRequirements::none().with_drop_decisions()
    }

    async fn process_control(
        &mut self,
        control: NodeControlMsg<OtapPdata>,
        _effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if let NodeControlMsg::CollectTelemetry {
            mut metrics_reporter,
        } = control
        {
            _ = metrics_reporter.report_measurement(&mut self.metrics);
            self.compute_duration.report(&mut metrics_reporter);
        }
        Ok(())
    }

    async fn process_native(
        &mut self,
        arrow_pdata: OtapArrowPdata,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let signal = arrow_pdata.signal_type();
        if signal != SignalType::Metrics {
            effect_handler
                .send_message_with_source_node(arrow_pdata.into_pdata())
                .await?;
            return Ok(());
        }

        let (context, mut arrow_records) = arrow_pdata.into_parts();
        arrow_records.decode_transport_optimized_ids()?;

        let stats = effect_handler.timed(&self.compute_duration, || -> Result<_, Error> {
            self.engine
                .process_metrics_batch(&mut arrow_records, Instant::now())
                .map_err(|e| {
                    let source_detail = format_error_sources(&e);
                    Error::ProcessorError {
                        processor: effect_handler.processor_id(),
                        kind: ProcessorErrorKind::Other,
                        error: format!("MetricStartTime error: {e}"),
                        source_detail,
                    }
                })
        })?;

        let metric = self.metrics.with(SignalAttributes { signal });
        metric.dropped_items.add(stats.dropped_initial);
        metric.adjusted_data_points.add(stats.adjusted_points);
        metric.resets_detected.add(stats.resets_detected);

        if stats.dropped_initial > 0 {
            effect_handler.record_flow_dropped_items(signal, stats.dropped_initial);
        }

        let kept_items = arrow_records.num_items();
        let mut pdata = OtapPdata::new(context, arrow_records.into());
        if kept_items == 0 {
            pdata.complete_processor_without_output(effect_handler);
            effect_handler.notify_ack(AckMsg::new(pdata)).await?;
        } else {
            effect_handler.send_message_with_source_node(pdata).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::testing::processor::TestRuntime;
    use otel_arrow_dfe_engine::testing::test_node;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, DataPointFlags, ExponentialHistogram,
        ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint, Metric, MetricsData,
        NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_dfe_pdata::testing::equiv::assert_equivalent;
    use otel_arrow_dfe_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;

    fn test_processor(gc_interval: Duration) -> MetricStartTimeProcessor {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        MetricStartTimeProcessor::new(
            Config {
                strategy: config::Strategy::SubtractInitialPoint,
                gc_interval,
            },
            pipeline_ctx,
        )
        .unwrap()
    }

    fn kv_str(k: &str, v: &str) -> KeyValue {
        KeyValue::new(k, AnyValue::new_string(v))
    }

    fn run_batch(
        proc: &mut MetricStartTimeProcessor,
        input: MetricsData,
        now: Instant,
    ) -> (Option<MetricsData>, BatchAdjustStats) {
        let mut records = otlp_to_otap(&OtlpProtoMessage::Metrics(input));
        let stats = proc
            .engine
            .process_metrics_batch(&mut records, now)
            .unwrap();
        if records.num_items() == 0 {
            return (None, stats);
        }
        let OtlpProtoMessage::Metrics(out) = otap_to_otlp(&records) else {
            panic!("expected metrics");
        };
        (Some(out), stats)
    }

    #[test]
    fn test_cumulative_sum_subtract_initial_and_reset() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();

        let make_sum_batch = |ts: u64, val_a: f64, val_b: f64| {
            MetricsData::new(vec![ResourceMetrics::new(
                Resource::build()
                    .attributes(vec![kv_str("service.name", "checkout")])
                    .finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().name("scope").finish(),
                    vec![
                        Metric::build()
                            .name("http_requests_total")
                            .data_sum(Sum::new(
                                AggregationTemporality::Cumulative,
                                true,
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(ts)
                                        .attributes(vec![kv_str("method", "GET")])
                                        .value_double(val_a)
                                        .finish(),
                                    NumberDataPoint::build()
                                        .time_unix_nano(ts)
                                        .attributes(vec![kv_str("method", "POST")])
                                        .value_double(val_b)
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )])
        };

        // 1. Initial scrape: both points are cached as reference values and dropped.
        let (out1, stats1) = run_batch(&mut proc, make_sum_batch(10_000_000, 100.0, 50.0), t0);
        assert!(out1.is_none());
        assert_eq!(stats1.dropped_initial, 2);
        assert_eq!(stats1.adjusted_points, 0);

        // 2. Second scrape: values have initial reference subtracted and start_time = 10_000_000.
        let (out2, stats2) = run_batch(&mut proc, make_sum_batch(20_000_000, 135.0, 55.0), t0);
        assert_eq!(stats2.dropped_initial, 0);
        assert_eq!(stats2.adjusted_points, 2);
        assert_eq!(stats2.resets_detected, 0);

        let expected2 = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build()
                .attributes(vec![kv_str("service.name", "checkout")])
                .finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().name("scope").finish(),
                vec![
                    Metric::build()
                        .name("http_requests_total")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(20_000_000u64)
                                    .attributes(vec![kv_str("method", "GET")])
                                    .value_double(35.0)
                                    .finish(),
                                NumberDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(20_000_000u64)
                                    .attributes(vec![kv_str("method", "POST")])
                                    .value_double(5.0)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        assert_equivalent(
            &[OtlpProtoMessage::Metrics(out2.unwrap())],
            &[OtlpProtoMessage::Metrics(expected2)],
        );

        // 3. Third scrape: GET resets from 135 -> 12, POST continues 55 -> 70.
        let (out3, stats3) = run_batch(&mut proc, make_sum_batch(30_000_000, 12.0, 70.0), t0);
        assert_eq!(stats3.resets_detected, 1);
        assert_eq!(stats3.adjusted_points, 2);

        let expected3 = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build()
                .attributes(vec![kv_str("service.name", "checkout")])
                .finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().name("scope").finish(),
                vec![
                    Metric::build()
                        .name("http_requests_total")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .start_time_unix_nano(29_000_000u64)
                                    .time_unix_nano(30_000_000u64)
                                    .attributes(vec![kv_str("method", "GET")])
                                    .value_double(12.0)
                                    .finish(),
                                NumberDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(30_000_000u64)
                                    .attributes(vec![kv_str("method", "POST")])
                                    .value_double(20.0)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        assert_equivalent(
            &[OtlpProtoMessage::Metrics(out3.unwrap())],
            &[OtlpProtoMessage::Metrics(expected3)],
        );
    }

    #[test]
    fn test_passthrough_gauge_delta_existing_start_time_and_no_recorded_value() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();

        // Scrape 1: a Gauge, a Delta Sum, a Cumulative Sum with existing start_time, and a
        // Cumulative Sum without start_time. Only the last one should be dropped as initial.
        let batch1 = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric::build()
                        .name("cpu_gauge")
                        .data_gauge(Gauge::new(vec![
                            NumberDataPoint::build()
                                .time_unix_nano(10_000_000u64)
                                .value_int(42)
                                .finish(),
                        ]))
                        .finish(),
                    Metric::build()
                        .name("delta_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Delta,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(10_000_000u64)
                                    .value_int(7)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                    Metric::build()
                        .name("already_adjusted_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .start_time_unix_nano(5_000_000u64)
                                    .time_unix_nano(10_000_000u64)
                                    .value_int(99)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                    Metric::build()
                        .name("unadjusted_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(10_000_000u64)
                                    .value_int(100)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);

        let (out1, stats1) = run_batch(&mut proc, batch1, t0);
        assert_eq!(stats1.dropped_initial, 1);
        assert_eq!(stats1.adjusted_points, 0);
        let out1 = out1.unwrap();
        assert_eq!(out1.resource_metrics[0].scope_metrics[0].metrics.len(), 3);

        // Scrape 2 for unadjusted_counter with FLAG_NO_RECORDED_VALUE (staleness marker):
        // start_time_unix_nano is set to 10_000_000, value is untouched, and reference state is preserved.
        let stale_batch = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric::build()
                        .name("unadjusted_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(20_000_000u64)
                                    .flags(DataPointFlags::NoRecordedValueMask as u32)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        let (out2, stats2) = run_batch(&mut proc, stale_batch, t0);
        assert_eq!(stats2.adjusted_points, 1);
        assert_eq!(stats2.resets_detected, 0);
        let expected_stale = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric::build()
                        .name("unadjusted_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(20_000_000u64)
                                    .flags(DataPointFlags::NoRecordedValueMask as u32)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        assert_equivalent(
            &[OtlpProtoMessage::Metrics(out2.unwrap())],
            &[OtlpProtoMessage::Metrics(expected_stale)],
        );

        // Scrape 3 for unadjusted_counter with value 125 -> 125 - 100 = 25 (still subtracted against 100!).
        let resume_batch = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric::build()
                        .name("unadjusted_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .time_unix_nano(30_000_000u64)
                                    .value_int(125)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        let (out3, _) = run_batch(&mut proc, resume_batch, t0);
        let expected_resume = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric::build()
                        .name("unadjusted_counter")
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            vec![
                                NumberDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(30_000_000u64)
                                    .value_int(25)
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        assert_equivalent(
            &[OtlpProtoMessage::Metrics(out3.unwrap())],
            &[OtlpProtoMessage::Metrics(expected_resume)],
        );
    }

    #[test]
    fn test_summary_histogram_and_exp_histogram_adjustment() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();

        let make_complex_batch = |ts: u64,
                                  s_count: u64,
                                  s_sum: f64,
                                  h_count: u64,
                                  h_sum: f64,
                                  h_buckets: Vec<u64>,
                                  eh_count: u64,
                                  eh_sum: f64,
                                  eh_zero: u64,
                                  eh_pos_offset: i32,
                                  eh_pos_buckets: Vec<u64>| {
            MetricsData::new(vec![ResourceMetrics::new(
                Resource::default(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::default(),
                    vec![
                        Metric::build()
                            .name("rpc_summary")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(ts)
                                    .count(s_count)
                                    .sum(s_sum)
                                    .quantile_values(vec![ValueAtQuantile::new(0.99, 0.42)])
                                    .finish(),
                            ]))
                            .finish(),
                        Metric::build()
                            .name("rpc_hist")
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    HistogramDataPoint::build()
                                        .time_unix_nano(ts)
                                        .count(h_count)
                                        .sum(h_sum)
                                        .explicit_bounds(vec![1.0, 5.0])
                                        .bucket_counts(h_buckets)
                                        .finish(),
                                ],
                            ))
                            .finish(),
                        Metric::build()
                            .name("rpc_exp_hist")
                            .data_exponential_histogram(ExponentialHistogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    ExponentialHistogramDataPoint::build()
                                        .time_unix_nano(ts)
                                        .scale(2)
                                        .count(eh_count)
                                        .sum(eh_sum)
                                        .zero_count(eh_zero)
                                        .positive(Buckets::new(eh_pos_offset, eh_pos_buckets))
                                        .negative(Buckets::new(0, vec![]))
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )])
        };

        // 1. Initial scrape: all 3 points dropped.
        let (out1, stats1) = run_batch(
            &mut proc,
            make_complex_batch(
                10_000_000,
                10,
                100.0,
                10,
                50.0,
                vec![2, 5, 3],
                10,
                50.0,
                2,
                1,
                vec![3, 5],
            ),
            t0,
        );
        assert!(out1.is_none());
        assert_eq!(stats1.dropped_initial, 3);

        // 2. Second scrape (with ExpHistogram positive bucket offset shifted from 1 -> 0):
        // At offset 0, bucket 0 has ref_idx = 0 + (0 - 1) = -1 (unadjusted = 4),
        // bucket 1 has ref_idx = 1 - 1 = 0 (subtract ref[0]=3 -> 7 - 3 = 4).
        let (out2, stats2) = run_batch(
            &mut proc,
            make_complex_batch(
                20_000_000,
                15,
                160.0,
                16,
                80.0,
                vec![4, 8, 4],
                18,
                90.0,
                3,
                0,
                vec![4, 7],
            ),
            t0,
        );
        assert_eq!(stats2.dropped_initial, 0);
        assert_eq!(stats2.adjusted_points, 3);
        assert_eq!(stats2.resets_detected, 0);

        let expected2 = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![
                    Metric::build()
                        .name("rpc_summary")
                        .data_summary(Summary::new(vec![
                            SummaryDataPoint::build()
                                .start_time_unix_nano(10_000_000u64)
                                .time_unix_nano(20_000_000u64)
                                .count(5u64)
                                .sum(60.0)
                                .quantile_values(vec![ValueAtQuantile::new(0.99, 0.42)])
                                .finish(),
                        ]))
                        .finish(),
                    Metric::build()
                        .name("rpc_hist")
                        .data_histogram(Histogram::new(
                            AggregationTemporality::Cumulative,
                            vec![
                                HistogramDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(20_000_000u64)
                                    .count(6u64)
                                    .sum(30.0)
                                    .explicit_bounds(vec![1.0, 5.0])
                                    .bucket_counts(vec![2, 3, 1])
                                    .finish(),
                            ],
                        ))
                        .finish(),
                    Metric::build()
                        .name("rpc_exp_hist")
                        .data_exponential_histogram(ExponentialHistogram::new(
                            AggregationTemporality::Cumulative,
                            vec![
                                ExponentialHistogramDataPoint::build()
                                    .start_time_unix_nano(10_000_000u64)
                                    .time_unix_nano(20_000_000u64)
                                    .scale(2)
                                    .count(8u64)
                                    .sum(40.0)
                                    .zero_count(1u64)
                                    .positive(Buckets::new(0, vec![4, 4]))
                                    .negative(Buckets::new(0, vec![]))
                                    .finish(),
                            ],
                        ))
                        .finish(),
                ],
            )],
        )]);
        assert_equivalent(
            &[OtlpProtoMessage::Metrics(out2.unwrap())],
            &[OtlpProtoMessage::Metrics(expected2)],
        );
    }

    #[test]
    fn test_mark_and_sweep_gc_evicts_stale_series_and_resources() {
        let gc_interval = Duration::from_secs(60);
        let mut proc = test_processor(gc_interval);
        let t0 = Instant::now();

        let make_resource_batch = |res_name: &str, ts: u64, val: i64| {
            MetricsData::new(vec![ResourceMetrics::new(
                Resource::build()
                    .attributes(vec![kv_str("service.name", res_name)])
                    .finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::default(),
                    vec![
                        Metric::build()
                            .name("requests_total")
                            .data_sum(Sum::new(
                                AggregationTemporality::Cumulative,
                                true,
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(ts)
                                        .value_int(val)
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )])
        };

        // At t0: see resource A and resource B (both dropped as initial, marks set to true).
        let (out_a1, _) = run_batch(&mut proc, make_resource_batch("svc-a", 10_000_000, 10), t0);
        let (out_b1, _) = run_batch(&mut proc, make_resource_batch("svc-b", 10_000_000, 20), t0);
        assert!(out_a1.is_none());
        assert!(out_b1.is_none());
        assert_eq!(proc.engine.cache.resource_count(), 2);

        // At t0 + 61s: first GC sweep clears marks from t0 (keeping svc-a and svc-b for one cycle).
        let t1 = t0 + Duration::from_secs(61);
        let (out_a2, _) = run_batch(&mut proc, make_resource_batch("svc-a", 20_000_000, 15), t1);
        assert!(out_a2.is_some());
        assert_eq!(proc.engine.cache.resource_count(), 2);

        // Touch svc-a again before next GC so svc-a is marked while svc-b remains unmarked.
        let t2 = t1 + Duration::from_secs(30);
        let (out_a3, _) = run_batch(&mut proc, make_resource_batch("svc-a", 30_000_000, 25), t2);
        assert!(out_a3.is_some());

        // At t1 + 61s: second GC sweep evicts unmarked svc-b while retaining svc-a.
        let t3 = t1 + Duration::from_secs(61);
        let (out_a4, _) = run_batch(&mut proc, make_resource_batch("svc-a", 40_000_000, 30), t3);
        assert!(out_a4.is_some());
        assert_eq!(proc.engine.cache.resource_count(), 1);

        // If svc-b reappears now, its first point is treated as a fresh initial point and dropped.
        let (out_b_fresh, stats_b_fresh) =
            run_batch(&mut proc, make_resource_batch("svc-b", 50_000_000, 100), t3);
        assert!(out_b_fresh.is_none());
        assert_eq!(stats_b_fresh.dropped_initial, 1);
    }

    #[test]
    fn test_pipeline_runtime_drops_initial_and_emits_subsequent_batch() {
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let controller = ControllerContext::new(test_runtime.metrics_registry());
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let user_cfg = Arc::new(NodeUserConfig::with_user_config(
            METRIC_START_TIME_PROCESSOR_URN.into(),
            json!({
                "strategy": "subtract_initial_point",
                "gc_interval": "10m"
            }),
        ));
        let processor = create_metric_start_time_processor(
            pipeline_ctx,
            test_node(test_runtime.config().name.clone()),
            user_cfg,
            test_runtime.config(),
        )
        .unwrap();

        test_runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let make_pdata = |ts: u64, val: i64| {
                    let md = MetricsData::new(vec![ResourceMetrics::new(
                        Resource::default(),
                        vec![ScopeMetrics::new(
                            InstrumentationScope::default(),
                            vec![
                                Metric::build()
                                    .name("counter_total")
                                    .data_sum(Sum::new(
                                        AggregationTemporality::Cumulative,
                                        true,
                                        vec![
                                            NumberDataPoint::build()
                                                .time_unix_nano(ts)
                                                .value_int(val)
                                                .finish(),
                                        ],
                                    ))
                                    .finish(),
                            ],
                        )],
                    )]);
                    OtapPdata::new_default(otlp_to_otap(&OtlpProtoMessage::Metrics(md)).into())
                };

                // Initial batch: all points dropped, so no output batch is emitted downstream.
                ctx.process(Message::PData(make_pdata(10_000_000, 40)))
                    .await
                    .unwrap();
                let emitted1 = ctx.drain_pdata().await;
                assert!(emitted1.is_empty());

                // Second batch: emitted with start_time = 10_000_000 and value = 55 - 40 = 15.
                ctx.process(Message::PData(make_pdata(20_000_000, 55)))
                    .await
                    .unwrap();
                let emitted2 = ctx.drain_pdata().await;
                assert_eq!(emitted2.len(), 1);
            })
            .validate(|_ctx| async move {});
    }
}
