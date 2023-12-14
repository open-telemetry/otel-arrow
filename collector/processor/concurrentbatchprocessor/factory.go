// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package concurrentbatchprocessor // import "github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor"

import (
	"context"
	"time"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/processor"
)

const (
	// The value of "type" key in configuration.
	typeStr = "concurrentbatch"

	defaultSendBatchSize = uint32(8192)
	defaultTimeout       = 200 * time.Millisecond
	// default inflight bytes is 2 MiB
	defaultMaxMiB = 2

	// defaultMetadataCardinalityLimit should be set to the number
	// of metadata configurations the user expects to submit to
	// the collector.
	defaultMetadataCardinalityLimit = 1000
)

// This featuregate might already be registered, but is access is internal to the collector repo.
// In the meantime this batchprocessor will only support otel for metrics (no support for opencensus).
var UseOtelForInternalMetricsfeatureGate = true

// NewFactory returns a new factory for the Batch processor.
func NewFactory() processor.Factory {
	return processor.NewFactory(
		typeStr,
		createDefaultConfig,
		processor.WithTraces(createTraces, component.StabilityLevelStable),
		processor.WithMetrics(createMetrics, component.StabilityLevelStable),
		processor.WithLogs(createLogs, component.StabilityLevelStable))
}

func createDefaultConfig() component.Config {
	return &Config{
		SendBatchSize:            defaultSendBatchSize,
		Timeout:                  defaultTimeout,
		MaxInFlightBytesMiB:      defaultMaxMiB,
		MetadataCardinalityLimit: defaultMetadataCardinalityLimit,
	}
}

func createTraces(
	_ context.Context,
	set processor.CreateSettings,
	cfg component.Config,
	nextConsumer consumer.Traces,
) (processor.Traces, error) {
	return newBatchTracesProcessor(set, nextConsumer, cfg.(*Config), UseOtelForInternalMetricsfeatureGate)
}

func createMetrics(
	_ context.Context,
	set processor.CreateSettings,
	cfg component.Config,
	nextConsumer consumer.Metrics,
) (processor.Metrics, error) {
	return newBatchMetricsProcessor(set, nextConsumer, cfg.(*Config), UseOtelForInternalMetricsfeatureGate)
}

func createLogs(
	_ context.Context,
	set processor.CreateSettings,
	cfg component.Config,
	nextConsumer consumer.Logs,
) (processor.Logs, error) {
	return newBatchLogsProcessor(set, nextConsumer, cfg.(*Config), UseOtelForInternalMetricsfeatureGate)
}
