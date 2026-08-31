//! Internal logs receiver state

use bytes::Bytes;
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::otlp::ProtoBuffer;
use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
use tokio::time::Instant;

/// Owns internal log input state and the scope-encoding cache.
pub(super) struct LogExportState {
    enabled: bool,
    channel_open: bool,
    scope_cache: ScopeToBytesMap,
}

impl LogExportState {
    /// Create the internal logs exporter.
    pub(super) fn new(enabled: bool, registry: TelemetryRegistryHandle) -> Self {
        Self {
            enabled,
            channel_open: enabled,
            scope_cache: ScopeToBytesMap::new(registry),
        }
    }

    /// Reports whether the internal log channel can still deliver events.
    pub(super) const fn channel_open(&self) -> bool {
        self.channel_open
    }

    /// Records that the internal log channel has closed.
    pub(super) const fn close_channel(&mut self) {
        self.channel_open = false;
    }

    /// Send a log event as OTLP logs with scope attributes from entity context.
    pub(super) async fn send_log_event(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        log_event: LogEvent,
        resource_field_bytes: &Bytes,
    ) -> Result<(), Error> {
        let mut buf = ProtoBuffer::with_capacity(512);

        encode_export_logs_request(
            &mut buf,
            &log_event,
            resource_field_bytes,
            &mut self.scope_cache,
        );

        let pdata = OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(buf.into_bytes()).into(),
        );
        effect_handler.send_message(pdata).await?;
        Ok(())
    }

    /// Drains queued logs within the terminal-control deadline.
    ///
    /// The deadline is observed between log records and while awaiting downstream
    /// capacity. Synchronous encoding of the current record runs to completion.
    pub(super) async fn flush_until(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        internal: &otel_arrow_dfe_telemetry::InternalTelemetrySettings,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        if !self.enabled {
            return Ok(());
        }

        loop {
            let Ok(event) = internal.logs_receiver.try_recv() else {
                break;
            };
            if std::time::Instant::now() >= deadline {
                return Err(Error::InternalError {
                    message: "timed out while flushing internal logs during shutdown; remaining terminal telemetry was not flushed".to_owned(),
                });
            }
            if let ObservedEvent::Log(log_event) = event {
                if let Some(log_tap) = internal.log_tap.as_ref() {
                    log_tap.record(log_event.clone());
                }
                tokio::time::timeout_at(
                    Instant::from_std(deadline),
                    self.send_log_event(
                        effect_handler,
                        log_event,
                        &internal.resource_field_bytes,
                    ),
                )
                .await
                .map_err(|_| Error::InternalError {
                    message: "timed out while flushing internal logs during shutdown; remaining terminal telemetry was not flushed".to_owned(),
                })??;
            }
        }
        Ok(())
    }
}
