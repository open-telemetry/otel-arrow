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
use otel_arrow_dfe_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
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
    log_event: &LogEvent,
    resource_field_bytes: &Bytes,
    scope_cache: &mut ScopeToBytesMap,
    symbol_cache: &mut SymbolCache,
) -> Result<PDataArrowRecordSet, String> {
    let mut buf = ProtoBuffer::with_capacity(512);
    encode_export_logs_request(&mut buf, log_event, resource_field_bytes, scope_cache);
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

    if let Some(stacktrace) = &log_event.record.stacktrace {
        let callsite = log_event.record.callsite();
        add_stack_tables(
            &mut tables,
            stacktrace.frames(),
            callsite.file(),
            callsite.line(),
            symbol_cache,
        )?;
    }

    PDataArrowRecordSet::new(
        EXTENDED_LOGS_FORMAT_ID,
        EXTENDED_LOGS_VERSION,
        otel_arrow_dfe_config::SignalType::Logs,
        1,
        tables,
    )
    .map_err(|error| error.to_string())
}

fn add_stack_tables(
    tables: &mut Vec<PDataArrowTable>,
    frames: &[usize],
    callsite_file: Option<&str>,
    callsite_line: Option<u32>,
    symbol_cache: &mut SymbolCache,
) -> Result<(), String> {
    if frames.is_empty() {
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
                Arc::new(UInt16Array::from(vec![0])),
                Arc::new(UInt32Array::from(vec![0])),
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
                Arc::new(UInt32Array::from(vec![0; frames.len()])),
                Arc::new(UInt16Array::from_iter_values(
                    (0..frames.len()).map(|ordinal| ordinal as u16),
                )),
                Arc::new(UInt32Array::from_iter_values(
                    (0..frames.len()).map(|location_id| location_id as u32),
                )),
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
                Arc::new(UInt32Array::from_iter_values(
                    (0..frames.len()).map(|location_id| location_id as u32),
                )),
                Arc::new(UInt64Array::from_iter_values(
                    frames.iter().map(|address| *address as u64),
                )),
            ],
        )
        .map_err(|error| error.to_string())?,
    });

    let mut symbols: Vec<_> = frames
        .iter()
        .map(|address| symbol_cache.resolve(*address).clone())
        .collect();
    if let Some(callsite) = symbols.first_mut() {
        callsite.filename = callsite_file.map(ToOwned::to_owned);
        callsite.line = callsite_line;
    }
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
                Arc::new(UInt32Array::from_iter_values(
                    (0..frames.len()).map(|location_id| location_id as u32),
                )),
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
        let records = encode(&event, &Bytes::new(), &mut scope_cache, &mut symbol_cache)
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
}
