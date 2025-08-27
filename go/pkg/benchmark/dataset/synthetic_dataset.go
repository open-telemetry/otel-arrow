/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package dataset

import (
	"strings"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/open-telemetry/otel-arrow/go/pkg/datagen"
	carrow "github.com/open-telemetry/otel-arrow/go/pkg/otel/common/otlp"
)

type MetricsDataset interface {
	Len() int
	Metrics(start, size int) []pmetric.Metrics
}

type LogsDataset interface {
	Len() int
	ShowStats()
	Logs(start, size int) []plog.Logs
	SizeInBytes() int
}

type TraceDataset interface {
	Len() int
	Traces(start, size int) []ptrace.Traces
}

func ResourceAndScopeId(r pcommon.Resource, is pcommon.InstrumentationScope) string {
	var b strings.Builder
	b.WriteString(carrow.ResourceID(r, ""))
	b.WriteString("|")
	b.WriteString(carrow.ScopeID(is, ""))
	return b.String()
}

// ===== Fake metrics dataset =====

// FakeMetricsDataset is an implementation of MetricsDataset returning fake metrics.
type FakeMetricsDataset struct {
	len       int
	generator *datagen.MetricsGenerator
}

func NewFakeMetricsDataset(size int) *FakeMetricsDataset {
	entropy := datagen.NewTestEntropy()
	return &FakeMetricsDataset{len: size, generator: datagen.NewMetricsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())}
}

func (d *FakeMetricsDataset) Len() int {
	return d.len
}

func (d *FakeMetricsDataset) Metrics(_, size int) []pmetric.Metrics {
	return []pmetric.Metrics{d.generator.GenerateAllKindOfMetrics(size, 100)}
}

// ===== Fake logs dataset =====

// FakeLogsDataset is an implementation of LogsDataset returning fake logs.
type FakeLogsDataset struct {
	len       int
	generator *datagen.LogsGenerator
}

func NewFakeLogsDataset(size int) *FakeLogsDataset {
	entropy := datagen.NewTestEntropy()
	return &FakeLogsDataset{len: size, generator: datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())}
}

func (d *FakeLogsDataset) SizeInBytes() int {
	return 0
}

func (d *FakeLogsDataset) Len() int {
	return d.len
}

func (d *FakeLogsDataset) ShowStats() {
	// Not implemented
}

func (d *FakeLogsDataset) Logs(_, size int) []plog.Logs {
	return []plog.Logs{d.generator.Generate(size, 100)}
}

// ===== Fake traces dataset =====

// FakeTraceDataset is an implementation of TraceDataset returning fake traces.
type FakeTraceDataset struct {
	len       int
	generator *datagen.TraceGenerator
}

func NewFakeTraceDataset(size int) *FakeTraceDataset {
	entropy := datagen.NewTestEntropy()
	return &FakeTraceDataset{len: size, generator: datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())}
}

func (d *FakeTraceDataset) Len() int {
	return d.len
}

func (d *FakeTraceDataset) Traces(_, size int) []ptrace.Traces {
	return []ptrace.Traces{d.generator.Generate(size, 100)}
}
