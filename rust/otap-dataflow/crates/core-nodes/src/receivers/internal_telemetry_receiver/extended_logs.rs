// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal-log encoding for the compact extended Arrow representation.

use arrow::array::{RecordBatch, StringArray, UInt16Array, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use bytes::Bytes;
use otel_arrow_dfe_pdata::extended_logs::{
    EXTENDED_LOGS_FORMAT_ID, EXTENDED_LOGS_VERSION, LOCATIONS_TABLE_ID, LOG_STACKS_TABLE_ID,
    STACK_FRAMES_TABLE_ID, SYMBOLS_TABLE_ID,
};
use otel_arrow_dfe_pdata::otlp::ProtoBuffer;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::{
    OtapPayload, OtlpProtoBytes, PDataArrowRecordSet, PDataArrowTable, TryFromWithOptions,
};
use otel_arrow_dfe_telemetry::event::LogEvent;
use otel_arrow_dfe_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request_batch};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Arc;

const MAX_SYMBOL_CACHE_ENTRIES: usize = 16 * 1024;

#[derive(Clone, Debug, Default)]
struct ResolvedSymbol {
    function_name: Option<String>,
    filename: Option<String>,
    line: Option<u32>,
}

/// Receiver-local address-to-symbol cache.
#[derive(Default)]
pub(super) struct SymbolCache {
    symbols: HashMap<usize, ResolvedSymbol>,
}

impl SymbolCache {
    fn resolve(&mut self, address: usize) -> &ResolvedSymbol {
        if self.symbols.len() >= MAX_SYMBOL_CACHE_ENTRIES && !self.symbols.contains_key(&address) {
            self.symbols.clear();
        }
        self.symbols.entry(address).or_insert_with(|| {
            let mut resolved = ResolvedSymbol::default();
            backtrace::resolve(address as *mut c_void, |symbol| {
                if resolved.function_name.is_none() {
                    resolved.function_name = symbol.name().map(|name| name.to_string());
                    resolved.filename = symbol
                        .filename()
                        .map(|path| path.to_string_lossy().into_owned());
                    resolved.line = symbol.lineno();
                }
            });
            resolved
        })
    }
}

pub(super) fn encode(
    log_events: &[LogEvent],
    resource_field_bytes: &Bytes,
    scope_cache: &mut ScopeToBytesMap,
    symbol_cache: &mut SymbolCache,
) -> Result<PDataArrowRecordSet, String> {
    let capacity = log_events.iter().fold(512usize, |capacity, event| {
        capacity.saturating_add(event.record.body_attrs_bytes.len())
    });
    let mut buf = ProtoBuffer::with_capacity(capacity);
    let output_order =
        encode_export_logs_request_batch(&mut buf, log_events, resource_field_bytes, scope_cache);
    let standard = otel_arrow_dfe_pdata::OtapArrowRecords::try_from_with_default(
        OtapPayload::from(OtlpProtoBytes::ExportLogsRequest(buf.into_bytes())),
    )
    .map_err(|error| error.to_string())?;

    let mut tables = Vec::new();
    for payload_type in [
        ArrowPayloadType::ResourceAttrs,
        ArrowPayloadType::ScopeAttrs,
        ArrowPayloadType::Logs,
        ArrowPayloadType::LogAttrs,
    ] {
        if let Some(batch) = standard.get(payload_type) {
            tables.push(PDataArrowTable {
                table_id: payload_type as u32,
                batch: batch.clone(),
            });
        }
    }

    add_stack_tables(&mut tables, log_events, &output_order, symbol_cache)?;

    PDataArrowRecordSet::new(
        EXTENDED_LOGS_FORMAT_ID,
        EXTENDED_LOGS_VERSION,
        otel_arrow_dfe_config::SignalType::Logs,
        log_events.len(),
        tables,
    )
    .map_err(|error| error.to_string())
}

fn add_stack_tables(
    tables: &mut Vec<PDataArrowTable>,
    log_events: &[LogEvent],
    output_order: &[usize],
    symbol_cache: &mut SymbolCache,
) -> Result<(), String> {
    let mut log_ids = Vec::new();
    let mut stack_ids = Vec::new();
    let mut frame_stack_ids = Vec::new();
    let mut ordinals = Vec::new();
    let mut frame_location_ids = Vec::new();
    let mut location_ids = Vec::new();
    let mut addresses = Vec::new();
    let mut symbols = Vec::new();
    let mut stack_ids_by_trace = HashMap::new();
    let mut location_ids_by_source = HashMap::new();

    for (log_id, &input_index) in output_order.iter().enumerate() {
        let event = &log_events[input_index];
        let Some(stacktrace) = &event.record.stacktrace else {
            continue;
        };
        if stacktrace.frames().is_empty() {
            continue;
        }

        let log_id = u16::try_from(log_id).map_err(|_| "extended log batch exceeds u16 log IDs")?;
        let callsite = event.record.callsite();
        let stack_key = (
            stacktrace.frames().to_vec(),
            callsite.file(),
            callsite.line(),
        );
        let stack_id = if let Some(stack_id) = stack_ids_by_trace.get(&stack_key) {
            *stack_id
        } else {
            let stack_id = u32::try_from(stack_ids_by_trace.len())
                .map_err(|_| "extended log batch has too many stacks")?;
            let _ = stack_ids_by_trace.insert(stack_key, stack_id);

            for (ordinal, address) in stacktrace.frames().iter().copied().enumerate() {
                let ordinal =
                    u16::try_from(ordinal).map_err(|_| "extended log stack exceeds u16 frames")?;
                let source = if ordinal == 0 {
                    (callsite.file(), callsite.line())
                } else {
                    (None, None)
                };
                let location_key = (address, source.0, source.1);
                let location_id =
                    if let Some(location_id) = location_ids_by_source.get(&location_key) {
                        *location_id
                    } else {
                        let location_id = u32::try_from(addresses.len())
                            .map_err(|_| "extended log batch has too many locations")?;
                        let mut symbol = symbol_cache.resolve(address).clone();
                        if ordinal == 0 {
                            symbol.filename = source.0.map(ToOwned::to_owned);
                            symbol.line = source.1;
                        }
                        location_ids.push(location_id);
                        addresses.push(address as u64);
                        symbols.push(symbol);
                        let _ = location_ids_by_source.insert(location_key, location_id);
                        location_id
                    };

                frame_stack_ids.push(stack_id);
                ordinals.push(ordinal);
                frame_location_ids.push(location_id);
            }
            stack_id
        };
        log_ids.push(log_id);
        stack_ids.push(stack_id);
    }

    if log_ids.is_empty() {
        return Ok(());
    }

    tables.push(PDataArrowTable {
        table_id: LOG_STACKS_TABLE_ID,
        batch: RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("log_id", DataType::UInt16, false),
                Field::new("stack_id", DataType::UInt32, false),
            ])),
            vec![
                Arc::new(UInt16Array::from(log_ids)),
                Arc::new(UInt32Array::from(stack_ids)),
            ],
        )
        .map_err(|error| error.to_string())?,
    });

    tables.push(PDataArrowTable {
        table_id: STACK_FRAMES_TABLE_ID,
        batch: RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("stack_id", DataType::UInt32, false),
                Field::new("ordinal", DataType::UInt16, false),
                Field::new("location_id", DataType::UInt32, false),
            ])),
            vec![
                Arc::new(UInt32Array::from(frame_stack_ids)),
                Arc::new(UInt16Array::from(ordinals)),
                Arc::new(UInt32Array::from(frame_location_ids)),
            ],
        )
        .map_err(|error| error.to_string())?,
    });

    tables.push(PDataArrowTable {
        table_id: LOCATIONS_TABLE_ID,
        batch: RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::UInt32, false),
                Field::new("address", DataType::UInt64, false),
            ])),
            vec![
                Arc::new(UInt32Array::from(location_ids.clone())),
                Arc::new(UInt64Array::from(addresses)),
            ],
        )
        .map_err(|error| error.to_string())?,
    });

    tables.push(PDataArrowTable {
        table_id: SYMBOLS_TABLE_ID,
        batch: RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("location_id", DataType::UInt32, false),
                Field::new("function_name", DataType::Utf8, true),
                Field::new("filename", DataType::Utf8, true),
                Field::new("line", DataType::UInt32, true),
            ])),
            vec![
                Arc::new(UInt32Array::from(location_ids)),
                Arc::new(StringArray::from(
                    symbols
                        .iter()
                        .map(|symbol| symbol.function_name.as_deref())
                        .collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(
                    symbols
                        .iter()
                        .map(|symbol| symbol.filename.as_deref())
                        .collect::<Vec<_>>(),
                )),
                Arc::new(UInt32Array::from(
                    symbols.iter().map(|symbol| symbol.line).collect::<Vec<_>>(),
                )),
            ],
        )
        .map_err(|error| error.to_string())?,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::observed_state::SendPolicy;
    use otel_arrow_dfe_config::settings::telemetry::logs::{LogLevel, StackTraceConfig};
    use otel_arrow_dfe_pdata::extended_logs::ExtendedLogsView;
    use otel_arrow_dfe_telemetry::event::{ObservedEvent, ObservedEventReporter};
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_telemetry::self_tracing::LogContext;
    use otel_arrow_dfe_telemetry::tracing_init::ProviderSetup;
    use otel_arrow_dfe_telemetry::{TracingSetup, otel_info};

    /// Scenario: a caller-captured internal log is encoded as extended OTAP logs.
    /// Guarantees: canonical logs and ordered stack frames survive the ITR encoding boundary.
    #[test]
    fn encodes_captured_stack_as_extended_log_tables() {
        let (sender, receiver) = flume::bounded(1);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), sender);
        let level: LogLevel = serde_json::from_str("\"info\"").unwrap();
        let setup = TracingSetup::new(
            ProviderSetup::InternalAsync {
                reporter,
                stacktraces: StackTraceConfig {
                    enabled: true,
                    max_frames: 8,
                },
            },
            level,
            LogContext::new,
        );

        setup.with_subscriber(|| otel_info!("extended.stacktrace.test"));
        let ObservedEvent::Log(event) = receiver.try_recv().expect("captured internal log") else {
            panic!("expected log event");
        };
        let expected_callsite_file = event.record.callsite().file().map(ToOwned::to_owned);
        let expected_callsite_line = event.record.callsite().line();

        let registry = TelemetryRegistryHandle::new();
        let mut scope_cache = ScopeToBytesMap::new(registry);
        let mut symbol_cache = SymbolCache::default();
        let records = encode(
            std::slice::from_ref(&event),
            &Bytes::new(),
            &mut scope_cache,
            &mut symbol_cache,
        )
        .expect("extended logs encode");
        let view = ExtendedLogsView::try_from(&records).expect("extended logs view");

        assert_eq!(view.standard_logs().unwrap().num_items(), 1);
        let stacks = view.stacks().expect("stack tables");
        let frames = stacks.get(&0).expect("log stack");
        assert!(!frames.is_empty());
        assert!(frames.len() <= 8);
        assert_eq!(frames[0].filename, expected_callsite_file);
        assert_eq!(frames[0].line, expected_callsite_line);
        assert!(
            !frames[0]
                .function_name
                .as_deref()
                .is_some_and(|name| name.contains("otel_arrow_dfe_telemetry"))
        );
    }

    /// Scenario: two logs contain the same captured stack.
    /// Guarantees: compact extension tables share one stack and its locations.
    #[test]
    fn deduplicates_identical_stacks_and_locations() {
        let (sender, receiver) = flume::bounded(1);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), sender);
        let level: LogLevel = serde_json::from_str("\"info\"").unwrap();
        let setup = TracingSetup::new(
            ProviderSetup::InternalAsync {
                reporter,
                stacktraces: StackTraceConfig {
                    enabled: true,
                    max_frames: 8,
                },
            },
            level,
            LogContext::new,
        );

        setup.with_subscriber(|| otel_info!("extended.stacktrace.shared"));
        let ObservedEvent::Log(event) = receiver.try_recv().expect("captured internal log") else {
            panic!("expected log event");
        };
        let frame_count = event
            .record
            .stacktrace
            .as_ref()
            .expect("captured stack")
            .frames()
            .len();

        let registry = TelemetryRegistryHandle::new();
        let mut scope_cache = ScopeToBytesMap::new(registry);
        let mut symbol_cache = SymbolCache::default();
        let records = encode(
            &[event.clone(), event],
            &Bytes::new(),
            &mut scope_cache,
            &mut symbol_cache,
        )
        .expect("extended logs encode");

        let log_stacks = records.table(LOG_STACKS_TABLE_ID).expect("log stacks");
        let stack_ids = log_stacks
            .column_by_name("stack_id")
            .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
            .expect("stack IDs");
        assert_eq!(stack_ids.values(), &[0, 0]);
        assert_eq!(
            records
                .table(STACK_FRAMES_TABLE_ID)
                .expect("stack frames")
                .num_rows(),
            frame_count
        );
        assert_eq!(
            records
                .table(LOCATIONS_TABLE_ID)
                .expect("locations")
                .num_rows(),
            frame_count
        );
    }

    /// Scenario: a batch interleaves stack-bearing logs from two tracing targets.
    /// Guarantees: scope grouping preserves each stack's join to its reordered log row.
    #[test]
    fn batched_stacks_follow_scope_grouped_log_order() {
        let (sender, receiver) = flume::bounded(3);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), sender);
        let level: LogLevel = serde_json::from_str("\"info\"").unwrap();
        let setup = TracingSetup::new(
            ProviderSetup::InternalAsync {
                reporter,
                stacktraces: StackTraceConfig {
                    enabled: true,
                    max_frames: 8,
                },
            },
            level,
            LogContext::new,
        );

        setup.with_subscriber(|| {
            otel_info!(target: "z-target", "extended.stacktrace.z-first");
            otel_info!(target: "a-target", "extended.stacktrace.a");
            otel_info!(target: "z-target", "extended.stacktrace.z-second");
        });
        let events: Vec<_> = receiver
            .try_iter()
            .map(|event| match event {
                ObservedEvent::Log(event) => event,
                ObservedEvent::Engine(_) => panic!("expected log event"),
            })
            .collect();
        assert_eq!(events.len(), 3);
        let expected_lines = [
            events[1].record.callsite().line(),
            events[0].record.callsite().line(),
            events[2].record.callsite().line(),
        ];

        let registry = TelemetryRegistryHandle::new();
        let mut scope_cache = ScopeToBytesMap::new(registry);
        let mut symbol_cache = SymbolCache::default();
        let records = encode(&events, &Bytes::new(), &mut scope_cache, &mut symbol_cache)
            .expect("extended logs encode");
        let view = ExtendedLogsView::try_from(&records).expect("extended logs view");
        assert_eq!(view.standard_logs().unwrap().num_items(), 3);
        let stacks = view.stacks().expect("stack tables");

        for (log_id, expected_line) in expected_lines.into_iter().enumerate() {
            assert_eq!(
                stacks[&(log_id as u16)][0].line,
                expected_line,
                "stack must remain attached after scope grouping"
            );
        }
    }
}
