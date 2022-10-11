/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package dataset

import (
	"otel-arrow-adapter/pkg/datagen"
	"otel-arrow-adapter/pkg/otel/common"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

type MetricsDataset interface {
	Len() int
	Metrics(start, size int) []pmetric.Metrics
}

type LogsDataset interface {
	Len() int
	Logs(start, size int) []plog.Logs
}

type TraceDataset interface {
	Len() int
	Traces(start, size int) []ptrace.Traces
}

func ResourceAndScopeId(r pcommon.Resource, is pcommon.InstrumentationScope) string {
	return common.ResourceId(r) + "|" + common.ScopeId(is)
}

// ===== Fake metrics dataset =====

// FakeMetricsDataset is an implementation of MetricsDataset returning fake metrics.
type FakeMetricsDataset struct {
	len       int
	generator *datagen.MetricsGenerator
}

func NewFakeMetricsDataset(len int) *FakeMetricsDataset {
	return &FakeMetricsDataset{len: len, generator: datagen.NewMetricsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())}
}

func (d *FakeMetricsDataset) Len() int {
	return d.len
}

func (d *FakeMetricsDataset) Metrics(_, size int) []pmetric.Metrics {
	return []pmetric.Metrics{d.generator.Generate(size, 100)}
}

// ===== Fake logs dataset =====

// FakeLogsDataset is an implementation of LogsDataset returning fake logs.
type FakeLogsDataset struct {
	len       int
	generator *datagen.LogsGenerator
}

func NewFakeLogsDataset(len int) *FakeLogsDataset {
	return &FakeLogsDataset{len: len, generator: datagen.NewLogsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())}
}

func (d *FakeLogsDataset) Len() int {
	return d.len
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

func NewFakeTraceDataset(len int) *FakeTraceDataset {
	return &FakeTraceDataset{len: len, generator: datagen.NewTraceGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())}
}

func (d *FakeTraceDataset) Len() int {
	return d.len
}

func (d *FakeTraceDataset) Traces(_, size int) []ptrace.Traces {
	return []ptrace.Traces{d.generator.Generate(size, 100)}
}
