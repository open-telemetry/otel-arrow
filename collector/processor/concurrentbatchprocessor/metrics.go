// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package concurrentbatchprocessor // import "github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor"

import (
	"context"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"

	"github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor/internal/metadata"
	"go.opentelemetry.io/collector/processor"
)

type trigger int

const (
	triggerTimeout trigger = iota
	triggerBatchSize
)

type batchProcessorTelemetry struct {
	exportCtx context.Context

	processorAttr    metric.MeasurementOption
	telemetryBuilder *metadata.TelemetryBuilder
}

func newBatchProcessorTelemetry(set processor.Settings, currentMetadataCardinality func() int) (*batchProcessorTelemetry, error) {
	attrs := metric.WithAttributeSet(attribute.NewSet(attribute.String("processor", set.ID.String())))

	telemetryBuilder, err := metadata.NewTelemetryBuilder(set.TelemetrySettings)
	if err != nil {
		return nil, err
	}
	err = telemetryBuilder.RegisterProcessorBatchMetadataCardinalityCallback(func(_ context.Context, observer metric.Int64Observer) error {
		observer.Observe(int64(currentMetadataCardinality()), attrs)
		return nil
	})
	if err != nil {
		return nil, err
	}

	return &batchProcessorTelemetry{
		exportCtx:        context.Background(),
		telemetryBuilder: telemetryBuilder,
		processorAttr:    attrs,
	}, nil
}

func (bpt *batchProcessorTelemetry) record(trigger trigger, sent, bytes int64) {
	switch trigger {
	case triggerBatchSize:
		bpt.telemetryBuilder.ProcessorBatchBatchSizeTriggerSend.Add(bpt.exportCtx, 1, bpt.processorAttr)
	case triggerTimeout:
		bpt.telemetryBuilder.ProcessorBatchTimeoutTriggerSend.Add(bpt.exportCtx, 1, bpt.processorAttr)
	}

	bpt.telemetryBuilder.ProcessorBatchBatchSendSize.Record(bpt.exportCtx, sent, bpt.processorAttr)
	bpt.telemetryBuilder.ProcessorBatchBatchSendSizeBytes.Record(bpt.exportCtx, bytes, bpt.processorAttr)
}
