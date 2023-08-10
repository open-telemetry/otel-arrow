package main

import (
	"github.com/open-telemetry/otel-arrow/collector/connector/validationconnector"
	"github.com/open-telemetry/otel-arrow/collector/gen/exporter/fileexporter"
	"github.com/open-telemetry/otel-arrow/collector/gen/exporter/otlpexporter"
	"github.com/open-telemetry/otel-arrow/collector/gen/receiver/otlpreceiver"
	"github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor"
	"github.com/open-telemetry/otel-arrow/collector/processor/obfuscationprocessor"
	"github.com/open-telemetry/otel-arrow/collector/receiver/filereceiver"

	"github.com/lightstep/telemetry-generator/generatorreceiver"
	"github.com/open-telemetry/opentelemetry-collector-contrib/extension/basicauthextension"
	"github.com/open-telemetry/opentelemetry-collector-contrib/extension/headerssetterextension"
	"go.opentelemetry.io/collector/connector"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/exporter/loggingexporter"
	"go.opentelemetry.io/collector/exporter/otlphttpexporter"
	"go.opentelemetry.io/collector/extension"
	"go.opentelemetry.io/collector/extension/ballastextension"
	"go.opentelemetry.io/collector/extension/zpagesextension"
	"go.opentelemetry.io/collector/otelcol"
	"go.opentelemetry.io/collector/processor"
	"go.opentelemetry.io/collector/processor/batchprocessor"
	"go.opentelemetry.io/collector/processor/memorylimiterprocessor"
	"go.opentelemetry.io/collector/receiver"
)

func components() (otelcol.Factories, error) {
	var err error
	factories := otelcol.Factories{}

	factories.Extensions, err = extension.MakeFactoryMap(
		ballastextension.NewFactory(),
		zpagesextension.NewFactory(),
		headerssetterextension.NewFactory(),
		basicauthextension.NewFactory(),
	)
	if err != nil {
		return otelcol.Factories{}, err
	}

	factories.Receivers, err = receiver.MakeFactoryMap(
		otlpreceiver.NewFactory(),
		filereceiver.NewFactory(),
		generatorreceiver.NewFactory(),
	)
	if err != nil {
		return otelcol.Factories{}, err
	}

	factories.Exporters, err = exporter.MakeFactoryMap(
		loggingexporter.NewFactory(),
		otlpexporter.NewFactory(),
		otlphttpexporter.NewFactory(),
		fileexporter.NewFactory(),
	)
	if err != nil {
		return otelcol.Factories{}, err
	}

	factories.Processors, err = processor.MakeFactoryMap(
		batchprocessor.NewFactory(),
		memorylimiterprocessor.NewFactory(),
		experimentprocessor.NewFactory(),
		obfuscationprocessor.NewFactory(),
	)
	if err != nil {
		return otelcol.Factories{}, err
	}

	factories.Connectors, err = connector.MakeFactoryMap(
		validationconnector.NewFactory(),
	)

	return factories, nil
}
