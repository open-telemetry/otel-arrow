package benchmark

import (
	colmetrics "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/metrics/v1"
	"otel-arrow-adapter/pkg/otel/fake"
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
	Logs(start, size int) []*coltrace.ExportTraceServiceRequest
}

// ===== Fake metrics dataset =====

// FakeMetricsDataset is an implementation of MetricsDataset returning fake metrics.
type FakeMetricsDataset struct {
	len       int
	generator *fake.MetricsGenerator
}

func NewFakeMetricsDataset(len int) *FakeMetricsDataset {
	return &FakeMetricsDataset{len: len, generator: fake.NewMetricsGenerator(fake.DefaultResourceAttributes(), fake.DefaultInstrumentationScope())}
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
	generator *fake.LogsGenerator
}

func NewFakeLogsDataset(len int) *FakeLogsDataset {
	return &FakeLogsDataset{len: len, generator: fake.NewLogsGenerator(fake.DefaultResourceAttributes(), fake.DefaultInstrumentationScope())}
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
	generator *fake.TraceGenerator
}

func NewFakeTraceDataset(len int) *FakeTraceDataset {
	return &FakeTraceDataset{len: len, generator: fake.NewTraceGenerator(fake.DefaultResourceAttributes(), fake.DefaultInstrumentationScope())}
}

func (d *FakeTraceDataset) Len() int {
	return d.len
}

func (d *FakeTraceDataset) Logs(_, size int) []*coltrace.ExportTraceServiceRequest {
	return []*coltrace.ExportTraceServiceRequest{d.generator.Generate(size, 100)}
}
