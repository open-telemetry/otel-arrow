package benchmark

import (
	colmetrics "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	datagen2 "otel-arrow-adapter/pkg/datagen"
)
import collogs "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/logs/v1"
import coltrace "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/trace/v1"

type MetricsDataset interface {
	Len() int
	Metrics(start, size int) []*colmetrics.ExportMetricsServiceRequest
}

type LogsDataset interface {
	Len() int
	Logs(start, size int) []*collogs.ExportLogsServiceRequest
}

type TraceDataset interface {
	Len() int
	Traces(start, size int) []*coltrace.ExportTraceServiceRequest
}

// ===== Fake metrics dataset =====

// FakeMetricsDataset is an implementation of MetricsDataset returning fake metrics.
type FakeMetricsDataset struct {
	len       int
	generator *datagen2.MetricsGenerator
}

func NewFakeMetricsDataset(len int) *FakeMetricsDataset {
	return &FakeMetricsDataset{len: len, generator: datagen2.NewMetricsGenerator(datagen2.DefaultResourceAttributes(), datagen2.DefaultInstrumentationScope())}
}

func (d *FakeMetricsDataset) Len() int {
	return d.len
}

func (d *FakeMetricsDataset) Metrics(_, size int) []*colmetrics.ExportMetricsServiceRequest {
	return []*colmetrics.ExportMetricsServiceRequest{d.generator.Generate(size, 100)}
}

// ===== Fake logs dataset =====

// FakeLogsDataset is an implementation of LogsDataset returning fake logs.
type FakeLogsDataset struct {
	len       int
	generator *datagen2.LogsGenerator
}

func NewFakeLogsDataset(len int) *FakeLogsDataset {
	return &FakeLogsDataset{len: len, generator: datagen2.NewLogsGenerator(datagen2.DefaultResourceAttributes(), datagen2.DefaultInstrumentationScope())}
}

func (d *FakeLogsDataset) Len() int {
	return d.len
}

func (d *FakeLogsDataset) Logs(_, size int) []*collogs.ExportLogsServiceRequest {
	return []*collogs.ExportLogsServiceRequest{d.generator.Generate(size, 100)}
}

// ===== Fake trace dataset =====

// FakeTraceDataset is an implementation of TraceDataset returning fake traces.
type FakeTraceDataset struct {
	len       int
	generator *datagen2.TraceGenerator
}

func NewFakeTraceDataset(len int) *FakeTraceDataset {
	return &FakeTraceDataset{len: len, generator: datagen2.NewTraceGenerator(datagen2.DefaultResourceAttributes(), datagen2.DefaultInstrumentationScope())}
}

func (d *FakeTraceDataset) Len() int {
	return d.len
}

func (d *FakeTraceDataset) Traces(_, size int) []*coltrace.ExportTraceServiceRequest {
	return []*coltrace.ExportTraceServiceRequest{d.generator.Generate(size, 100)}
}
