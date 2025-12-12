// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opentelemetry Metrics Views configuration.

use opentelemetry_sdk::metrics::{Instrument, MeterProviderBuilder, Stream};
use otap_df_config::pipeline::service::telemetry::metrics::views::ViewConfig;

use crate::error::Error;

/// Provider for configuring OpenTelemetry Metrics Views.
pub struct ViewsProvider {}

impl ViewsProvider {
    /// Configure the meter provider builder with the given views configuration.
    pub fn configure(
        mut sdk_meter_builder: MeterProviderBuilder,
        views_config: Vec<ViewConfig>,
    ) -> Result<MeterProviderBuilder, Error> {
        for view_config in views_config {
            let view = DeclarativeView::new(view_config);
            sdk_meter_builder = sdk_meter_builder.with_view(view.to_view_funtion());
        }
        Ok(sdk_meter_builder)
    }
}

/// A declarative metrics view based on a configuration.
pub struct DeclarativeView {
    config: ViewConfig,
}

impl DeclarativeView {
    /// Create a new declarative view from the given configuration.
    #[must_use]
    pub fn new(config: ViewConfig) -> Self {
        Self { config }
    }

    /// Convert the declarative view into a function that can be used to configure the SDK.
    pub fn to_view_funtion(
        &self,
    ) -> impl Fn(&Instrument) -> Option<Stream> + Send + Sync + 'static {
        let config = self.config.clone();
        move |instrument: &Instrument| {
            if let Some(instrument_name) = &config.selector.instrument_name
                && !instrument.name().contains(instrument_name)
            {
                return None;
            }

            let mut stream_builder = Stream::builder();
            if let Some(name) = &config.stream.name {
                stream_builder = stream_builder.with_name(name.clone());
            }
            if let Some(description) = &config.stream.description {
                stream_builder = stream_builder.with_description(description.clone());
            }

            stream_builder.build().ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use otap_df_config::pipeline::service::telemetry::metrics::views::{
        ViewConfig, ViewSelector, ViewStream,
    };

    #[test]
    fn test_views_provider_configure() {
        let view_config = ViewConfig {
            selector: ViewSelector {
                instrument_name: Some("requests.total".to_string()),
            },
            stream: ViewStream {
                name: Some("http.requests.total".to_string()),
                description: Some("Total number of HTTP requests".to_string()),
            },
        };
        let views_config = vec![view_config];
        let sdk_meter_builder = SdkMeterProvider::builder();
        let result = ViewsProvider::configure(sdk_meter_builder, views_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_declarative_view_creation() {
        let view_config = ViewConfig {
            selector: ViewSelector {
                instrument_name: Some("requests.total".to_string()),
            },
            stream: ViewStream {
                name: Some("http.requests.total".to_string()),
                description: Some("Total number of HTTP requests".to_string()),
            },
        };
        let declarative_view = DeclarativeView::new(view_config.clone());
        assert_eq!(
            declarative_view.config.selector.instrument_name,
            view_config.selector.instrument_name
        );
        assert_eq!(declarative_view.config.stream.name, view_config.stream.name);
        assert_eq!(
            declarative_view.config.stream.description,
            view_config.stream.description
        );
    }

    #[test]
    fn test_declarative_view_to_view_function() {
        let view_config = ViewConfig {
            selector: ViewSelector {
                instrument_name: Some("requests.total".to_string()),
            },
            stream: ViewStream {
                name: Some("http.requests.total".to_string()),
                description: Some("Total number of HTTP requests".to_string()),
            },
        };
        let declarative_view = DeclarativeView::new(view_config);
        let _view_function = declarative_view.to_view_funtion();
    }
}
