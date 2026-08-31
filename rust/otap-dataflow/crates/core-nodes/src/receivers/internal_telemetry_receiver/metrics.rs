// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal metrics receiver state

use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_telemetry::metrics::otlp::{
    MetricView, MetricViewSelector, MetricViewStream, MetricsOtlpEncoder,
};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

/// Registry-backed internal metrics configuration.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    /// How frequently accumulated registry metrics are emitted.
    ///
    /// When omitted, the engine telemetry reporting interval is used.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,

    /// Views applied while projecting metric-set fields to OTLP metrics.
    #[serde(default)]
    pub views: Vec<ViewConfig>,
}

/// A supported metric view transformation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ViewConfig {
    /// Selects metric-set fields to transform.
    pub selector: ViewSelector,

    /// Overrides properties of each selected OTLP metric stream.
    pub stream: ViewStream,
}

/// Exact-match selector for a metric view.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ViewSelector {
    /// Metric-set (instrumentation scope) name to match.
    pub scope_name: Option<String>,

    /// Scalar metric-set entity attributes that must all match exactly.
    #[serde(default)]
    pub scope_attributes: HashMap<String, ConfigAttributeValue>,

    /// Metric field (instrument) name to match.
    pub instrument_name: Option<String>,
}

/// Supported output stream overrides for a metric view.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ViewStream {
    /// Replacement metric name.
    pub name: Option<String>,

    /// Replacement metric description.
    pub description: Option<String>,
}

impl MetricsConfig {
    /// Validates a MetricsConfig
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.interval.is_some_and(|interval| interval.is_zero()) {
            return Err(ConfigError::InvalidUserConfig {
                error: "internal telemetry receiver metrics interval must be greater than zero"
                    .to_owned(),
            });
        }
        if let Some((key, _)) = self.views.iter().find_map(|view| {
            view.selector
                .scope_attributes
                .iter()
                .find(|(_, value)| matches!(value, ConfigAttributeValue::Array(_)))
        }) {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver metric view scope attribute '{key}' must be a scalar value"
                ),
            });
        }
        Ok(())
    }
}

impl From<ViewConfig> for MetricView {
    fn from(view: ViewConfig) -> Self {
        Self {
            selector: MetricViewSelector {
                scope_name: view.selector.scope_name,
                scope_attributes: view.selector.scope_attributes,
                instrument_name: view.selector.instrument_name,
            },
            stream: MetricViewStream {
                name: view.stream.name,
                description: view.stream.description,
            },
        }
    }
}

/// Keeps the metric timer independently borrowable from an in-flight export.
pub(super) struct MetricExportState {
    interval: Interval,
    exporter: MetricExporter,
    pending_export: Option<PendingExport>,
}

/// A future metrics export.
type PendingExport = Pin<Box<dyn Future<Output = Result<(), Error>>>>;

/// Internal metrics exporter.
pub(super) struct MetricExporter {
    registry: TelemetryRegistryHandle,
    encoder: Option<Rc<MetricsOtlpEncoder>>,
}

impl MetricExportState {
    /// Create the internal metrics exporter.
    pub(super) fn new(
        interval: Duration,
        registry: TelemetryRegistryHandle,
        encoder: Option<MetricsOtlpEncoder>,
    ) -> Self {
        let mut interval = interval_at(Instant::now() + interval, interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self {
            interval,
            exporter: MetricExporter {
                registry,
                encoder: encoder.map(Rc::new),
            },
            pending_export: None,
        }
    }

    /// Advances the metric interval and downstream delivery by one event.
    pub(super) async fn run_once(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if let Some(export) = self.pending_export.as_mut() {
            let result = export.await;
            self.pending_export = None;
            return result;
        }

        let _ = self.interval.tick().await;
        self.pending_export = Some(self.exporter.begin_export(effect_handler));
        Ok(())
    }

    /// Cancels a periodic export so its transaction restores drained values.
    pub(super) fn cancel_pending(&mut self) {
        drop(self.pending_export.take());
    }

    /// Performs one final metric export within the terminal-control deadline.
    pub(super) async fn flush_until(
        &self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        self.exporter.flush_until(effect_handler, deadline).await
    }
}

impl MetricExporter {
    /// Starts an export future that may remain pending on downstream backpressure.
    fn begin_export(&self, effect_handler: &local::EffectHandler<OtapPdata>) -> PendingExport {
        let effect_handler = effect_handler.clone();
        let registry = self.registry.clone();
        let encoder = self.encoder.clone();
        Box::pin(async move {
            Self::process_batch(&effect_handler, &registry, encoder.as_deref()).await
        })
    }

    /// Performs one final metric export within the terminal-control deadline.
    async fn flush_until(
        &self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        tokio::time::timeout_at(
            Instant::from_std(deadline),
            Self::process_batch(effect_handler, &self.registry, self.encoder.as_deref()),
        )
        .await
        .map_err(|_| Error::InternalError {
            message: "timed out while flushing internal metrics during shutdown".to_owned(),
        })?
    }

    /// Flushes pending snapshots and consumes one registry export window.
    ///
    /// When an encoder is provided, the batch is converted to OTLP and committed
    /// only after downstream delivery. Without an encoder, the export-only
    /// accumulator is committed immediately without conversion or emission. The
    /// independent admin accumulator is unaffected in both cases.
    pub(super) async fn process_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        registry: &TelemetryRegistryHandle,
        encoder: Option<&MetricsOtlpEncoder>,
    ) -> Result<(), Error> {
        registry
            .flush_pending_metrics()
            .await
            .map_err(|error| Error::InternalError {
                message: format!("failed to flush internal metrics collector: {error}"),
            })?;
        let export = registry.begin_metric_export_batch();
        let Some(encoder) = encoder else {
            let _ = export.commit();
            return Ok(());
        };
        let Some(metrics) =
            encoder
                .encode(export.batch())
                .map_err(|error| Error::PdataConversionError {
                    error: error.to_string(),
                })?
        else {
            let _ = export.commit();
            return Ok(());
        };

        effect_handler
            .send_message(OtapPdata::new(Context::default(), metrics.into()))
            .await?;
        let _ = export.commit();
        Ok(())
    }
}
