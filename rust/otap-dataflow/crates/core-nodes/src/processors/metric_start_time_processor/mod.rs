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
    Array, ArrayRef, ArrowPrimitiveType, BooleanArray, BooleanBufferBuilder, BooleanBuilder,
    Int32Array, ListArray, NullBufferBuilder, PrimitiveArray, RecordBatch, StringArray,
    StructArray, UInt16Array, UInt64Array,
};
use arrow::buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::compute::filter_record_batch;
use arrow::datatypes::{
    ArrowNativeType, DataType, Field, FieldRef, Float64Type, Int32Type, Int64Type, Schema,
    SchemaRef, TimestampNanosecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
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
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords, OtapBatchStore};
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, DataPointFlags,
};
use otel_arrow_dfe_pdata::schema::consts;
use otel_arrow_dfe_telemetry::common_attributes::SignalRegistrationAttributes;
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use serde_json::Value;

mod config;
mod metrics;
mod state;

use self::config::Config;
use self::metrics::MetricStartTimeMetrics;
use self::state::{
    AttrHash, AttrHasher, CountSumState, ExpHistogramState, HistogramState, NumberState,
    ResourceSeriesMap, SeriesEntry, SeriesTable, SubtractInitialCache,
};

/// The URN for the metric start time processor.
pub const METRIC_START_TIME_PROCESSOR_URN: &str = "urn:otel:processor:metric_start_time";

const METRIC_TYPE_SUM: u8 = MetricType::Sum as u8;
const METRIC_TYPE_HISTOGRAM: u8 = MetricType::Histogram as u8;
const METRIC_TYPE_EXP_HISTOGRAM: u8 = MetricType::ExponentialHistogram as u8;
const METRIC_TYPE_SUMMARY: u8 = MetricType::Summary as u8;

const TEMPORALITY_CUMULATIVE: i32 = AggregationTemporality::Cumulative as i32;
const FLAG_NO_RECORDED_VALUE: u32 = DataPointFlags::NoRecordedValueMask as u32;
const RESET_START_TIME_OFFSET_NS: i64 = 1_000_000;
const MAX_DENSE_METRIC_ID: usize = 65_536;

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

#[derive(Debug, Clone, Copy)]
struct EligibleMetricSlot {
    resource_hash: AttrHash,
    metric_row: usize,
    metric_type: u8,
}

#[derive(Debug, Default)]
struct BatchMetricLookup {
    slots: Vec<Option<EligibleMetricSlot>>,
    populated_ids: Vec<u16>,
    eligible_row_flags: Vec<bool>,
    has_table: [bool; 4],
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
        self.has_table = [false; 4];
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

/// Stack-allocated replacement column list (at most 5 columns for `ExpHistogramDataPoints`),
/// avoiding a per-table `Vec` heap allocation on every batch.
#[derive(Default)]
struct ReplList {
    items: [Option<(&'static str, ArrayRef)>; 5],
    len: usize,
}

impl ReplList {
    #[inline]
    fn push(&mut self, name: &'static str, col: ArrayRef) {
        self.items[self.len] = Some((name, col));
        self.len += 1;
    }

    #[inline]
    fn find(&self, name: &str) -> Option<&ArrayRef> {
        self.items[..self.len]
            .iter()
            .flatten()
            .find_map(|(n, col)| (*n == name).then_some(col))
    }
}

/// Caches the output [`SchemaRef`] per data point table so steady-state batches reuse the schema
/// via `Arc::clone` with zero `Field`, `Schema`, or metadata map allocations.
#[derive(Default)]
struct OutputSchemaCache {
    entries: [Option<(SchemaRef, SchemaRef)>; 4],
}

fn filter_or_clone(
    col: &ArrayRef,
    keep_filter: Option<&BooleanArray>,
) -> Result<ArrayRef, PdataError> {
    match keep_filter {
        Some(filter) => arrow::compute::filter(col, filter)
            .map_err(|source| PdataError::ColumnLengthMismatch { source }),
        None => Ok(Arc::clone(col)),
    }
}

fn start_time_insert_idx(fields: &[FieldRef]) -> usize {
    fields
        .iter()
        .position(|f| f.name().as_str() == consts::PARENT_ID)
        .map_or(0, |idx| idx + 1)
}

impl OutputSchemaCache {
    fn rebuild_batch(
        &mut self,
        table_idx: usize,
        rb: &RecordBatch,
        keep_filter: Option<&BooleanArray>,
        start_time_col: ArrayRef,
        replacements: &ReplList,
    ) -> Result<RecordBatch, PdataError> {
        let old_schema = rb.schema_ref();
        let out_schema = self.get_or_build(table_idx, old_schema, start_time_col.data_type());
        let mut columns = Vec::with_capacity(out_schema.fields().len());
        let mut has_start_time = false;

        for (col_idx, field) in old_schema.fields().iter().enumerate() {
            let name = field.name().as_str();
            if name == consts::START_TIME_UNIX_NANO {
                has_start_time = true;
                columns.push(Arc::clone(&start_time_col));
            } else if let Some(rep_col) = replacements.find(name) {
                columns.push(Arc::clone(rep_col));
            } else {
                columns.push(filter_or_clone(rb.column(col_idx), keep_filter)?);
            }
        }

        if !has_start_time {
            columns.insert(start_time_insert_idx(old_schema.fields()), start_time_col);
        }

        RecordBatch::try_new(out_schema, columns)
            .map_err(|source| PdataError::ColumnLengthMismatch { source })
    }

    fn get_or_build(
        &mut self,
        table_idx: usize,
        old_schema: &SchemaRef,
        start_dt: &DataType,
    ) -> SchemaRef {
        if let Some((cached_in, cached_out)) = &self.entries[table_idx]
            && Arc::ptr_eq(cached_in, old_schema)
        {
            return Arc::clone(cached_out);
        }

        let old_fields = old_schema.fields();
        let start_pos = old_fields
            .iter()
            .position(|f| f.name().as_str() == consts::START_TIME_UNIX_NANO);
        let start_ok = start_pos.is_some_and(|idx| {
            old_fields[idx].is_nullable() && old_fields[idx].data_type() == start_dt
        });
        let needs_null_upgrade = old_fields
            .iter()
            .any(|f| !f.is_nullable() && f.name().as_str() == consts::SUM);
        let out_schema = if start_ok && !needs_null_upgrade {
            Arc::clone(old_schema)
        } else {
            let mut fields = old_fields.to_vec();
            for field in &mut fields {
                if !field.is_nullable() && field.name().as_str() == consts::SUM {
                    *field = Arc::new(field.as_ref().clone().with_nullable(true));
                }
            }
            match start_pos {
                Some(idx) if !start_ok => {
                    fields[idx] = Arc::new(
                        fields[idx]
                            .as_ref()
                            .clone()
                            .with_data_type(start_dt.clone())
                            .with_nullable(true),
                    );
                }
                Some(_) => {}
                None => fields.insert(
                    start_time_insert_idx(&fields),
                    Arc::new(Field::new(
                        consts::START_TIME_UNIX_NANO,
                        start_dt.clone(),
                        true,
                    )),
                ),
            }
            Arc::new(Schema::new_with_metadata(
                fields,
                old_schema.metadata().clone(),
            ))
        };

        self.entries[table_idx] = Some((Arc::clone(old_schema), Arc::clone(&out_schema)));
        out_schema
    }
}

struct SubtractInitialEngine {
    cache: SubtractInitialCache,
    res_attr_hasher: AttrHasher,
    dp_attr_hasher: AttrHasher,
    metric_lookup: BatchMetricLookup,
    id_bitmap_pool: IdBitmapPool,
    schema_cache: OutputSchemaCache,
    utc_tz: Arc<str>,
}

/// Processor that adjusts `start_time_unix_nano` on cumulative metrics using the
/// `subtract_initial_point` strategy.
pub struct MetricStartTimeProcessor {
    engine: SubtractInitialEngine,
    metrics: MetricSet<MetricStartTimeMetrics>,
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
    fn new(config: Config, pipeline_ctx: PipelineContext) -> Result<Self, ConfigError> {
        config.validate()?;
        let config::Strategy::SubtractInitialPoint = config.strategy;
        let metrics = MetricStartTimeMetrics::register(
            &pipeline_ctx,
            &SignalRegistrationAttributes {
                signal: SignalType::Metrics,
            },
        );
        let compute_duration = ComputeDuration::new(&pipeline_ctx);
        Ok(Self {
            engine: SubtractInitialEngine {
                cache: SubtractInitialCache::new(config.gc_interval, Instant::now()),
                res_attr_hasher: AttrHasher::new(),
                dp_attr_hasher: AttrHasher::new(),
                metric_lookup: BatchMetricLookup::default(),
                id_bitmap_pool: IdBitmapPool::new(),
                schema_cache: OutputSchemaCache::default(),
                utc_tz: Arc::from("UTC"),
            },
            metrics,
            compute_duration,
        })
    }

    /// Creates a new [`MetricStartTimeProcessor`] from a JSON configuration value.
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config = otel_arrow_dfe_config::validation::deserialize_typed_config(config)?;
        Self::new(config, pipeline_ctx)
    }
}

fn is_plain_field(f: &Field) -> bool {
    f.metadata()
        .get(consts::metadata::COLUMN_ENCODING)
        .is_some_and(|enc| enc == consts::metadata::encodings::PLAIN)
}

fn all_ids_already_plain(records: &OtapArrowRecords) -> bool {
    let col_plain =
        |s: &Schema, n: &str| s.column_with_name(n).is_none_or(|(_, f)| is_plain_field(f));
    let struct_id_plain = |s: &Schema, n: &str| {
        let Some((_, col)) = s.column_with_name(n) else {
            return true;
        };
        match col.data_type() {
            DataType::Struct(fields) => fields
                .find(consts::ID)
                .is_none_or(|(_, f)| is_plain_field(f)),
            _ => true,
        }
    };
    Metrics::allowed_payload_types().iter().all(|&pt| {
        records.get(pt).is_none_or(|rb| {
            let s = rb.schema_ref();
            col_plain(s, consts::ID)
                && col_plain(s, consts::PARENT_ID)
                && struct_id_plain(s, consts::RESOURCE)
                && struct_id_plain(s, consts::SCOPE)
        })
    })
}

trait DpAdjuster {
    const TABLE_IDX: usize;
    const METRIC_TYPE: u8;
    const DP_PAYLOAD: ArrowPayloadType;
    const ATTRS_PAYLOAD: ArrowPayloadType;
    const EXEMPLARS_PAYLOAD: Option<(ArrowPayloadType, ArrowPayloadType)>;

    type State;
    type Table<'a>;

    fn try_table(rb: &RecordBatch) -> Result<Self::Table<'_>, PdataError>;
    fn series_table(res: &mut ResourceSeriesMap) -> &mut SeriesTable<Self::State>;
    fn init_outputs(t: &mut Self::Table<'_>, num_rows: usize, prefix_rows: usize);
    fn append_passthrough(t: &mut Self::Table<'_>, row: usize);
    fn seed_initial(t: &Self::Table<'_>, row: usize) -> Self::State;
    fn adjust_occupied(t: &mut Self::Table<'_>, row: usize, state: &mut Self::State) -> bool;
    fn finish_replacements(
        t: Self::Table<'_>,
        keep_filter: Option<&BooleanArray>,
        out: &mut ReplList,
    ) -> Result<(), PdataError>;
}

enum TableMutation {
    Unchanged,
    AllDropped,
    Rebuilt {
        batch: RecordBatch,
        filtered_rows: bool,
    },
}

impl SubtractInitialEngine {
    fn process_metrics_batch(
        &mut self,
        records: &mut OtapArrowRecords,
        now: Instant,
    ) -> Result<BatchAdjustStats, PdataError> {
        self.cache.maybe_clear_stale(now);
        if !self.scan_and_index_metrics(records)? {
            self.cache.maybe_gc(now);
            return Ok(BatchAdjustStats::default());
        }

        let mut stats = BatchAdjustStats::default();
        stats.merge(self.adjust_dp_table::<NumberAdjuster>(records)?);
        stats.merge(self.adjust_dp_table::<SummaryAdjuster>(records)?);
        stats.merge(self.adjust_dp_table::<HistogramAdjuster>(records)?);
        stats.merge(self.adjust_dp_table::<ExpHistogramAdjuster>(records)?);

        if stats.dropped_initial > 0 {
            self.prune_empty_metrics(records)?;
        }

        self.cache.maybe_gc(now);
        Ok(stats)
    }

    /// Scans `UnivariateMetrics` for eligible cumulative metrics before touching ID encodings;
    /// returns `Ok(false)` immediately if none are present.
    fn scan_and_index_metrics(
        &mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<bool, PdataError> {
        self.metric_lookup.clear();
        let num_rows = match records.get(ArrowPayloadType::UnivariateMetrics) {
            Some(rb) if rb.num_rows() > 0 => {
                let m_types = PrimCol::from_required::<UInt8Type>(rb, consts::METRIC_TYPE)?;
                let temp = rb
                    .column_by_name(consts::AGGREGATION_TEMPORALITY)
                    .map(MaybeDictArrayAccessor::<Int32Array>::try_new)
                    .transpose()?;
                let is_monotonic = rb
                    .column_by_name(consts::IS_MONOTONIC)
                    .and_then(|c| c.as_any().downcast_ref::<BooleanArray>());
                let is_cum = |row| {
                    temp.as_ref()
                        .and_then(|t| t.value_at(row))
                        .is_some_and(|t| t == TEMPORALITY_CUMULATIVE)
                };

                self.metric_lookup.eligible_row_flags.reserve(rb.num_rows());
                let mut any = false;
                for row in 0..rb.num_rows() {
                    let ok = match m_types.get(row) {
                        Some(METRIC_TYPE_SUMMARY) => true,
                        // Skip non-monotonic cumulative sums (UpDownCounter), whose absolute
                        // values legitimately decrease without representing a counter reset.
                        Some(METRIC_TYPE_SUM) => {
                            is_cum(row)
                                && is_monotonic.is_some_and(|m| m.is_valid(row) && m.value(row))
                        }
                        Some(METRIC_TYPE_HISTOGRAM | METRIC_TYPE_EXP_HISTOGRAM) => is_cum(row),
                        _ => false,
                    };
                    self.metric_lookup.eligible_row_flags.push(ok);
                    any |= ok;
                }
                if !any {
                    return Ok(false);
                }
                rb.num_rows()
            }
            _ => return Ok(false),
        };

        if !all_ids_already_plain(records) {
            records.decode_transport_optimized_ids()?;
        }

        let metrics_rb = records
            .get(ArrowPayloadType::UnivariateMetrics)
            .expect("checked above");
        let ids = PrimCol::from_required::<UInt16Type>(metrics_rb, consts::ID)?;
        let m_types = PrimCol::from_required::<UInt8Type>(metrics_rb, consts::METRIC_TYPE)?;
        let res_struct = metrics_rb
            .column_by_name(consts::RESOURCE)
            .and_then(|c| c.as_any().downcast_ref::<StructArray>());
        let res_ids = res_struct
            .and_then(|s| s.column_by_name(consts::ID))
            .and_then(|c| c.as_any().downcast_ref::<UInt16Array>());

        self.res_attr_hasher
            .populate::<UInt16Type>(records.get(ArrowPayloadType::ResourceAttrs))?;

        for row in 0..num_rows {
            if !self.metric_lookup.eligible_row_flags[row] {
                continue;
            }
            let Some(metric_id) = ids.get(row) else {
                self.metric_lookup.eligible_row_flags[row] = false;
                continue;
            };
            let m_type = m_types.values[row];
            let resource_hash = if res_struct.is_none_or(|s| s.is_valid(row))
                && let Some(r_ids) = res_ids
                && r_ids.is_valid(row)
            {
                self.res_attr_hasher.get(u32::from(r_ids.value(row)))
            } else {
                [0, 0]
            };

            let idx = metric_id as usize;
            if idx < MAX_DENSE_METRIC_ID {
                if idx >= self.metric_lookup.slots.len() {
                    let next_len = (idx + 1).next_power_of_two().min(MAX_DENSE_METRIC_ID);
                    self.metric_lookup.slots.resize(next_len, None);
                }
                self.metric_lookup.slots[idx] = Some(EligibleMetricSlot {
                    resource_hash,
                    metric_row: row,
                    metric_type: m_type,
                });
                self.metric_lookup.populated_ids.push(metric_id);
            }

            let table_idx = match m_type {
                METRIC_TYPE_SUM => NumberAdjuster::TABLE_IDX,
                METRIC_TYPE_SUMMARY => SummaryAdjuster::TABLE_IDX,
                METRIC_TYPE_HISTOGRAM => HistogramAdjuster::TABLE_IDX,
                METRIC_TYPE_EXP_HISTOGRAM => ExpHistogramAdjuster::TABLE_IDX,
                _ => continue,
            };
            self.metric_lookup.has_table[table_idx] = true;
        }
        Ok(true)
    }

    fn adjust_dp_table<A: DpAdjuster>(
        &mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<BatchAdjustStats, PdataError> {
        let mut stats = BatchAdjustStats::default();
        if !self.metric_lookup.has_table[A::TABLE_IDX] {
            return Ok(stats);
        }

        let mutation = {
            let Some(dp_rb) = records.get(A::DP_PAYLOAD) else {
                return Ok(stats);
            };
            let num_rows = dp_rb.num_rows();
            if num_rows == 0 {
                return Ok(stats);
            }

            let metrics_rb = records
                .get(ArrowPayloadType::UnivariateMetrics)
                .expect("checked by caller");
            let metric_names = MaybeDictArrayAccessor::<StringArray>::try_new(get_required_array(
                metrics_rb,
                consts::NAME,
            )?)?;

            let common = CommonDpColumns::try_new(dp_rb)?;
            let mut table = A::try_table(dp_rb)?;

            struct ActiveState {
                start_times: StartTimeBuilder,
                keep_flags: Option<BooleanBufferBuilder>,
                kept_count: usize,
            }

            let mut active: Option<ActiveState> = None;
            let mut active_parent_id: Option<Option<u16>> = None;
            let mut active_map: Option<&mut HashMap<AttrHash, SeriesEntry<A::State>>> = None;

            for row in 0..num_rows {
                let orig_start_opt = common.start_times.and_then(|c| c.get(row));
                let time_ns = common.times.and_then(|c| c.get(row)).unwrap_or(0);
                let orig_start_ns = orig_start_opt.unwrap_or(0);

                let is_preset = orig_start_ns != 0 && orig_start_ns != time_ns;
                if !is_preset {
                    let parent_id_opt = common.parent_ids.get(row);
                    if active_parent_id != Some(parent_id_opt) {
                        active_parent_id = Some(parent_id_opt);
                        active_map = parent_id_opt
                            .and_then(|pid| self.metric_lookup.get(pid, A::METRIC_TYPE))
                            .map(|slot| {
                                let name = metric_names.str_at(slot.metric_row).unwrap_or("");
                                A::series_table(self.cache.resource_mut(slot.resource_hash))
                                    .series_mut(name)
                            });
                    }
                }

                if is_preset || active_map.is_none() {
                    if let Some(act) = active.as_mut() {
                        if let Some(kf) = act.keep_flags.as_mut() {
                            kf.append(true);
                        }
                        act.kept_count += 1;
                        act.start_times.push_opt(orig_start_opt);
                        A::append_passthrough(&mut table, row);
                    }
                    continue;
                }

                if active.is_none() {
                    self.dp_attr_hasher
                        .populate::<UInt32Type>(records.get(A::ATTRS_PAYLOAD))?;
                    let mut st = StartTimeBuilder::with_capacity(num_rows);
                    for prev in 0..row {
                        st.push_opt(common.start_times.and_then(|c| c.get(prev)));
                    }
                    A::init_outputs(&mut table, num_rows, row);
                    active = Some(ActiveState {
                        start_times: st,
                        keep_flags: None,
                        kept_count: row,
                    });
                }

                let act = active.as_mut().expect("initialized above");
                let series_map = active_map.as_deref_mut().expect("checked above");
                let dp_attr_hash = common
                    .ids
                    .and_then(|c| c.get(row))
                    .map_or([0, 0], |id| self.dp_attr_hasher.get(id));

                match series_map.entry(dp_attr_hash) {
                    Entry::Vacant(vacant) => {
                        let _ =
                            vacant.insert(SeriesEntry::new(time_ns, A::seed_initial(&table, row)));
                        let kf = act.keep_flags.get_or_insert_with(|| {
                            let mut kb = BooleanBufferBuilder::new(num_rows);
                            kb.append_n(row, true);
                            kb
                        });
                        kf.append(false);
                        stats.dropped_initial += 1;
                    }
                    Entry::Occupied(mut occupied) => {
                        let entry = occupied.get_mut();
                        entry.mark();
                        if let Some(kf) = act.keep_flags.as_mut() {
                            kf.append(true);
                        }
                        act.kept_count += 1;
                        stats.adjusted_points += 1;

                        if common.is_no_recorded_value(row) {
                            A::append_passthrough(&mut table, row);
                        } else if A::adjust_occupied(&mut table, row, &mut entry.data) {
                            entry.set_start_time_ns(
                                time_ns.saturating_sub(RESET_START_TIME_OFFSET_NS).max(0),
                            );
                            stats.resets_detected += 1;
                        }
                        act.start_times.push_val(entry.start_time_ns());
                    }
                }
            }

            match active {
                None => TableMutation::Unchanged,
                Some(act) if act.kept_count == 0 => TableMutation::AllDropped,
                Some(act) => {
                    let keep_filter = act.keep_flags.map(|mut kf| BooleanArray::from(kf.finish()));
                    let start_col: ArrayRef = Arc::new(
                        act.start_times
                            .finish()
                            .with_timezone_opt(Some(Arc::clone(&self.utc_tz))),
                    );
                    let mut replacements = ReplList::default();
                    A::finish_replacements(table, keep_filter.as_ref(), &mut replacements)?;
                    let batch = self.schema_cache.rebuild_batch(
                        A::TABLE_IDX,
                        dp_rb,
                        keep_filter.as_ref(),
                        start_col,
                        &replacements,
                    )?;
                    TableMutation::Rebuilt {
                        batch,
                        filtered_rows: keep_filter.is_some(),
                    }
                }
            }
        };

        match mutation {
            TableMutation::Unchanged => {}
            TableMutation::AllDropped => {
                let _ = records.remove(A::DP_PAYLOAD);
                let _ = records.remove(A::ATTRS_PAYLOAD);
                if let Some((ex_pt, ex_attrs_pt)) = A::EXEMPLARS_PAYLOAD {
                    let _ = records.remove(ex_pt);
                    let _ = records.remove(ex_attrs_pt);
                }
            }
            TableMutation::Rebuilt {
                batch,
                filtered_rows,
            } => {
                records.set(A::DP_PAYLOAD, batch)?;
                if filtered_rows {
                    self.filter_dp_children(
                        records,
                        A::DP_PAYLOAD,
                        A::ATTRS_PAYLOAD,
                        A::EXEMPLARS_PAYLOAD,
                    )?;
                }
            }
        }

        Ok(stats)
    }

    fn filter_dp_children(
        &mut self,
        records: &mut OtapArrowRecords,
        parent_pt: ArrowPayloadType,
        attrs_pt: ArrowPayloadType,
        exemplars_pts: Option<(ArrowPayloadType, ArrowPayloadType)>,
    ) -> Result<(), PdataError> {
        let has_attrs = records.get(attrs_pt).is_some();
        let has_ex = exemplars_pts.is_some_and(|(ex_pt, _)| records.get(ex_pt).is_some());
        if !has_attrs && !has_ex {
            if let Some((_, ex_attrs_pt)) = exemplars_pts {
                let _ = records.remove(ex_attrs_pt);
            }
            return Ok(());
        }

        let mut id_bitmap = self.id_bitmap_pool.acquire();
        let res = (|| -> Result<(), PdataError> {
            let Some(parent_rb) = records.get(parent_pt) else {
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
                && let Some(p_col) =
                    PrimCol::<u16>::from_opt::<UInt16Type>(dp_rb, consts::PARENT_ID)
            {
                for row in 0..dp_rb.num_rows() {
                    if let Some(pid) = p_col.get(row) {
                        surviving_ids.insert(u32::from(pid));
                    }
                }
            }
        }

        let metric_ids = PrimCol::from_required::<UInt16Type>(metrics_rb, consts::ID)?;
        let mut selection_builder = BooleanBuilder::with_capacity(num_metrics);
        let mut kept_metrics = 0usize;
        for row in 0..num_metrics {
            let keep = !self.metric_lookup.is_eligible_row(row)
                || metric_ids
                    .get(row)
                    .is_some_and(|id| surviving_ids.contains(u32::from(id)));
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

fn filter_nulls(
    orig: Option<&NullBuffer>,
    keep_filter: Option<&BooleanArray>,
) -> Option<NullBuffer> {
    let orig = orig?;
    match keep_filter {
        None => Some(orig.clone()),
        Some(filter) => {
            let mut nb = NullBufferBuilder::new(filter.true_count());
            for row in 0..filter.len() {
                if filter.value(row) {
                    nb.append(orig.is_valid(row));
                }
            }
            nb.finish()
        }
    }
}

#[derive(Clone, Copy)]
struct PrimCol<'a, T> {
    values: &'a [T],
    nulls: Option<&'a NullBuffer>,
}

impl<'a, T: ArrowNativeType> PrimCol<'a, T> {
    #[inline]
    fn from_array<A: ArrowPrimitiveType<Native = T>>(col: &'a ArrayRef) -> Option<Self> {
        let arr = col.as_any().downcast_ref::<PrimitiveArray<A>>()?;
        Some(Self {
            values: arr.values().as_ref(),
            nulls: arr.nulls(),
        })
    }

    #[inline]
    fn from_opt<A: ArrowPrimitiveType<Native = T>>(
        rb: &'a RecordBatch,
        name: &str,
    ) -> Option<Self> {
        Self::from_array::<A>(rb.column_by_name(name)?)
    }

    #[inline]
    fn from_required<A: ArrowPrimitiveType<Native = T>>(
        rb: &'a RecordBatch,
        name: &'static str,
    ) -> Result<Self, PdataError> {
        let col = get_required_array(rb, name)?;
        Self::from_array::<A>(col).ok_or_else(|| PdataError::ColumnDataTypeMismatch {
            name: name.into(),
            actual: col.data_type().clone(),
            expect: A::DATA_TYPE,
        })
    }

    #[inline]
    fn get(&self, row: usize) -> Option<T> {
        if self.nulls.is_some_and(|n| n.is_null(row)) {
            None
        } else {
            Some(self.values[row])
        }
    }

    #[inline]
    fn val_or_default(&self, row: usize) -> T {
        self.get(row).unwrap_or_default()
    }
}

/// Builder for `start_time_unix_nano` (where nullability can differ from the input column).
struct StartTimeBuilder {
    values: Vec<i64>,
    nulls: Option<NullBufferBuilder>,
}

impl StartTimeBuilder {
    #[inline]
    fn with_capacity(cap: usize) -> Self {
        Self {
            values: Vec::with_capacity(cap),
            nulls: None,
        }
    }

    #[inline]
    fn push_val(&mut self, val: i64) {
        if let Some(nb) = self.nulls.as_mut() {
            nb.append_non_null();
        }
        self.values.push(val);
    }

    #[inline]
    fn push_opt(&mut self, opt: Option<i64>) {
        match opt {
            Some(v) => self.push_val(v),
            None => {
                let cur_len = self.values.len();
                let cap = self.values.capacity();
                let nb = self.nulls.get_or_insert_with(|| {
                    let mut b = NullBufferBuilder::new(cap);
                    b.append_n_non_nulls(cur_len);
                    b
                });
                nb.append_null();
                self.values.push(0);
            }
        }
    }

    #[inline]
    fn finish(self) -> PrimitiveArray<TimestampNanosecondType> {
        let nulls = self.nulls.and_then(|mut nb| nb.finish());
        PrimitiveArray::new(ScalarBuffer::from(self.values), nulls)
    }
}

/// Combined optional primitive column reader and lazy output buffer.
///
/// Allocates nothing until `init_output` is called on the first adjusted row, bulk-copies any
/// leading passthrough slice, and reuses `orig.nulls().cloned()` in steady state.
struct OptValCol<'a, A: ArrowPrimitiveType> {
    name: &'static str,
    src: Option<PrimCol<'a, A::Native>>,
    out: Vec<A::Native>,
    clear_mask: Option<NullBufferBuilder>,
}

impl<'a, A: ArrowPrimitiveType> OptValCol<'a, A> {
    #[inline]
    fn from_rb(rb: &'a RecordBatch, name: &'static str) -> Self {
        Self {
            name,
            src: PrimCol::from_opt::<A>(rb, name),
            out: Vec::new(),
            clear_mask: None,
        }
    }

    #[inline]
    fn init(&mut self, num_rows: usize, prefix_rows: usize) {
        if let Some(c) = self.src {
            self.out.reserve_exact(num_rows);
            self.out.extend_from_slice(&c.values[..prefix_rows]);
        }
    }

    #[inline]
    fn read(&self, row: usize) -> A::Native {
        self.src
            .map_or_else(Default::default, |c| c.val_or_default(row))
    }

    #[inline]
    fn get_opt(&self, row: usize) -> Option<A::Native> {
        self.src.and_then(|c| c.get(row))
    }

    #[inline]
    fn push_passthrough(&mut self, row: usize) {
        if let Some(c) = self.src {
            self.out.push(c.values[row]);
            if let Some(m) = self.clear_mask.as_mut() {
                m.append_non_null();
            }
        }
    }

    #[inline]
    fn push(&mut self, val: A::Native) {
        if self.src.is_some() {
            self.out.push(val);
            if let Some(m) = self.clear_mask.as_mut() {
                m.append_non_null();
            }
        }
    }

    #[inline]
    fn push_null(&mut self) {
        if self.src.is_some() {
            let cur_len = self.out.len();
            let cap = self.out.capacity();
            let m = self.clear_mask.get_or_insert_with(|| {
                let mut b = NullBufferBuilder::new(cap);
                b.append_n_non_nulls(cur_len);
                b
            });
            m.append_null();
            self.out.push(A::Native::default());
        }
    }

    #[inline]
    fn finish(self, keep_filter: Option<&BooleanArray>, repl: &mut ReplList) {
        if let Some(c) = self.src {
            let orig_nulls = filter_nulls(c.nulls, keep_filter);
            let cleared_nulls = self.clear_mask.and_then(|mut m| m.finish());
            let nulls = NullBuffer::union(orig_nulls.as_ref(), cleared_nulls.as_ref());
            let arr = PrimitiveArray::<A>::new(ScalarBuffer::from(self.out), nulls);
            repl.push(self.name, Arc::new(arr));
        }
    }
}

struct CommonDpColumns<'a> {
    parent_ids: PrimCol<'a, u16>,
    ids: Option<PrimCol<'a, u32>>,
    start_times: Option<PrimCol<'a, i64>>,
    times: Option<PrimCol<'a, i64>>,
    flags: Option<PrimCol<'a, u32>>,
}

impl<'a> CommonDpColumns<'a> {
    fn try_new(rb: &'a RecordBatch) -> Result<Self, PdataError> {
        Ok(Self {
            parent_ids: PrimCol::from_required::<UInt16Type>(rb, consts::PARENT_ID)?,
            ids: PrimCol::from_opt::<UInt32Type>(rb, consts::ID),
            start_times: PrimCol::from_opt::<TimestampNanosecondType>(
                rb,
                consts::START_TIME_UNIX_NANO,
            ),
            times: PrimCol::from_opt::<TimestampNanosecondType>(rb, consts::TIME_UNIX_NANO),
            flags: PrimCol::from_opt::<UInt32Type>(rb, consts::FLAGS),
        })
    }

    #[inline]
    fn is_no_recorded_value(&self, row: usize) -> bool {
        self.flags
            .and_then(|c| c.get(row))
            .is_some_and(|f| (f & FLAG_NO_RECORDED_VALUE) != 0)
    }
}

#[derive(Clone, Copy)]
struct ListSliceAccessor<'a, T: ArrowPrimitiveType> {
    list: &'a ListArray,
    item_field: &'a FieldRef,
    values: &'a [T::Native],
}

impl<'a, T: ArrowPrimitiveType> ListSliceAccessor<'a, T> {
    fn try_new(col: &'a ArrayRef) -> Result<Self, PdataError> {
        let err = |actual: &DataType| PdataError::InvalidListArray {
            expect_oneof: vec![T::DATA_TYPE],
            actual: actual.clone(),
        };
        let list = col
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| err(col.data_type()))?;
        let DataType::List(item_field) = list.data_type() else {
            return Err(err(col.data_type()));
        };
        let prim = list
            .values()
            .as_any()
            .downcast_ref::<PrimitiveArray<T>>()
            .ok_or_else(|| err(list.values().data_type()))?;
        Ok(Self {
            list,
            item_field,
            values: prim.values().as_ref(),
        })
    }

    #[inline]
    fn slice_at(&self, row: usize) -> Option<&'a [T::Native]> {
        if self.list.nulls().is_some_and(|n| n.is_null(row)) {
            return None;
        }
        let offsets = self.list.value_offsets();
        Some(&self.values[offsets[row] as usize..offsets[row + 1] as usize])
    }
}

/// Combined optional `List<UInt64>` column reader and lazy pre-sized output builder.
struct OptU64ListCol<'a> {
    src: Option<ListSliceAccessor<'a, UInt64Type>>,
    offsets: Vec<i32>,
    values: Vec<u64>,
}

impl<'a> OptU64ListCol<'a> {
    fn try_from_opt(col: Option<&'a ArrayRef>) -> Result<Self, PdataError> {
        Ok(Self {
            src: col
                .map(ListSliceAccessor::<UInt64Type>::try_new)
                .transpose()?,
            offsets: Vec::new(),
            values: Vec::new(),
        })
    }

    fn init(&mut self, num_rows: usize, prefix_rows: usize) {
        let Some(acc) = self.src.as_ref() else {
            return;
        };
        let in_off = acc.list.value_offsets();
        let base = in_off[0];
        let total_vals = (in_off[num_rows] - base).max(0) as usize;
        self.offsets.reserve_exact(num_rows + 1);
        for &o in &in_off[..=prefix_rows] {
            self.offsets.push(o - base);
        }
        self.values.reserve_exact(total_vals);
        self.values
            .extend_from_slice(&acc.values[base as usize..in_off[prefix_rows] as usize]);
    }

    #[inline]
    fn slice_at(&self, row: usize) -> Option<&'a [u64]> {
        self.src.as_ref().and_then(|a| a.slice_at(row))
    }

    #[inline]
    fn push_passthrough(&mut self, row: usize) {
        self.push_raw(self.slice_at(row));
    }

    #[inline]
    fn push_raw(&mut self, opt_slice: Option<&[u64]>) {
        if self.src.is_some() {
            if let Some(s) = opt_slice {
                self.values.extend_from_slice(s);
            }
            self.offsets.push(self.values.len() as i32);
        }
    }

    #[inline]
    fn push_subtracted(&mut self, opt_slice: Option<&[u64]>, ref_buckets: &[u64]) {
        if self.src.is_some() {
            if let Some(s) = opt_slice {
                for (&cur_b, &ref_b) in s.iter().zip(ref_buckets.iter()) {
                    self.values.push(cur_b.saturating_sub(ref_b));
                }
            }
            self.offsets.push(self.values.len() as i32);
        }
    }

    fn finish_list(self, keep_filter: Option<&BooleanArray>) -> Option<ArrayRef> {
        let acc = self.src?;
        let values_arr = Arc::new(UInt64Array::new(ScalarBuffer::from(self.values), None));
        Some(Arc::new(ListArray::new(
            Arc::clone(acc.item_field),
            OffsetBuffer::new(ScalarBuffer::from(self.offsets)),
            values_arr,
            filter_nulls(acc.list.nulls(), keep_filter),
        )))
    }
}

/// Shared `count` + `sum` column pair for `Summary`, `Histogram`, and `ExponentialHistogram`.
struct CountSumCols<'a> {
    counts: OptValCol<'a, UInt64Type>,
    sums: OptValCol<'a, Float64Type>,
}

impl<'a> CountSumCols<'a> {
    fn from_rb(rb: &'a RecordBatch) -> Self {
        Self {
            counts: OptValCol::from_rb(rb, consts::COUNT),
            sums: OptValCol::from_rb(rb, consts::SUM),
        }
    }

    #[inline]
    fn init(&mut self, num_rows: usize, prefix_rows: usize) {
        self.counts.init(num_rows, prefix_rows);
        self.sums.init(num_rows, prefix_rows);
    }

    #[inline]
    fn read(&self, row: usize) -> (u64, Option<f64>) {
        (self.counts.read(row), self.sums.get_opt(row))
    }

    /// Reads `(count, Some(sum))` for `SummaryDataPoint`, where `sum` is a non-optional
    /// proto3 `double` whose omitted Arrow column represents `0.0` rather than an unknown sum.
    #[inline]
    fn read_summary(&self, row: usize) -> (u64, Option<f64>) {
        (self.counts.read(row), Some(self.sums.read(row)))
    }

    #[inline]
    fn push_passthrough(&mut self, row: usize) {
        self.counts.push_passthrough(row);
        self.sums.push_passthrough(row);
    }

    #[inline]
    fn push_raw(&mut self, count: u64, sum: Option<f64>) {
        self.counts.push(count);
        self.sums.push(sum.unwrap_or(0.0));
    }

    #[inline]
    fn push_sub(&mut self, count: u64, sum: Option<f64>, cs: &CountSumState) {
        self.counts.push(count.saturating_sub(cs.ref_count));
        if !cs.has_known_ref_sum() {
            self.sums.push_null();
        } else {
            self.sums.push(sum.map_or(0.0, |v| v - cs.ref_sum));
        }
    }

    #[inline]
    fn finish(self, keep_filter: Option<&BooleanArray>, out: &mut ReplList) {
        self.counts.finish(keep_filter, out);
        self.sums.finish(keep_filter, out);
    }
}

// --- 1. NumberDataPoints (`Sum`) ---

struct NumberAdjuster;
struct NumberTable<'a> {
    ints: OptValCol<'a, Int64Type>,
    doubles: OptValCol<'a, Float64Type>,
}

impl DpAdjuster for NumberAdjuster {
    const TABLE_IDX: usize = 0;
    const METRIC_TYPE: u8 = METRIC_TYPE_SUM;
    const DP_PAYLOAD: ArrowPayloadType = ArrowPayloadType::NumberDataPoints;
    const ATTRS_PAYLOAD: ArrowPayloadType = ArrowPayloadType::NumberDpAttrs;
    const EXEMPLARS_PAYLOAD: Option<(ArrowPayloadType, ArrowPayloadType)> = Some((
        ArrowPayloadType::NumberDpExemplars,
        ArrowPayloadType::NumberDpExemplarAttrs,
    ));
    type State = NumberState;
    type Table<'a> = NumberTable<'a>;

    fn try_table(rb: &RecordBatch) -> Result<Self::Table<'_>, PdataError> {
        Ok(NumberTable {
            ints: OptValCol::from_rb(rb, consts::INT_VALUE),
            doubles: OptValCol::from_rb(rb, consts::DOUBLE_VALUE),
        })
    }

    fn series_table(res: &mut ResourceSeriesMap) -> &mut SeriesTable<Self::State> {
        &mut res.numbers
    }

    fn init_outputs(t: &mut Self::Table<'_>, num_rows: usize, prefix_rows: usize) {
        t.ints.init(num_rows, prefix_rows);
        t.doubles.init(num_rows, prefix_rows);
    }

    #[inline]
    fn append_passthrough(t: &mut Self::Table<'_>, row: usize) {
        t.ints.push_passthrough(row);
        t.doubles.push_passthrough(row);
    }

    #[inline]
    fn seed_initial(t: &Self::Table<'_>, row: usize) -> Self::State {
        // Check `double_value` before `int_value` to match `otel_arrow_dfe_pdata` encoding precedence.
        match t.doubles.get_opt(row) {
            Some(cd) => NumberState::Double {
                ref_val: cd,
                prev_val: cd,
            },
            None => {
                let ci = t.ints.read(row);
                NumberState::Int {
                    ref_val: ci,
                    prev_val: ci,
                }
            }
        }
    }

    #[inline]
    fn adjust_occupied(t: &mut Self::Table<'_>, row: usize, s: &mut Self::State) -> bool {
        if let Some(cd) = t.doubles.get_opt(row) {
            let (out, reset) = if cd.is_nan() {
                (cd, false)
            } else if let NumberState::Double { ref_val, prev_val } = s
                && !prev_val.is_nan()
                && !ref_val.is_nan()
                && cd >= *prev_val
            {
                *prev_val = cd;
                (cd - *ref_val, false)
            } else {
                *s = NumberState::Double {
                    ref_val: 0.0,
                    prev_val: cd,
                };
                (cd, true)
            };
            t.doubles.push(out);
            t.ints.push(0);
            reset
        } else {
            let ci = t.ints.read(row);
            let (out, reset) = if let NumberState::Int { ref_val, prev_val } = s
                && ci >= *prev_val
            {
                *prev_val = ci;
                (ci.wrapping_sub(*ref_val), false)
            } else {
                *s = NumberState::Int {
                    ref_val: 0,
                    prev_val: ci,
                };
                (ci, true)
            };
            t.ints.push(out);
            t.doubles.push(0.0);
            reset
        }
    }

    fn finish_replacements(
        t: Self::Table<'_>,
        keep_filter: Option<&BooleanArray>,
        out: &mut ReplList,
    ) -> Result<(), PdataError> {
        t.ints.finish(keep_filter, out);
        t.doubles.finish(keep_filter, out);
        Ok(())
    }
}

// --- 2. SummaryDataPoints (`Summary`) ---

struct SummaryAdjuster;

impl DpAdjuster for SummaryAdjuster {
    const TABLE_IDX: usize = 1;
    const METRIC_TYPE: u8 = METRIC_TYPE_SUMMARY;
    const DP_PAYLOAD: ArrowPayloadType = ArrowPayloadType::SummaryDataPoints;
    const ATTRS_PAYLOAD: ArrowPayloadType = ArrowPayloadType::SummaryDpAttrs;
    const EXEMPLARS_PAYLOAD: Option<(ArrowPayloadType, ArrowPayloadType)> = None;
    type State = CountSumState;
    type Table<'a> = CountSumCols<'a>;

    fn try_table(rb: &RecordBatch) -> Result<Self::Table<'_>, PdataError> {
        Ok(CountSumCols::from_rb(rb))
    }

    fn series_table(res: &mut ResourceSeriesMap) -> &mut SeriesTable<Self::State> {
        &mut res.summaries
    }

    fn init_outputs(t: &mut Self::Table<'_>, num_rows: usize, prefix_rows: usize) {
        t.init(num_rows, prefix_rows);
    }

    #[inline]
    fn append_passthrough(t: &mut Self::Table<'_>, row: usize) {
        t.push_passthrough(row);
    }

    #[inline]
    fn seed_initial(t: &Self::Table<'_>, row: usize) -> Self::State {
        let (c, s) = t.read_summary(row);
        CountSumState::new(c, s)
    }

    #[inline]
    fn adjust_occupied(t: &mut Self::Table<'_>, row: usize, s: &mut Self::State) -> bool {
        let (cur_c, cur_s) = t.read_summary(row);
        let reset = s.decreased(cur_c, cur_s);
        if reset {
            s.reset(cur_c, cur_s);
            t.push_raw(cur_c, cur_s);
        } else {
            s.advance(cur_c, cur_s);
            t.push_sub(cur_c, cur_s, s);
        }
        reset
    }

    fn finish_replacements(
        t: Self::Table<'_>,
        keep_filter: Option<&BooleanArray>,
        out: &mut ReplList,
    ) -> Result<(), PdataError> {
        t.finish(keep_filter, out);
        Ok(())
    }
}

// --- 3. HistogramDataPoints (`Histogram`) ---

struct HistogramAdjuster;
struct HistogramTable<'a> {
    cs: CountSumCols<'a>,
    buckets: OptU64ListCol<'a>,
    bounds: Option<ListSliceAccessor<'a, Float64Type>>,
}

impl<'a> HistogramTable<'a> {
    #[inline]
    fn bounds_at(&self, row: usize) -> &'a [f64] {
        self.bounds
            .as_ref()
            .and_then(|a| a.slice_at(row))
            .unwrap_or(&[])
    }
}

impl DpAdjuster for HistogramAdjuster {
    const TABLE_IDX: usize = 2;
    const METRIC_TYPE: u8 = METRIC_TYPE_HISTOGRAM;
    const DP_PAYLOAD: ArrowPayloadType = ArrowPayloadType::HistogramDataPoints;
    const ATTRS_PAYLOAD: ArrowPayloadType = ArrowPayloadType::HistogramDpAttrs;
    const EXEMPLARS_PAYLOAD: Option<(ArrowPayloadType, ArrowPayloadType)> = Some((
        ArrowPayloadType::HistogramDpExemplars,
        ArrowPayloadType::HistogramDpExemplarAttrs,
    ));
    type State = HistogramState;
    type Table<'a> = HistogramTable<'a>;

    fn try_table(rb: &RecordBatch) -> Result<Self::Table<'_>, PdataError> {
        Ok(HistogramTable {
            cs: CountSumCols::from_rb(rb),
            buckets: OptU64ListCol::try_from_opt(
                rb.column_by_name(consts::HISTOGRAM_BUCKET_COUNTS),
            )?,
            bounds: rb
                .column_by_name(consts::HISTOGRAM_EXPLICIT_BOUNDS)
                .map(ListSliceAccessor::<Float64Type>::try_new)
                .transpose()?,
        })
    }

    fn series_table(res: &mut ResourceSeriesMap) -> &mut SeriesTable<Self::State> {
        &mut res.histograms
    }

    fn init_outputs(t: &mut Self::Table<'_>, num_rows: usize, prefix_rows: usize) {
        t.cs.init(num_rows, prefix_rows);
        t.buckets.init(num_rows, prefix_rows);
    }

    #[inline]
    fn append_passthrough(t: &mut Self::Table<'_>, row: usize) {
        t.cs.push_passthrough(row);
        t.buckets.push_passthrough(row);
    }

    #[inline]
    fn seed_initial(t: &Self::Table<'_>, row: usize) -> Self::State {
        let (c, s) = t.cs.read(row);
        let cur_b = t.buckets.slice_at(row).unwrap_or(&[]);
        HistogramState::new(c, s, hash_bounds(t.bounds_at(row)), cur_b)
    }

    #[inline]
    fn adjust_occupied(t: &mut Self::Table<'_>, row: usize, s: &mut Self::State) -> bool {
        let (cur_c, cur_s) = t.cs.read(row);
        let cur_b_opt = t.buckets.slice_at(row);
        let cur_b = cur_b_opt.unwrap_or(&[]);
        let half = s.buckets.len() / 2;
        let cur_bounds_hash = hash_bounds(t.bounds_at(row));
        let reset = s.cs.decreased(cur_c, cur_s)
            || cur_bounds_hash != s.bounds_hash
            || cur_b.len() != half
            || cur_b
                .iter()
                .zip(&s.buckets[half..])
                .any(|(&cb, &pb)| cb < pb);

        if reset {
            s.cs.reset(cur_c, cur_s);
            s.bounds_hash = cur_bounds_hash;
            if cur_b.len() != half {
                let mut buf = vec![0u64; cur_b.len() * 2].into_boxed_slice();
                buf[cur_b.len()..].copy_from_slice(cur_b);
                s.buckets = buf;
            } else {
                s.buckets[..half].fill(0);
                s.buckets[half..].copy_from_slice(cur_b);
            }
            t.cs.push_raw(cur_c, cur_s);
            t.buckets.push_raw(cur_b_opt);
        } else {
            s.cs.advance(cur_c, cur_s);
            t.cs.push_sub(cur_c, cur_s, &s.cs);
            t.buckets.push_subtracted(cur_b_opt, &s.buckets[..half]);
            s.buckets[half..].copy_from_slice(cur_b);
        }
        reset
    }

    fn finish_replacements(
        t: Self::Table<'_>,
        keep_filter: Option<&BooleanArray>,
        out: &mut ReplList,
    ) -> Result<(), PdataError> {
        t.cs.finish(keep_filter, out);
        if let Some(arr) = t.buckets.finish_list(keep_filter) {
            out.push(consts::HISTOGRAM_BUCKET_COUNTS, arr);
        }
        Ok(())
    }
}

// --- 4. ExpHistogramDataPoints (`ExponentialHistogram`) ---

struct ExpHistogramAdjuster;
struct ExpHistogramTable<'a> {
    cs: CountSumCols<'a>,
    zero_counts: OptValCol<'a, UInt64Type>,
    scales: Option<PrimCol<'a, i32>>,
    pos: Option<ExpBucketsCol<'a>>,
    neg: Option<ExpBucketsCol<'a>>,
}

impl DpAdjuster for ExpHistogramAdjuster {
    const TABLE_IDX: usize = 3;
    const METRIC_TYPE: u8 = METRIC_TYPE_EXP_HISTOGRAM;
    const DP_PAYLOAD: ArrowPayloadType = ArrowPayloadType::ExpHistogramDataPoints;
    const ATTRS_PAYLOAD: ArrowPayloadType = ArrowPayloadType::ExpHistogramDpAttrs;
    const EXEMPLARS_PAYLOAD: Option<(ArrowPayloadType, ArrowPayloadType)> = Some((
        ArrowPayloadType::ExpHistogramDpExemplars,
        ArrowPayloadType::ExpHistogramDpExemplarAttrs,
    ));
    type State = ExpHistogramState;
    type Table<'a> = ExpHistogramTable<'a>;

    fn try_table(rb: &RecordBatch) -> Result<Self::Table<'_>, PdataError> {
        Ok(ExpHistogramTable {
            cs: CountSumCols::from_rb(rb),
            zero_counts: OptValCol::from_rb(rb, consts::EXP_HISTOGRAM_ZERO_COUNT),
            scales: PrimCol::from_opt::<Int32Type>(rb, consts::EXP_HISTOGRAM_SCALE),
            pos: ExpBucketsCol::try_from_rb(rb, consts::EXP_HISTOGRAM_POSITIVE)?,
            neg: ExpBucketsCol::try_from_rb(rb, consts::EXP_HISTOGRAM_NEGATIVE)?,
        })
    }

    fn series_table(res: &mut ResourceSeriesMap) -> &mut SeriesTable<Self::State> {
        &mut res.exp_histograms
    }

    fn init_outputs(t: &mut Self::Table<'_>, num_rows: usize, prefix_rows: usize) {
        t.cs.init(num_rows, prefix_rows);
        t.zero_counts.init(num_rows, prefix_rows);
        for side in [&mut t.pos, &mut t.neg].into_iter().flatten() {
            side.buckets.init(num_rows, prefix_rows);
        }
    }

    #[inline]
    fn append_passthrough(t: &mut Self::Table<'_>, row: usize) {
        t.cs.push_passthrough(row);
        t.zero_counts.push_passthrough(row);
        for side in [&mut t.pos, &mut t.neg].into_iter().flatten() {
            side.buckets.push_passthrough(row);
        }
    }

    #[inline]
    fn seed_initial(t: &Self::Table<'_>, row: usize) -> Self::State {
        let (c, s) = t.cs.read(row);
        let zc = t.zero_counts.read(row);
        let scale = t.scales.map_or(0, |sc| sc.val_or_default(row));
        let (po, pc) = t
            .pos
            .as_ref()
            .map_or((0, &[][..]), |a| a.read_or_empty(row));
        let (no, nc) = t
            .neg
            .as_ref()
            .map_or((0, &[][..]), |a| a.read_or_empty(row));
        ExpHistogramState::new(scale, c, s, zc, po, pc, no, nc)
    }

    #[inline]
    fn adjust_occupied(t: &mut Self::Table<'_>, row: usize, s: &mut Self::State) -> bool {
        let (cur_c, cur_s) = t.cs.read(row);
        let cur_z = t.zero_counts.read(row);
        let cur_scale = t.scales.map_or(0, |sc| sc.val_or_default(row));
        let (po, p_opt) = t.pos.as_ref().map_or((0, None), |a| a.row_at(row));
        let (no, n_opt) = t.neg.as_ref().map_or((0, None), |a| a.row_at(row));
        let (pc, nc) = (p_opt.unwrap_or(&[]), n_opt.unwrap_or(&[]));

        let (prev_po, prev_pc) = s.prev_pos();
        let (prev_no, prev_nc) = s.prev_neg();
        let reset = s.cs.decreased(cur_c, cur_s)
            || cur_z < s.prev_zero_count
            || cur_scale != s.scale
            || exp_buckets_decreased(po, pc, prev_po, prev_pc)
            || exp_buckets_decreased(no, nc, prev_no, prev_nc);

        if reset {
            s.reset_and_update_prev(cur_scale, cur_c, cur_s, cur_z, po, pc, no, nc);
            t.cs.push_raw(cur_c, cur_s);
            t.zero_counts.push(cur_z);
            if let Some(p) = t.pos.as_mut() {
                p.buckets.push_raw(p_opt);
            }
            if let Some(n) = t.neg.as_mut() {
                n.buckets.push_raw(n_opt);
            }
        } else {
            s.cs.advance(cur_c, cur_s);
            t.cs.push_sub(cur_c, cur_s, &s.cs);
            t.zero_counts.push(cur_z.saturating_sub(s.ref_zero_count));
            let (ref_po, ref_pc) = s.ref_pos();
            let (ref_no, ref_nc) = s.ref_neg();
            if let Some(p) = t.pos.as_mut() {
                p.push_exp_sub(po, p_opt, ref_po, ref_pc);
            }
            if let Some(n) = t.neg.as_mut() {
                n.push_exp_sub(no, n_opt, ref_no, ref_nc);
            }
            s.update_prev(cur_z, po, pc, no, nc);
        }
        reset
    }

    fn finish_replacements(
        t: Self::Table<'_>,
        keep_filter: Option<&BooleanArray>,
        out: &mut ReplList,
    ) -> Result<(), PdataError> {
        t.cs.finish(keep_filter, out);
        t.zero_counts.finish(keep_filter, out);
        for (name, side) in [
            (consts::EXP_HISTOGRAM_POSITIVE, t.pos),
            (consts::EXP_HISTOGRAM_NEGATIVE, t.neg),
        ] {
            if let Some(s) = side {
                out.push(name, s.finish(keep_filter)?);
            }
        }
        Ok(())
    }
}

#[inline]
fn hash_bounds(bounds: &[f64]) -> u64 {
    let mut h = (bounds.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for &b in bounds {
        let k = b.to_bits().wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = (h ^ k).rotate_left(27).wrapping_mul(0x94D0_49BB_1331_11EB);
    }
    h ^ (h >> 31)
}

#[inline]
fn exp_bucket_at(counts: &[u64], idx: usize, offset_diff: i64) -> u64 {
    i64::try_from(idx)
        .ok()
        .and_then(|i| i.checked_add(offset_diff))
        .and_then(|i| usize::try_from(i).ok())
        .and_then(|i| counts.get(i).copied())
        .unwrap_or(0)
}

#[inline]
fn exp_buckets_decreased(
    cur_offset: i32,
    cur_counts: &[u64],
    prev_offset: i32,
    prev_counts: &[u64],
) -> bool {
    let offset_diff = i64::from(prev_offset) - i64::from(cur_offset);
    prev_counts
        .iter()
        .enumerate()
        .any(|(i, &prev_val)| exp_bucket_at(cur_counts, i, offset_diff) < prev_val)
}

struct ExpBucketsCol<'a> {
    struct_arr: &'a StructArray,
    offsets: Option<PrimCol<'a, i32>>,
    buckets: OptU64ListCol<'a>,
}

impl<'a> ExpBucketsCol<'a> {
    fn try_from_rb(rb: &'a RecordBatch, name: &str) -> Result<Option<Self>, PdataError> {
        let Some(col) = rb.column_by_name(name) else {
            return Ok(None);
        };
        let struct_arr = col.as_any().downcast_ref::<StructArray>().ok_or_else(|| {
            PdataError::ColumnDataTypeMismatch {
                name: name.into(),
                actual: col.data_type().clone(),
                expect: DataType::Struct(arrow::datatypes::Fields::empty()),
            }
        })?;
        let offsets = struct_arr
            .column_by_name(consts::EXP_HISTOGRAM_OFFSET)
            .and_then(PrimCol::from_array::<Int32Type>);
        let buckets = OptU64ListCol::try_from_opt(
            struct_arr.column_by_name(consts::EXP_HISTOGRAM_BUCKET_COUNTS),
        )?;
        Ok(Some(Self {
            struct_arr,
            offsets,
            buckets,
        }))
    }

    #[inline]
    fn row_at(&self, row: usize) -> (i32, Option<&'a [u64]>) {
        if self.struct_arr.nulls().is_some_and(|n| n.is_null(row)) {
            return (0, None);
        }
        (
            self.offsets.map_or(0, |c| c.val_or_default(row)),
            self.buckets.slice_at(row),
        )
    }

    #[inline]
    fn read_or_empty(&self, row: usize) -> (i32, &'a [u64]) {
        let (off, opt) = self.row_at(row);
        (off, opt.unwrap_or(&[]))
    }

    #[inline]
    fn push_exp_sub(
        &mut self,
        cur_off: i32,
        cur_opt: Option<&[u64]>,
        ref_off: i32,
        ref_counts: &[u64],
    ) {
        if self.buckets.src.is_none() {
            return;
        }
        if let Some(cur_counts) = cur_opt {
            let offset_diff = i64::from(cur_off) - i64::from(ref_off);
            for (i, &cur_val) in cur_counts.iter().enumerate() {
                self.buckets
                    .values
                    .push(cur_val.saturating_sub(exp_bucket_at(ref_counts, i, offset_diff)));
            }
        }
        self.buckets.offsets.push(self.buckets.values.len() as i32);
    }

    fn finish(self, keep_filter: Option<&BooleanArray>) -> Result<ArrayRef, PdataError> {
        let new_buckets = self.buckets.finish_list(keep_filter);
        let mut columns = Vec::with_capacity(self.struct_arr.columns().len());
        for (field, col) in self
            .struct_arr
            .fields()
            .iter()
            .zip(self.struct_arr.columns())
        {
            if field.name().as_str() == consts::EXP_HISTOGRAM_BUCKET_COUNTS
                && let Some(new_col) = new_buckets.as_ref()
            {
                columns.push(Arc::clone(new_col));
            } else {
                columns.push(filter_or_clone(col, keep_filter)?);
            }
        }
        let nulls = filter_nulls(self.struct_arr.nulls(), keep_filter);
        let struct_arr = StructArray::try_new(self.struct_arr.fields().clone(), columns, nulls)
            .map_err(|source| PdataError::ColumnLengthMismatch { source })?;
        Ok(Arc::new(struct_arr))
    }
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
        // Forward non-metric payloads without materializing OTLP bytes into OTAP.
        if let Message::PData(pdata) = msg {
            if pdata.signal_type() != SignalType::Metrics {
                effect_handler.send_message_with_source_node(pdata).await?;
                return Ok(());
            }
            return NativeProcessorAdapter::process(self, Message::PData(pdata), effect_handler)
                .await;
        }
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
            _ = metrics_reporter.report(&mut self.metrics);
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
        debug_assert_eq!(signal, SignalType::Metrics);

        let (context, mut arrow_records) = arrow_pdata.into_parts();

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

        self.metrics.dropped_items.add(stats.dropped_initial);
        self.metrics.adjusted_data_points.add(stats.adjusted_points);
        self.metrics.resets_detected.add(stats.resets_detected);

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
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, DataPointFlags, Exemplar, ExponentialHistogram,
        ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint, Metric, MetricsData,
        NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_dfe_pdata::testing::equiv::assert_equivalent;
    use otel_arrow_dfe_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;

    const CUM: AggregationTemporality = AggregationTemporality::Cumulative;
    const DELTA: AggregationTemporality = AggregationTemporality::Delta;

    fn test_processor(gc_interval: Duration) -> MetricStartTimeProcessor {
        let controller = ControllerContext::new(TelemetryRegistryHandle::new());
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

    fn metrics_batch(res_attrs: Vec<KeyValue>, metrics: Vec<Metric>) -> MetricsData {
        MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().attributes(res_attrs).finish(),
            vec![ScopeMetrics::new(InstrumentationScope::default(), metrics)],
        )])
    }

    fn one_batch(metrics: Vec<Metric>) -> MetricsData {
        metrics_batch(vec![], metrics)
    }

    fn sum_metric(name: &str, temp: AggregationTemporality, pts: Vec<NumberDataPoint>) -> Metric {
        Metric::build()
            .name(name)
            .data_sum(Sum::new(temp, true, pts))
            .finish()
    }

    fn summary_metric(name: &str, pts: Vec<SummaryDataPoint>) -> Metric {
        Metric::build()
            .name(name)
            .data_summary(Summary::new(pts))
            .finish()
    }

    fn hist_metric(
        name: &str,
        temp: AggregationTemporality,
        pts: Vec<HistogramDataPoint>,
    ) -> Metric {
        Metric::build()
            .name(name)
            .data_histogram(Histogram::new(temp, pts))
            .finish()
    }

    fn exp_hist_metric(
        name: &str,
        temp: AggregationTemporality,
        pts: Vec<ExponentialHistogramDataPoint>,
    ) -> Metric {
        Metric::build()
            .name(name)
            .data_exponential_histogram(ExponentialHistogram::new(temp, pts))
            .finish()
    }

    fn num_dp(start: u64, ts: u64, attrs: Vec<KeyValue>, val: i64) -> NumberDataPoint {
        NumberDataPoint::build()
            .start_time_unix_nano(start)
            .time_unix_nano(ts)
            .attributes(attrs)
            .value_int(val)
            .finish()
    }

    fn num_dp_f64(start: u64, ts: u64, attrs: Vec<KeyValue>, val: f64) -> NumberDataPoint {
        NumberDataPoint::build()
            .start_time_unix_nano(start)
            .time_unix_nano(ts)
            .attributes(attrs)
            .value_double(val)
            .finish()
    }

    fn summary_dp(
        start: u64,
        ts: u64,
        attrs: Vec<KeyValue>,
        c: u64,
        s: f64,
        q: Vec<ValueAtQuantile>,
    ) -> SummaryDataPoint {
        SummaryDataPoint::build()
            .start_time_unix_nano(start)
            .time_unix_nano(ts)
            .attributes(attrs)
            .count(c)
            .sum(s)
            .quantile_values(q)
            .finish()
    }

    fn hist_dp(
        start: u64,
        ts: u64,
        attrs: Vec<KeyValue>,
        c: u64,
        s: f64,
        bounds: Vec<f64>,
        buckets: Vec<u64>,
    ) -> HistogramDataPoint {
        HistogramDataPoint::build()
            .start_time_unix_nano(start)
            .time_unix_nano(ts)
            .attributes(attrs)
            .count(c)
            .sum(s)
            .explicit_bounds(bounds)
            .bucket_counts(buckets)
            .finish()
    }

    #[allow(clippy::too_many_arguments)]
    fn exp_hist_dp(
        start: u64,
        ts: u64,
        attrs: Vec<KeyValue>,
        scale: i32,
        c: u64,
        s: f64,
        z: u64,
        pos: Buckets,
        neg: Buckets,
    ) -> ExponentialHistogramDataPoint {
        ExponentialHistogramDataPoint::build()
            .start_time_unix_nano(start)
            .time_unix_nano(ts)
            .attributes(attrs)
            .scale(scale)
            .count(c)
            .sum(s)
            .zero_count(z)
            .positive(pos)
            .negative(neg)
            .finish()
    }

    fn ex(tag: &str, v: i64) -> Vec<Exemplar> {
        vec![
            Exemplar::build()
                .time_unix_nano(20_000_000u64)
                .value_int(v)
                .filtered_attributes(vec![kv_str("trace", tag)])
                .finish(),
        ]
    }

    fn with_num_ex(mut dp: NumberDataPoint, tag: &str, v: i64) -> NumberDataPoint {
        dp.exemplars = ex(tag, v);
        dp
    }

    fn with_hist_ex(mut dp: HistogramDataPoint, tag: &str, v: i64) -> HistogramDataPoint {
        dp.exemplars = ex(tag, v);
        dp
    }

    fn with_eh_ex(
        mut dp: ExponentialHistogramDataPoint,
        tag: &str,
        v: i64,
    ) -> ExponentialHistogramDataPoint {
        dp.exemplars = ex(tag, v);
        dp
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

    fn assert_metrics_eq(actual: Option<MetricsData>, expected: MetricsData) {
        assert_equivalent(
            &[OtlpProtoMessage::Metrics(actual.unwrap())],
            &[OtlpProtoMessage::Metrics(expected)],
        );
    }

    fn assert_step(
        proc: &mut MetricStartTimeProcessor,
        now: Instant,
        input: MetricsData,
        exp_stats: (u64, u64, u64),
        expected: Option<MetricsData>,
    ) {
        let (out, s) = run_batch(proc, input, now);
        assert_eq!(
            (s.dropped_initial, s.adjusted_points, s.resets_detected),
            exp_stats
        );
        match expected {
            None => assert!(out.is_none()),
            Some(exp) => assert_metrics_eq(out, exp),
        }
    }

    fn assert_rows(records: &OtapArrowRecords, expected: &[(ArrowPayloadType, Option<usize>)]) {
        for &(pt, rows) in expected {
            assert_eq!(
                records.get(pt).map(RecordBatch::num_rows),
                rows,
                "payload {pt:?}"
            );
        }
    }

    /// Scenario: Cumulative Sum series observed across initial scrape, monotonic increase, and counter reset.
    /// Guarantees: Initial points are cached and dropped, subsequent points subtract the initial baseline with start_time from the initial point, and resets set start_time to time - 1ms with a zeroed baseline.
    #[test]
    fn test_cumulative_sum_subtract_initial_and_reset() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let make = |sg: u64, sp: u64, ts: u64, a: f64, b: f64| {
            metrics_batch(
                vec![kv_str("service.name", "checkout")],
                vec![sum_metric(
                    "http_requests_total",
                    CUM,
                    vec![
                        num_dp_f64(sg, ts, vec![kv_str("method", "GET")], a),
                        num_dp_f64(sp, ts, vec![kv_str("method", "POST")], b),
                    ],
                )],
            )
        };

        assert_step(
            &mut proc,
            t0,
            make(0, 0, 10_000_000, 100.0, 50.0),
            (2, 0, 0),
            None,
        );
        assert_step(
            &mut proc,
            t0,
            make(0, 0, 20_000_000, 135.0, 55.0),
            (0, 2, 0),
            Some(make(10_000_000, 10_000_000, 20_000_000, 35.0, 5.0)),
        );
        assert_step(
            &mut proc,
            t0,
            make(0, 0, 30_000_000, 12.0, 70.0),
            (0, 2, 1),
            Some(make(29_000_000, 10_000_000, 30_000_000, 12.0, 20.0)),
        );
        assert_step(
            &mut proc,
            t0,
            make(0, 0, 40_000_000, 19.0, 75.0),
            (0, 2, 0),
            Some(make(29_000_000, 10_000_000, 40_000_000, 19.0, 25.0)),
        );
    }

    /// Scenario: Mixed batch with Gauge, Delta Sum, Cumulative Sum with existing start_time, and staleness marker (FLAG_NO_RECORDED_VALUE).
    /// Guarantees: Ineligible and pre-adjusted points pass through untouched, while FLAG_NO_RECORDED_VALUE points receive the cached start_time without altering reference state.
    #[test]
    fn test_passthrough_gauge_delta_existing_start_time_and_no_recorded_value() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();

        let kept = vec![
            Metric::build()
                .name("cpu_gauge")
                .data_gauge(Gauge::new(vec![num_dp(0, 10_000_000, vec![], 42)]))
                .finish(),
            sum_metric(
                "delta_counter",
                DELTA,
                vec![num_dp(0, 10_000_000, vec![], 7)],
            ),
            sum_metric(
                "already_adjusted_counter",
                CUM,
                vec![num_dp(5_000_000, 10_000_000, vec![], 99)],
            ),
        ];
        let mut batch1 = kept.clone();
        batch1.push(sum_metric(
            "unadjusted_counter",
            CUM,
            vec![num_dp(0, 10_000_000, vec![], 100)],
        ));
        assert_step(
            &mut proc,
            t0,
            one_batch(batch1),
            (1, 0, 0),
            Some(one_batch(kept)),
        );

        let preset = one_batch(vec![sum_metric(
            "already_adjusted_counter",
            CUM,
            vec![num_dp(5_000_000, 15_000_000, vec![], 110)],
        )]);
        assert_step(&mut proc, t0, preset.clone(), (0, 0, 0), Some(preset));

        let stale_dp = |start: u64| {
            let mut b = NumberDataPoint::build()
                .time_unix_nano(20_000_000u64)
                .flags(DataPointFlags::NoRecordedValueMask as u32);
            if start != 0 {
                b = b.start_time_unix_nano(start);
            }
            one_batch(vec![sum_metric(
                "unadjusted_counter",
                CUM,
                vec![b.finish()],
            )])
        };
        assert_step(
            &mut proc,
            t0,
            stale_dp(0),
            (0, 1, 0),
            Some(stale_dp(10_000_000)),
        );
        assert_step(
            &mut proc,
            t0,
            one_batch(vec![sum_metric(
                "unadjusted_counter",
                CUM,
                vec![num_dp(0, 30_000_000, vec![], 125)],
            )]),
            (0, 1, 0),
            Some(one_batch(vec![sum_metric(
                "unadjusted_counter",
                CUM,
                vec![num_dp(10_000_000, 30_000_000, vec![], 25)],
            )])),
        );
    }

    /// Scenario: Cumulative Summary, Histogram, and ExponentialHistogram series across initial and subsequent scrapes with shifted bucket offset.
    /// Guarantees: Initial points are dropped, and subsequent points subtract reference counts, sums, and offset-aligned buckets while preserving quantiles.
    #[test]
    fn test_summary_histogram_and_exp_histogram_adjustment() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let make = |st: u64,
                    ts: u64,
                    (sc, ss): (u64, f64),
                    (hc, hs, hb): (u64, f64, Vec<u64>),
                    (ec, es, ez, eo, eb): (u64, f64, u64, i32, Vec<u64>)| {
            one_batch(vec![
                summary_metric(
                    "rpc_summary",
                    vec![summary_dp(
                        st,
                        ts,
                        vec![],
                        sc,
                        ss,
                        vec![ValueAtQuantile::new(0.99, 0.42)],
                    )],
                ),
                hist_metric(
                    "rpc_hist",
                    CUM,
                    vec![hist_dp(st, ts, vec![], hc, hs, vec![1.0, 5.0], hb)],
                ),
                exp_hist_metric(
                    "rpc_exp_hist",
                    CUM,
                    vec![exp_hist_dp(
                        st,
                        ts,
                        vec![],
                        2,
                        ec,
                        es,
                        ez,
                        Buckets::new(eo, eb),
                        Buckets::new(0, vec![]),
                    )],
                ),
            ])
        };

        assert_step(
            &mut proc,
            t0,
            make(
                0,
                10_000_000,
                (10, 100.0),
                (10, 50.0, vec![2, 5, 3]),
                (10, 50.0, 2, 1, vec![3, 5]),
            ),
            (3, 0, 0),
            None,
        );
        assert_step(
            &mut proc,
            t0,
            make(
                0,
                20_000_000,
                (15, 160.0),
                (16, 80.0, vec![4, 8, 4]),
                (18, 90.0, 3, 0, vec![4, 7, 5]),
            ),
            (0, 3, 0),
            Some(make(
                10_000_000,
                20_000_000,
                (5, 60.0),
                (6, 30.0, vec![2, 3, 1]),
                (8, 40.0, 1, 0, vec![4, 4, 0]),
            )),
        );
    }

    /// Scenario: Summary, Histogram, and ExponentialHistogram series with passthrough rows, staleness flags, resets, and post-reset scrapes.
    /// Guarantees: Delta and pre-adjusted points pass through, FLAG_NO_RECORDED_VALUE sets start_time without changing state, resets set start_time to time - 1ms, and post-reset points subtract zero baseline.
    #[test]
    fn test_summary_histogram_and_exp_histogram_resets_staleness_and_passthrough() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let (tr, pr) = (
            || vec![kv_str("s", "tracked")],
            || vec![kv_str("s", "preset")],
        );
        let h_dp =
            |st, ts, a, c, s, b: &[u64]| hist_dp(st, ts, a, c, s, vec![1.0, 5.0], b.to_vec());
        let e_dp = |st, ts, a, sc, c, s, z, po, pb: &[u64], nb: &[u64]| {
            exp_hist_dp(
                st,
                ts,
                a,
                sc,
                c,
                s,
                z,
                Buckets::new(po, pb.to_vec()),
                Buckets::new(0, nb.to_vec()),
            )
        };

        let make_mixed = |with_tracked: bool| {
            let mut sum_pts = vec![summary_dp(1_000_000, 10_000_000, pr(), 5, 50.0, vec![])];
            let mut hist_pts = vec![h_dp(1_000_000, 10_000_000, pr(), 4, 20.0, &[1, 2, 1])];
            let mut eh_pts = vec![e_dp(
                1_000_000,
                10_000_000,
                pr(),
                2,
                4,
                20.0,
                1,
                0,
                &[3],
                &[],
            )];
            if with_tracked {
                sum_pts.push(summary_dp(0, 10_000_000, tr(), 10, 100.0, vec![]));
                hist_pts.push(h_dp(0, 10_000_000, tr(), 10, 50.0, &[2, 5, 3]));
                eh_pts.push(e_dp(0, 10_000_000, tr(), 2, 10, 50.0, 2, 1, &[3, 5], &[1]));
            }
            one_batch(vec![
                summary_metric("summary_m", sum_pts),
                hist_metric(
                    "hist_delta",
                    DELTA,
                    vec![h_dp(0, 10_000_000, vec![], 3, 15.0, &[1, 1, 1])],
                ),
                hist_metric("hist_cum", CUM, hist_pts),
                exp_hist_metric(
                    "eh_delta",
                    DELTA,
                    vec![e_dp(0, 10_000_000, vec![], 1, 2, 10.0, 0, 0, &[2], &[])],
                ),
                exp_hist_metric("eh_cum", CUM, eh_pts),
            ])
        };
        assert_step(
            &mut proc,
            t0,
            make_mixed(true),
            (3, 0, 0),
            Some(make_mixed(false)),
        );

        let stale = DataPointFlags::NoRecordedValueMask as u32;
        let make_stale = |st: u64| {
            one_batch(vec![
                summary_metric(
                    "summary_m",
                    vec![
                        SummaryDataPoint::build()
                            .start_time_unix_nano(st)
                            .time_unix_nano(20_000_000u64)
                            .attributes(tr())
                            .flags(stale)
                            .finish(),
                    ],
                ),
                hist_metric(
                    "hist_cum",
                    CUM,
                    vec![
                        HistogramDataPoint::build()
                            .start_time_unix_nano(st)
                            .time_unix_nano(20_000_000u64)
                            .attributes(tr())
                            .flags(stale)
                            .finish(),
                    ],
                ),
                exp_hist_metric(
                    "eh_cum",
                    CUM,
                    vec![
                        ExponentialHistogramDataPoint::build()
                            .start_time_unix_nano(st)
                            .time_unix_nano(20_000_000u64)
                            .attributes(tr())
                            .flags(stale)
                            .finish(),
                    ],
                ),
            ])
        };
        assert_step(
            &mut proc,
            t0,
            make_stale(0),
            (0, 3, 0),
            Some(make_stale(10_000_000)),
        );

        let tracked = |st,
                       ts,
                       (sc, ss),
                       (hc, hs, hb): (u64, f64, &[u64]),
                       (ec, es, ez, eb): (u64, f64, u64, &[u64])| {
            one_batch(vec![
                summary_metric("summary_m", vec![summary_dp(st, ts, tr(), sc, ss, vec![])]),
                hist_metric("hist_cum", CUM, vec![h_dp(st, ts, tr(), hc, hs, hb)]),
                exp_hist_metric(
                    "eh_cum",
                    CUM,
                    vec![e_dp(st, ts, tr(), 2, ec, es, ez, 2, eb, &[1])],
                ),
            ])
        };
        for &(ts, s, h, e, stats) in &[
            (
                30_000_000,
                (4, 40.0),
                (12, 60.0, &[1, 6, 5][..]),
                (12, 60.0, 3, &[4, 5][..]),
                (0, 3, 3),
            ),
            (
                40_000_000,
                (7, 70.0),
                (15, 75.0, &[2, 7, 6][..]),
                (14, 70.0, 4, &[5, 6][..]),
                (0, 3, 0),
            ),
        ] {
            assert_step(
                &mut proc,
                t0,
                tracked(0, ts, s, h, e),
                stats,
                Some(tracked(29_000_000, ts, s, h, e)),
            );
        }
    }

    /// Scenario: A batch contains both an existing series and a newly introduced series with data point attributes and exemplars.
    /// Guarantees: Dropping the new series' initial point filters out its child attributes, exemplars, and exemplar attributes while preserving the surviving point's children.
    #[test]
    fn test_partial_drop_filters_dp_attrs_and_exemplars() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let s = |v: &str| vec![kv_str("series", v)];
        assert_step(
            &mut proc,
            t0,
            one_batch(vec![sum_metric(
                "work_total",
                CUM,
                vec![num_dp(0, 10_000_000, s("A"), 10)],
            )]),
            (1, 0, 0),
            None,
        );

        let batch2 = one_batch(vec![sum_metric(
            "work_total",
            CUM,
            vec![
                with_num_ex(num_dp(0, 20_000_000, s("A"), 25), "keep", 15),
                with_num_ex(num_dp(0, 20_000_000, s("B"), 100), "drop", 100),
            ],
        )]);
        let mut records = otlp_to_otap(&OtlpProtoMessage::Metrics(batch2));
        let s2 = proc.engine.process_metrics_batch(&mut records, t0).unwrap();
        assert_eq!((s2.dropped_initial, s2.adjusted_points), (1, 1));
        assert_rows(
            &records,
            &[
                (ArrowPayloadType::NumberDataPoints, Some(1)),
                (ArrowPayloadType::NumberDpAttrs, Some(1)),
                (ArrowPayloadType::NumberDpExemplars, Some(1)),
                (ArrowPayloadType::NumberDpExemplarAttrs, Some(1)),
            ],
        );
        let OtlpProtoMessage::Metrics(out2) = otap_to_otlp(&records) else {
            panic!("expected metrics");
        };
        assert_metrics_eq(
            Some(out2),
            one_batch(vec![sum_metric(
                "work_total",
                CUM,
                vec![with_num_ex(
                    num_dp(10_000_000, 20_000_000, s("A"), 15),
                    "keep",
                    15,
                )],
            )]),
        );
    }

    /// Scenario: Mixed batch partially drops new Summary/Histogram/ExpHistogram series while dropping all rows from NumberDataPoints.
    /// Guarantees: Surviving child tables retain only kept rows, empty exemplar tables remove their exemplar attribute tables, and AllDropped removes the entire DP table and all its children.
    #[test]
    fn test_partial_drop_and_all_dropped_child_cleanup_across_metric_types() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let (a, b) = (|| vec![kv_str("s", "A")], || vec![kv_str("s", "B")]);
        let eh = |ts, attrs, c, s, pb| {
            exp_hist_dp(
                0,
                ts,
                attrs,
                1,
                c,
                s,
                0,
                Buckets::new(0, vec![pb]),
                Buckets::new(0, vec![]),
            )
        };

        let seed = one_batch(vec![
            summary_metric(
                "sum_m",
                vec![summary_dp(0, 10_000_000, a(), 2, 10.0, vec![])],
            ),
            hist_metric(
                "hist_m",
                CUM,
                vec![hist_dp(0, 10_000_000, a(), 2, 10.0, vec![1.0], vec![1, 1])],
            ),
            exp_hist_metric("eh_m", CUM, vec![eh(10_000_000, a(), 2, 10.0, 2)]),
        ]);
        assert_step(&mut proc, t0, seed, (3, 0, 0), None);

        let mixed = one_batch(vec![
            sum_metric(
                "new_sum_only",
                CUM,
                vec![with_num_ex(
                    num_dp(0, 20_000_000, vec![kv_str("s", "C")], 5),
                    "drop-num",
                    1,
                )],
            ),
            summary_metric(
                "sum_m",
                vec![
                    summary_dp(0, 20_000_000, a(), 5, 25.0, vec![]),
                    summary_dp(0, 20_000_000, b(), 3, 15.0, vec![]),
                ],
            ),
            hist_metric(
                "hist_m",
                CUM,
                vec![
                    hist_dp(0, 20_000_000, a(), 5, 25.0, vec![1.0], vec![2, 3]),
                    with_hist_ex(
                        hist_dp(0, 20_000_000, b(), 1, 2.0, vec![1.0], vec![1, 0]),
                        "drop-hist",
                        1,
                    ),
                ],
            ),
            exp_hist_metric(
                "eh_m",
                CUM,
                vec![
                    with_eh_ex(eh(20_000_000, a(), 4, 20.0, 4), "keep-eh", 1),
                    with_eh_ex(eh(20_000_000, b(), 1, 3.0, 1), "drop-eh", 1),
                ],
            ),
        ]);
        let mut records = otlp_to_otap(&OtlpProtoMessage::Metrics(mixed));
        let stats = proc.engine.process_metrics_batch(&mut records, t0).unwrap();
        assert_eq!((stats.dropped_initial, stats.adjusted_points), (4, 3));
        assert_rows(
            &records,
            &[
                (ArrowPayloadType::NumberDataPoints, None),
                (ArrowPayloadType::NumberDpAttrs, None),
                (ArrowPayloadType::NumberDpExemplars, None),
                (ArrowPayloadType::NumberDpExemplarAttrs, None),
                (ArrowPayloadType::SummaryDpAttrs, Some(1)),
                (ArrowPayloadType::HistogramDpAttrs, Some(1)),
                (ArrowPayloadType::HistogramDpExemplars, None),
                (ArrowPayloadType::HistogramDpExemplarAttrs, None),
                (ArrowPayloadType::ExpHistogramDpAttrs, Some(1)),
                (ArrowPayloadType::ExpHistogramDpExemplars, Some(1)),
                (ArrowPayloadType::ExpHistogramDpExemplarAttrs, Some(1)),
            ],
        );
    }

    /// Scenario: Hashing multi-type attributes (str, int, double, bool, bytes, map, slice) across reordered batches, default-zero values, and transport-optimized IDs.
    /// Guarantees: AttrHasher produces identical 128-bit multiset hashes regardless of attribute row order, omitted default-zero columns, or transport-optimized ID encodings.
    #[test]
    fn test_attr_hasher_order_independence_all_value_types_and_transport_ids() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let attrs1 = vec![
            kv_str("k_str", "v"),
            KeyValue::new("k_int_zero", AnyValue::new_int(0)),
            KeyValue::new("k_double_zero", AnyValue::new_double(0.0)),
            KeyValue::new("k_bool", AnyValue::new_bool(true)),
            KeyValue::new("k_bytes", AnyValue::new_bytes(b"abc".to_vec())),
            KeyValue::new("k_map", AnyValue::new_kvlist(vec![kv_str("nested", "m1")])),
            KeyValue::new("k_slice", AnyValue::new_array(vec![AnyValue::new_int(7)])),
        ];
        let mut attrs2 = attrs1.clone();
        attrs2.reverse();

        assert_step(
            &mut proc,
            t0,
            metrics_batch(
                attrs1,
                vec![sum_metric(
                    "typed_counter",
                    CUM,
                    vec![num_dp(0, 10_000_000, attrs2.clone(), 50)],
                )],
            ),
            (1, 0, 0),
            None,
        );

        let mut attrs_diff_bytes = attrs2.clone();
        for kv in &mut attrs_diff_bytes {
            if kv.key == "k_bytes" {
                kv.value = Some(AnyValue::new_bytes(b"xyz".to_vec()));
            }
        }
        let batch2 = metrics_batch(
            attrs2.clone(),
            vec![sum_metric(
                "typed_counter",
                CUM,
                vec![
                    num_dp(0, 20_000_000, attrs2, 80),
                    num_dp(
                        0,
                        20_000_000,
                        vec![
                            KeyValue::new("k_int_nonzero", AnyValue::new_int(42)),
                            KeyValue::new("k_double_nonzero", AnyValue::new_double(3.5)),
                        ],
                        10,
                    ),
                    num_dp(0, 20_000_000, attrs_diff_bytes, 10),
                ],
            )],
        );
        let mut wire = otlp_to_otap(&OtlpProtoMessage::Metrics(batch2));
        wire.encode_transport_optimized().unwrap();
        assert!(!all_ids_already_plain(&wire));
        let s2 = proc.engine.process_metrics_batch(&mut wire, t0).unwrap();
        assert_eq!((s2.adjusted_points, s2.dropped_initial), (1, 2));
    }

    /// Scenario: Resource and data-point attributes contain nested KeyValueList maps and arrays of maps whose keys arrive in different orders across batches, plus an array with reordered elements.
    /// Guarantees: Reordered nested map keys (including inside arrays) match the same series and adjust the point, while reordered array elements create a distinct series.
    #[test]
    fn test_reordered_nested_map_attributes_match_series_and_preserve_array_order() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();

        let mk_nested = |flip_maps: bool, flip_arr: bool| {
            let mut inner = vec![kv_str("k1", "v1"), kv_str("k2", "v2")];
            let mut elem_a = vec![kv_str("a", "1"), KeyValue::new("n", AnyValue::new_int(10))];
            let mut elem_b = vec![kv_str("b", "2"), kv_str("c", "3")];
            if flip_maps {
                inner.reverse();
                elem_a.reverse();
                elem_b.reverse();
            }
            let mut arr_elems = vec![AnyValue::new_kvlist(elem_a), AnyValue::new_kvlist(elem_b)];
            if flip_arr {
                arr_elems.reverse();
            }
            let mut outer = vec![
                KeyValue::new("nested_map", AnyValue::new_kvlist(inner)),
                KeyValue::new("arr_of_maps", AnyValue::new_array(arr_elems)),
            ];
            if flip_maps {
                outer.reverse();
            }
            vec![
                KeyValue::new("meta", AnyValue::new_kvlist(outer)),
                KeyValue::new(
                    "list",
                    AnyValue::new_array(vec![AnyValue::new_kvlist(if flip_maps {
                        vec![kv_str("p", "2"), kv_str("q", "1")]
                    } else {
                        vec![kv_str("q", "1"), kv_str("p", "2")]
                    })]),
                ),
            ]
        };

        // 1. Seed with original map key order in both resource and data-point attributes.
        assert_step(
            &mut proc,
            t0,
            metrics_batch(
                mk_nested(false, false),
                vec![sum_metric(
                    "nested_attr_counter",
                    CUM,
                    vec![num_dp(0, 10_000_000, mk_nested(false, false), 40)],
                )],
            ),
            (1, 0, 0),
            None,
        );

        // 2. Reordering map keys (including maps inside arrays) adjusts the existing series.
        assert_step(
            &mut proc,
            t0,
            metrics_batch(
                mk_nested(true, false),
                vec![sum_metric(
                    "nested_attr_counter",
                    CUM,
                    vec![num_dp(0, 20_000_000, mk_nested(true, false), 65)],
                )],
            ),
            (0, 1, 0),
            Some(metrics_batch(
                mk_nested(true, false),
                vec![sum_metric(
                    "nested_attr_counter",
                    CUM,
                    vec![num_dp(10_000_000, 20_000_000, mk_nested(true, false), 25)],
                )],
            )),
        );

        // 3. Reordering elements inside an array produces a distinct series and drops its initial point.
        assert_step(
            &mut proc,
            t0,
            metrics_batch(
                mk_nested(false, false),
                vec![sum_metric(
                    "nested_attr_counter",
                    CUM,
                    vec![num_dp(0, 30_000_000, mk_nested(false, true), 90)],
                )],
            ),
            (1, 0, 0),
            None,
        );
    }

    /// Scenario: Two resources and multiple series observed initially, then only one series remains active across two mark-and-sweep GC intervals.
    /// Guarantees: Inactive resources and inactive per-metric series under active resources are evicted after two GC cycles while active series retain their baseline.
    #[test]
    fn test_mark_and_sweep_gc_evicts_stale_series_and_resources() {
        let mut proc = test_processor(Duration::from_secs(60));
        let t0 = Instant::now();
        let make = |svc: &str, st: u64, ts: u64, attrs: Vec<KeyValue>, val: i64| {
            metrics_batch(
                vec![kv_str("service.name", svc)],
                vec![sum_metric(
                    "requests_total",
                    CUM,
                    vec![num_dp(st, ts, attrs, val)],
                )],
            )
        };

        assert_step(
            &mut proc,
            t0,
            make("svc-a", 0, 10_000_000, vec![], 10),
            (1, 0, 0),
            None,
        );
        assert_step(
            &mut proc,
            t0,
            make("svc-b", 0, 10_000_000, vec![], 20),
            (1, 0, 0),
            None,
        );
        assert_eq!(proc.engine.cache.resource_count(), 2);

        let t1 = t0 + Duration::from_secs(61);
        assert_step(
            &mut proc,
            t1,
            make("svc-a", 0, 20_000_000, vec![], 15),
            (0, 1, 0),
            Some(make("svc-a", 10_000_000, 20_000_000, vec![], 5)),
        );
        assert_eq!(proc.engine.cache.resource_count(), 2);

        let _ = run_batch(
            &mut proc,
            make("svc-a", 0, 30_000_000, vec![], 25),
            t1 + Duration::from_secs(30),
        );
        let t3 = t1 + Duration::from_secs(61);
        assert_step(
            &mut proc,
            t3,
            make("svc-a", 0, 40_000_000, vec![], 30),
            (0, 1, 0),
            Some(make("svc-a", 10_000_000, 40_000_000, vec![], 20)),
        );
        assert_eq!(proc.engine.cache.resource_count(), 1);
        assert_step(
            &mut proc,
            t3,
            make("svc-b", 0, 50_000_000, vec![], 100),
            (1, 0, 0),
            None,
        );

        let post = vec![kv_str("method", "POST")];
        assert_step(
            &mut proc,
            t3,
            make("svc-a", 0, 50_000_000, post.clone(), 50),
            (1, 0, 0),
            None,
        );
        let t4 = t3 + Duration::from_secs(61);
        let _ = run_batch(&mut proc, make("svc-a", 0, 60_000_000, vec![], 35), t4);
        let _ = run_batch(
            &mut proc,
            make("svc-a", 0, 70_000_000, vec![], 40),
            t4 + Duration::from_secs(30),
        );
        let t6 = t4 + Duration::from_secs(61);
        let _ = run_batch(&mut proc, make("svc-a", 0, 80_000_000, vec![], 45), t6);
        assert_step(
            &mut proc,
            t6,
            make("svc-a", 0, 90_000_000, post, 99),
            (1, 0, 0),
            None,
        );
    }

    /// Scenario: Running MetricStartTimeProcessor inside the pipeline TestRuntime across initial drop, adjusted follow-up, reset, raw non-metric passthrough, and telemetry collection.
    /// Guarantees: Fully dropped initial batches emit DeliverAck with zero items, subsequent adjusted and reset batches carry expected payloads, non-metric OTLP bytes pass through without materialization, and CollectTelemetry reports dropped/adjusted/reset counters with signal="metrics".
    #[test]
    fn test_pipeline_runtime_drops_initial_and_emits_subsequent_batch() {
        use bytes::Bytes;
        use otel_arrow_dfe_engine::Interests;
        use otel_arrow_dfe_engine::control::{
            PipelineCompletionMsg, pipeline_completion_msg_channel,
        };
        use otel_arrow_dfe_otap::testing::TestCallData;
        use otel_arrow_dfe_pdata::OtlpProtoBytes;

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let metrics_reporter = test_runtime.metrics_reporter();
        let telemetry_registry = test_runtime.metrics_registry();
        let pipeline_ctx = ControllerContext::new(telemetry_registry.clone())
            .pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = create_metric_start_time_processor(
            pipeline_ctx,
            test_node(test_runtime.config().name.clone()),
            Arc::new(NodeUserConfig::with_user_config(
                METRIC_START_TIME_PROCESSOR_URN.into(),
                json!({ "strategy": "subtract_initial_point", "gc_interval": "10m" }),
            )),
            test_runtime.config(),
        )
        .unwrap();

        test_runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(4);
                ctx.set_pipeline_completion_sender(completion_tx);

                let sum_pdata = |ts: u64, val: i64| {
                    let md = one_batch(vec![sum_metric(
                        "counter_total",
                        CUM,
                        vec![num_dp(0, ts, vec![], val)],
                    )]);
                    OtapPdata::new_default(otlp_to_otap(&OtlpProtoMessage::Metrics(md)).into())
                };
                let decode = |pdata: OtapPdata| -> MetricsData {
                    let (_, payload) = pdata.into_parts();
                    let OtlpProtoMessage::Metrics(out) = otap_to_otlp(payload.otap_ref().unwrap())
                    else {
                        panic!("expected metrics");
                    };
                    out
                };

                // 1. Initial drop emits DeliverAck with 0 items.
                let init = sum_pdata(10_000_000, 40).test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::new_with(0, 0).into(),
                    7,
                );
                ctx.process(Message::PData(init)).await.unwrap();
                assert!(ctx.drain_pdata().await.is_empty());
                match completion_rx.recv().await.unwrap() {
                    PipelineCompletionMsg::DeliverAck { mut ack } => {
                        assert_eq!(ack.accepted.num_items(), 0)
                    }
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        panic!("unexpected nack: {}", nack.reason)
                    }
                }

                // 2. Adjusted follow-up and reset payloads.
                for (ts, val, exp_st, exp_val) in [
                    (20_000_000, 55, 10_000_000, 15),
                    (30_000_000, 12, 29_000_000, 12),
                ] {
                    ctx.process(Message::PData(sum_pdata(ts, val)))
                        .await
                        .unwrap();
                    let mut out = ctx.drain_pdata().await;
                    assert_eq!(out.len(), 1);
                    assert_metrics_eq(
                        Some(decode(out.remove(0))),
                        one_batch(vec![sum_metric(
                            "counter_total",
                            CUM,
                            vec![num_dp(exp_st, ts, vec![], exp_val)],
                        )]),
                    );
                }

                // 3. Raw OTLP-bytes and OTAP Logs pass through untouched without OTAP materialization.
                let raw_bytes = Bytes::from_static(b"unparsed-otlp-log-bytes");
                ctx.process(Message::PData(OtapPdata::new_default(
                    OtlpProtoBytes::ExportLogsRequest(raw_bytes.clone()).into(),
                )))
                .await
                .unwrap();
                let mut emitted_raw = ctx.drain_pdata().await;
                assert_eq!(emitted_raw.len(), 1);
                let (_, log_payload) = emitted_raw.remove(0).into_parts();
                assert!(log_payload.otap_ref().is_none());
                assert_eq!(log_payload.encoded_bytes(), Some(&raw_bytes));

                let logs = LogsData::new(vec![ResourceLogs::new(
                    Resource::default(),
                    vec![ScopeLogs::new(
                        InstrumentationScope::default(),
                        vec![LogRecord::build().time_unix_nano(1_000u64).finish()],
                    )],
                )]);
                ctx.process(Message::PData(OtapPdata::new_default(
                    otlp_to_otap(&OtlpProtoMessage::Logs(logs)).into(),
                )))
                .await
                .unwrap();
                let emitted_otap = ctx.drain_pdata().await;
                assert_eq!(
                    (emitted_otap.len(), emitted_otap[0].signal_type()),
                    (1, SignalType::Logs)
                );

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .unwrap();
            })
            .validate(move |_ctx| async move {
                telemetry_registry
                    .flush_pending_metrics()
                    .await
                    .expect("flush metric_start_time metrics");
                let (mut dropped, mut adjusted, mut resets, mut saw_signal) =
                    (0u64, 0u64, 0u64, false);
                telemetry_registry.visit_current_metrics_with_item_attrs(
                    |desc, _, item_attrs, iter| {
                        if desc.name != "processor.metric_start_time" {
                            return;
                        }
                        saw_signal |= item_attrs
                            .iter()
                            .any(|(k, v)| *k == "signal" && v.eq_ignore_ascii_case("metrics"));
                        for (f, v) in iter {
                            match f.name {
                                "dropped.items" => dropped = v.to_u64_lossy(),
                                "adjusted.data_points" => adjusted = v.to_u64_lossy(),
                                "resets.detected" => resets = v.to_u64_lossy(),
                                _ => {}
                            }
                        }
                    },
                    false,
                );
                assert!(saw_signal);
                assert_eq!((dropped, adjusted, resets), (1, 2, 1));
            });
    }

    /// Scenario: Cumulative Sum and Histogram data points arrive with start_time_unix_nano equal to time_unix_nano.
    /// Guarantees: Equal start and observation timestamps are treated as unset, dropping the first point and adjusting subsequent points against the initial baseline.
    #[test]
    fn test_start_time_equal_to_time_is_adjusted() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let make = |st: u64, ts: u64, v: i64, hc: u64, hs: f64, hb: Vec<u64>| {
            one_batch(vec![
                sum_metric("eq_sum", CUM, vec![num_dp(st, ts, vec![], v)]),
                hist_metric(
                    "eq_hist",
                    CUM,
                    vec![hist_dp(st, ts, vec![], hc, hs, vec![1.0], hb)],
                ),
            ])
        };
        assert_step(
            &mut proc,
            t0,
            make(10_000_000, 10_000_000, 25, 4, 12.0, vec![1, 3]),
            (2, 0, 0),
            None,
        );
        assert_step(
            &mut proc,
            t0,
            make(20_000_000, 20_000_000, 40, 7, 20.0, vec![2, 5]),
            (0, 2, 0),
            Some(make(10_000_000, 20_000_000, 15, 3, 8.0, vec![1, 2])),
        );
    }

    /// Scenario: Sum edge cases including integer reset with negative values, mixed int/double rows, int-to-double switches, NaN observations, sub-1ms reset timestamps, and non-monotonic cumulative sums.
    /// Guarantees: Integer resets zero their baseline, value-type switches reset cleanly without bit corruption, NaN does not poison state or mask resets, reset start times clamp at 0, and non-monotonic sums pass through untouched.
    #[test]
    fn test_sum_int_reset_mixed_types_value_switch_nan_and_non_monotonic() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::number_data_point::Value;

        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let tag = |v: &str| vec![kv_str("kind", v)];
        let sum_b = |pts| one_batch(vec![sum_metric("mixed_sum", CUM, pts)]);

        // Non-monotonic cumulative Sum passes through untouched.
        let non_mono = one_batch(vec![
            Metric::build()
                .name("active_connections")
                .data_sum(Sum::new(
                    CUM,
                    false,
                    vec![num_dp(0, 10_000_000, vec![], 100)],
                ))
                .finish(),
        ]);
        assert_step(&mut proc, t0, non_mono.clone(), (0, 0, 0), Some(non_mono));

        // Seed int (negative), double, type-switch, NaN-mid, and initial-NaN series.
        assert_step(
            &mut proc,
            t0,
            sum_b(vec![
                num_dp(0, 10_000_000, tag("int"), -10),
                num_dp_f64(0, 10_000_000, tag("dbl"), 20.0),
                num_dp(0, 10_000_000, tag("sw"), 100),
                num_dp_f64(0, 10_000_000, tag("nan"), 100.0),
                num_dp_f64(0, 10_000_000, tag("init_nan"), f64::NAN),
            ]),
            (5, 0, 0),
            None,
        );

        // Scrape 2: int rises (-10 -> -3 => 7), dbl rises (20 -> 35 => 15), int->double resets, mid-NaN passes through, initial-NaN -> finite resets.
        let (out2, s2) = run_batch(
            &mut proc,
            sum_b(vec![
                num_dp(0, 20_000_000, tag("int"), -3),
                num_dp_f64(0, 20_000_000, tag("dbl"), 35.0),
                num_dp_f64(0, 20_000_000, tag("sw"), 120.0),
                num_dp_f64(0, 20_000_000, tag("nan"), f64::NAN),
                num_dp_f64(0, 20_000_000, tag("init_nan"), 42.0),
            ]),
            t0,
        );
        assert_eq!((s2.adjusted_points, s2.resets_detected), (5, 2));
        let out2_val = out2.unwrap();
        let Data::Sum(sum2) = out2_val.resource_metrics[0].scope_metrics[0].metrics[0]
            .data
            .as_ref()
            .unwrap()
        else {
            panic!("expected Sum");
        };
        for (idx, exp_st, exp_val) in [
            (0, 10_000_000, Value::AsInt(7)),
            (1, 10_000_000, Value::AsDouble(15.0)),
            (2, 19_000_000, Value::AsDouble(120.0)),
            (4, 19_000_000, Value::AsDouble(42.0)),
        ] {
            let dp = &sum2.data_points[idx];
            assert_eq!((dp.start_time_unix_nano, dp.value), (exp_st, Some(exp_val)));
        }
        assert_eq!(sum2.data_points[3].start_time_unix_nano, 10_000_000);
        assert!(matches!(sum2.data_points[3].value, Some(Value::AsDouble(v)) if v.is_nan()));

        // Scrape 3: sub-1ms reset clamps to 0, double->int resets, mid-NaN did not mask reset to 5.0.
        assert_step(
            &mut proc,
            t0,
            sum_b(vec![
                num_dp(0, 500_000, tag("int"), -20),
                num_dp(0, 30_000_000, tag("sw"), 10),
                num_dp_f64(0, 30_000_000, tag("nan"), 5.0),
            ]),
            (0, 3, 3),
            Some(sum_b(vec![
                num_dp(0, 500_000, tag("int"), -20),
                num_dp(29_000_000, 30_000_000, tag("sw"), 10),
                num_dp_f64(29_000_000, 30_000_000, tag("nan"), 5.0),
            ])),
        );
        // Scrape 4: post-reset int series retains zeroed baseline.
        assert_step(
            &mut proc,
            t0,
            sum_b(vec![num_dp(0, 40_000_000, tag("int"), -12)]),
            (0, 1, 0),
            Some(sum_b(vec![num_dp(0, 40_000_000, tag("int"), -12)])),
        );
    }

    /// Scenario: A single batch contains multiple points for the same series followed by trailing passthrough Gauge/Delta and preset rows with null start_time.
    /// Guarantees: Same-batch initial points are dropped while subsequent same-batch points adjust/reset cleanly, and trailing passthrough rows preserve original values and nullability.
    #[test]
    fn test_trailing_passthrough_and_same_batch_multi_point_series() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let tail = vec![
            Metric::build()
                .name("trailing_gauge")
                .data_gauge(Gauge::new(vec![num_dp(0, 30_000_000, vec![], 9)]))
                .finish(),
            sum_metric(
                "trailing_delta",
                DELTA,
                vec![num_dp(0, 30_000_000, vec![], 3)],
            ),
        ];
        let x = || vec![kv_str("s", "x")];
        let preset = num_dp(2_000_000, 30_000_000, vec![kv_str("s", "preset")], 77);

        let mut input = vec![sum_metric(
            "same_batch_sum",
            CUM,
            vec![
                num_dp(0, 10_000_000, x(), 100),
                num_dp(0, 20_000_000, x(), 130),
                num_dp(0, 30_000_000, x(), 5),
                preset.clone(),
            ],
        )];
        input.extend(tail.clone());

        let mut expected = vec![sum_metric(
            "same_batch_sum",
            CUM,
            vec![
                num_dp(10_000_000, 20_000_000, x(), 30),
                num_dp(29_000_000, 30_000_000, x(), 5),
                preset,
            ],
        )];
        expected.extend(tail);
        assert_step(
            &mut proc,
            t0,
            one_batch(input),
            (1, 2, 1),
            Some(one_batch(expected)),
        );
    }

    /// Scenario: Histogram and Summary edge resets from explicit_bounds changes, bucket length changes, sum-only decreases, and omitted optional sum.
    /// Guarantees: Bounds changes, bucket-length changes, and sum-only decreases reset accurately without panicking on subsequent scrapes, while omitting optional sum never triggers a spurious reset.
    #[test]
    fn test_histogram_and_summary_edge_resets_and_optional_sum() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let h = |st, ts, c, s, bnd: &[f64], bkt: &[u64]| {
            hist_metric(
                "h",
                CUM,
                vec![hist_dp(st, ts, vec![], c, s, bnd.to_vec(), bkt.to_vec())],
            )
        };
        let sm = |st, ts, c, s| summary_metric("s", vec![summary_dp(st, ts, vec![], c, s, vec![])]);

        // 1. Seed Histogram and Summary.
        assert_step(
            &mut proc,
            t0,
            one_batch(vec![
                h(0, 10_000_000, 10, 50.0, &[1.0, 5.0], &[2, 5, 3]),
                sm(0, 10_000_000, 10, 50.0),
            ]),
            (2, 0, 0),
            None,
        );

        // 2. Omitted optional sum on scrape 2 does not trigger a reset.
        let mk_no_sum = |st: u64, c: u64, bkt: &[u64]| {
            one_batch(vec![hist_metric(
                "h",
                CUM,
                vec![
                    HistogramDataPoint::build()
                        .start_time_unix_nano(st)
                        .time_unix_nano(20_000_000u64)
                        .count(c)
                        .explicit_bounds(vec![1.0, 5.0])
                        .bucket_counts(bkt.to_vec())
                        .finish(),
                ],
            )])
        };
        assert_step(
            &mut proc,
            t0,
            mk_no_sum(0, 12, &[3, 6, 3]),
            (0, 1, 0),
            Some(mk_no_sum(10_000_000, 2, &[1, 1, 0])),
        );

        // 3. Bounds change (same len) and Summary sum-only decrease both reset.
        let b3 = |st| {
            one_batch(vec![
                h(st, 30_000_000, 14, 70.0, &[1.0, 10.0], &[4, 7, 3]),
                sm(st, 30_000_000, 12, 40.0),
            ])
        };
        assert_step(&mut proc, t0, b3(0), (0, 2, 2), Some(b3(29_000_000)));

        // 4. Bucket-count length change resets and reallocates; follow-up scrape succeeds cleanly.
        let bnd3 = [1.0, 5.0, 10.0];
        for &(ts, c, s, bkt, stats) in &[
            (40_000_000, 16, 80.0, &[4, 5, 4, 3][..], (0, 1, 1)),
            (50_000_000, 20, 95.0, &[5, 6, 5, 4][..], (0, 1, 0)),
        ] {
            assert_step(
                &mut proc,
                t0,
                one_batch(vec![h(0, ts, c, s, &bnd3, bkt)]),
                stats,
                Some(one_batch(vec![h(39_000_000, ts, c, s, &bnd3, bkt)])),
            );
        }
    }

    /// Scenario: Histogram and ExponentialHistogram series start with an absent or NaN sum (with non-zero count), alongside zero-count initial, known-sum, passthrough, and zero-sum Summary series, followed by adjustments and a reset.
    /// Guarantees: Unknown initial sums cause adjusted optional sums to be omitted (rather than subtracting zero) until a reset establishes a zero baseline, while passthrough, known-baseline, zero-count-baseline, and Summary zero-sum series retain exact sums.
    #[test]
    fn test_unknown_initial_histogram_and_exp_histogram_sum_omitted_until_reset() {
        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let tag = |v: &str| vec![kv_str("s", v)];

        let h_opt = |st: u64, ts: u64, attrs, c: u64, sum: Option<f64>, bkt: &[u64]| {
            let mut b = HistogramDataPoint::build()
                .start_time_unix_nano(st)
                .time_unix_nano(ts)
                .attributes(attrs)
                .count(c)
                .explicit_bounds(vec![1.0])
                .bucket_counts(bkt.to_vec());
            if let Some(v) = sum {
                b = b.sum(v);
            }
            b.finish()
        };
        let eh_opt = |st: u64, ts: u64, attrs, c: u64, sum: Option<f64>, pb: u64| {
            let mut b = ExponentialHistogramDataPoint::build()
                .start_time_unix_nano(st)
                .time_unix_nano(ts)
                .attributes(attrs)
                .scale(1)
                .count(c)
                .zero_count(0u64)
                .positive(Buckets::new(0, vec![pb]))
                .negative(Buckets::new(0, vec![]));
            if let Some(v) = sum {
                b = b.sum(v);
            }
            b.finish()
        };

        // 1. Seed: absent sum with count=10, NaN sum with count=10, zero-count absent sum, known sum, and Summary with sum=0.0.
        assert_step(
            &mut proc,
            t0,
            one_batch(vec![
                hist_metric(
                    "h_unk",
                    CUM,
                    vec![
                        h_opt(0, 10_000_000, tag("no_sum"), 10, None, &[4, 6]),
                        h_opt(0, 10_000_000, tag("nan_sum"), 10, Some(f64::NAN), &[4, 6]),
                        h_opt(0, 10_000_000, tag("zero_cnt"), 0, None, &[0, 0]),
                        h_opt(0, 10_000_000, tag("known"), 10, Some(50.0), &[4, 6]),
                    ],
                ),
                exp_hist_metric(
                    "eh_unk",
                    CUM,
                    vec![
                        eh_opt(0, 10_000_000, tag("no_sum"), 10, None, 10),
                        eh_opt(0, 10_000_000, tag("nan_sum"), 10, Some(f64::NAN), 10),
                    ],
                ),
                summary_metric(
                    "sm_zero",
                    vec![summary_dp(0, 10_000_000, vec![], 10, 0.0, vec![])],
                ),
            ]),
            (7, 0, 0),
            None,
        );

        // 2. Subsequent batch with finite sums + a passthrough preset row + a dropped new series
        //    in the same batch (exercising keep_filter alongside clear_mask): no_sum and nan_sum
        //    omit sum; zero_cnt, known, passthrough, and sm_zero keep exact sums.
        assert_step(
            &mut proc,
            t0,
            one_batch(vec![
                hist_metric(
                    "h_unk",
                    CUM,
                    vec![
                        h_opt(1_000_000, 20_000_000, tag("preset"), 5, Some(25.0), &[2, 3]),
                        h_opt(0, 20_000_000, tag("new_drop"), 3, Some(9.0), &[1, 2]),
                        h_opt(0, 20_000_000, tag("no_sum"), 12, Some(60.0), &[5, 7]),
                        h_opt(0, 20_000_000, tag("nan_sum"), 13, Some(70.0), &[6, 7]),
                        h_opt(0, 20_000_000, tag("zero_cnt"), 2, Some(11.0), &[1, 1]),
                        h_opt(0, 20_000_000, tag("known"), 12, Some(62.0), &[5, 7]),
                    ],
                ),
                exp_hist_metric(
                    "eh_unk",
                    CUM,
                    vec![
                        eh_opt(0, 20_000_000, tag("no_sum"), 12, Some(60.0), 12),
                        eh_opt(0, 20_000_000, tag("nan_sum"), 13, Some(70.0), 13),
                    ],
                ),
                summary_metric(
                    "sm_zero",
                    vec![summary_dp(0, 20_000_000, vec![], 12, 5.0, vec![])],
                ),
            ]),
            (1, 7, 0),
            Some(one_batch(vec![
                hist_metric(
                    "h_unk",
                    CUM,
                    vec![
                        h_opt(1_000_000, 20_000_000, tag("preset"), 5, Some(25.0), &[2, 3]),
                        h_opt(10_000_000, 20_000_000, tag("no_sum"), 2, None, &[1, 1]),
                        h_opt(10_000_000, 20_000_000, tag("nan_sum"), 3, None, &[2, 1]),
                        h_opt(
                            10_000_000,
                            20_000_000,
                            tag("zero_cnt"),
                            2,
                            Some(11.0),
                            &[1, 1],
                        ),
                        h_opt(10_000_000, 20_000_000, tag("known"), 2, Some(12.0), &[1, 1]),
                    ],
                ),
                exp_hist_metric(
                    "eh_unk",
                    CUM,
                    vec![
                        eh_opt(10_000_000, 20_000_000, tag("no_sum"), 2, None, 2),
                        eh_opt(10_000_000, 20_000_000, tag("nan_sum"), 3, None, 3),
                    ],
                ),
                summary_metric(
                    "sm_zero",
                    vec![summary_dp(10_000_000, 20_000_000, vec![], 2, 5.0, vec![])],
                ),
            ])),
        );

        // 3. Reset establishes a zero baseline, so both the reset row and follow-up row emit sum.
        for &(ts, est, c, sum, bkt, stats) in &[
            (30_000_000, 29_000_000, 3, 15.0, &[1, 2][..], (0, 4, 4)),
            (40_000_000, 29_000_000, 5, 27.0, &[2, 3][..], (0, 4, 0)),
        ] {
            assert_step(
                &mut proc,
                t0,
                one_batch(vec![
                    hist_metric(
                        "h_unk",
                        CUM,
                        vec![
                            h_opt(0, ts, tag("no_sum"), c, Some(sum), bkt),
                            h_opt(0, ts, tag("nan_sum"), c, Some(sum), bkt),
                        ],
                    ),
                    exp_hist_metric(
                        "eh_unk",
                        CUM,
                        vec![
                            eh_opt(0, ts, tag("no_sum"), c, Some(sum), c),
                            eh_opt(0, ts, tag("nan_sum"), c, Some(sum), c),
                        ],
                    ),
                ]),
                stats,
                Some(one_batch(vec![
                    hist_metric(
                        "h_unk",
                        CUM,
                        vec![
                            h_opt(est, ts, tag("no_sum"), c, Some(sum), bkt),
                            h_opt(est, ts, tag("nan_sum"), c, Some(sum), bkt),
                        ],
                    ),
                    exp_hist_metric(
                        "eh_unk",
                        CUM,
                        vec![
                            eh_opt(est, ts, tag("no_sum"), c, Some(sum), c),
                            eh_opt(est, ts, tag("nan_sum"), c, Some(sum), c),
                        ],
                    ),
                ])),
            );
        }
    }

    /// Scenario: ExponentialHistogram exercises scale changes (including > i16::MAX), zero_count decrease, negative bucket offset shift + growth across 3 scrapes, negative decrease reset, and consecutive same-length resets.
    /// Guarantees: Packed bucket state stays aligned across reallocations, scales outside i16 do not cause false resets, and every reset cause produces exact start times and bucket counts.
    #[test]
    fn test_exp_histogram_scale_zero_count_negative_buckets_and_reallocation() {
        struct Eh(i32, u64, f64, u64, i32, &'static [u64], i32, &'static [u64]);

        let mut proc = test_processor(Duration::from_secs(600));
        let t0 = Instant::now();
        let mk = |st, ts, d: &Eh| {
            one_batch(vec![exp_hist_metric(
                "eh_edge",
                CUM,
                vec![exp_hist_dp(
                    st,
                    ts,
                    vec![],
                    d.0,
                    d.1,
                    d.2,
                    d.3,
                    Buckets::new(d.4, d.5.to_vec()),
                    Buckets::new(d.6, d.7.to_vec()),
                )],
            )])
        };

        // 1. Seed with scale = 70_000 (> i16::MAX) and non-empty pos/neg buckets.
        assert_step(
            &mut proc,
            t0,
            mk(
                0,
                10_000_000,
                &Eh(70_000, 10, 50.0, 2, 1, &[2, 3], 1, &[1, 2]),
            ),
            (1, 0, 0),
            None,
        );

        // 2-3: Growth + offset shift (triggers reallocation) and subsequent post-reallocation read.
        for &(ts, ref inp, ref exp) in &[
            (
                20_000_000,
                Eh(70_000, 16, 80.0, 3, 0, &[1, 3, 4], 0, &[2, 2, 3]),
                Eh(70_000, 6, 30.0, 1, 0, &[1, 1, 1], 0, &[2, 1, 1]),
            ),
            (
                30_000_000,
                Eh(70_000, 20, 100.0, 4, 0, &[2, 4, 5], 0, &[3, 3, 4]),
                Eh(70_000, 10, 50.0, 2, 0, &[2, 2, 2], 0, &[3, 2, 2]),
            ),
        ] {
            assert_step(
                &mut proc,
                t0,
                mk(0, ts, inp),
                (0, 1, 0),
                Some(mk(10_000_000, ts, exp)),
            );
        }

        // 4-6: Negative bucket decrease reset, consecutive zero_count reset (in-place), and negative scale reset.
        for &(ts, est, ref d) in &[
            (
                40_000_000,
                39_000_000,
                Eh(70_000, 22, 110.0, 4, 0, &[2, 4, 5], 0, &[3, 2, 4]),
            ),
            (
                50_000_000,
                49_000_000,
                Eh(70_000, 23, 115.0, 1, 0, &[2, 4, 5], 0, &[3, 2, 4]),
            ),
            (
                60_000_000,
                59_000_000,
                Eh(-2, 25, 125.0, 2, 0, &[3, 5, 6], 0, &[4, 3, 5]),
            ),
        ] {
            assert_step(&mut proc, t0, mk(0, ts, d), (0, 1, 1), Some(mk(est, ts, d)));
        }
    }

    /// Scenario: OutputSchemaCache processes schemas without START_TIME_UNIX_NANO, non-nullable START_TIME_UNIX_NANO, already-nullable UTC START_TIME_UNIX_NANO, repeated hits, and schema transitions; plus early-return batches and stale-gap cache clearing.
    /// Guarantees: Output schemas insert/upgrade START_TIME_UNIX_NANO properly, reuse Arc<Schema> on cache hits, empty/ineligible batches trigger GC, and idle gaps > 2*gc_interval clear stale state before processing.
    #[test]
    fn test_output_schema_cache_early_return_and_gc_stale_clear() {
        let mut cache = OutputSchemaCache::default();
        let utc_dt =
            DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, Some("UTC".into()));
        let pid_f = Field::new(consts::PARENT_ID, DataType::UInt16, false);

        let s_missing = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, false),
            pid_f.clone(),
            Field::new(consts::INT_VALUE, DataType::Int64, true),
        ]));
        let out_missing = cache.get_or_build(0, &s_missing, &utc_dt);
        assert_eq!(
            (
                out_missing.field(2).name().as_str(),
                out_missing.field(2).is_nullable(),
                out_missing.field(2).data_type()
            ),
            (consts::START_TIME_UNIX_NANO, true, &utc_dt)
        );
        assert!(Arc::ptr_eq(
            &out_missing,
            &cache.get_or_build(0, &s_missing, &utc_dt)
        ));

        let s_non_null = Arc::new(Schema::new(vec![
            pid_f.clone(),
            Field::new(
                consts::START_TIME_UNIX_NANO,
                DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, None),
                false,
            ),
        ]));
        let out_upgraded = cache.get_or_build(0, &s_non_null, &utc_dt);
        assert_eq!(
            (
                out_upgraded.field(1).is_nullable(),
                out_upgraded.field(1).data_type()
            ),
            (true, &utc_dt)
        );

        let s_ready = Arc::new(Schema::new(vec![
            pid_f,
            Field::new(consts::START_TIME_UNIX_NANO, utc_dt.clone(), true),
        ]));
        assert!(Arc::ptr_eq(
            &s_ready,
            &cache.get_or_build(0, &s_ready, &utc_dt)
        ));

        let mut proc = test_processor(Duration::from_secs(60));
        let t0 = Instant::now();
        let idle_b = |ts, v| {
            one_batch(vec![sum_metric(
                "idle_total",
                CUM,
                vec![num_dp(0, ts, vec![], v)],
            )])
        };
        assert_step(&mut proc, t0, idle_b(10_000_000, 50), (1, 0, 0), None);
        assert_eq!(proc.engine.cache.resource_count(), 1);

        let mut empty = OtapArrowRecords::Metrics(Metrics::default());
        let s_empty = proc
            .engine
            .process_metrics_batch(&mut empty, t0 + Duration::from_secs(10))
            .unwrap();
        assert_eq!(
            (
                s_empty.dropped_initial,
                s_empty.adjusted_points,
                proc.engine.cache.resource_count()
            ),
            (0, 0, 1)
        );
        assert_step(
            &mut proc,
            t0 + Duration::from_secs(121),
            idle_b(200_000_000, 75),
            (1, 0, 0),
            None,
        );
    }

    /// Scenario: Invoking METRIC_START_TIME_PROCESSOR_FACTORY.validate_config and MetricStartTimeProcessor::from_config across valid and invalid configs.
    /// Guarantees: Both static validation and runtime construction accept valid configs and reject unknown fields or zero gc_interval with InvalidUserConfig.
    #[test]
    fn test_factory_validate_config_and_from_config_errors() {
        let validate = METRIC_START_TIME_PROCESSOR_FACTORY.validate_config;
        assert!(validate(&json!({})).is_ok());
        assert!(
            validate(&json!({ "strategy": "subtract_initial_point", "gc_interval": "2m" })).is_ok()
        );
        assert!(matches!(
            validate(&json!({ "gc_interval": "0s" })),
            Err(ConfigError::InvalidUserConfig { .. })
        ));
        assert!(validate(&json!({ "unexpected": 1 })).is_err());

        let pipeline_ctx = ControllerContext::new(TelemetryRegistryHandle::new())
            .pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        assert!(matches!(
            MetricStartTimeProcessor::from_config(
                pipeline_ctx.clone(),
                &json!({ "gc_interval": "0s" })
            ),
            Err(ConfigError::InvalidUserConfig { .. })
        ));
        assert!(MetricStartTimeProcessor::from_config(pipeline_ctx, &json!({ "bad": 1 })).is_err());
    }
}
