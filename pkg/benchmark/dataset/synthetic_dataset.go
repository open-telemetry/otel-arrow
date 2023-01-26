// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package dataset

import (
	"math/rand"

	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"

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
	SizeInBytes() int
}

type TraceDataset interface {
	Len() int
	Traces(start, size int) []ptrace.Traces
}

func ResourceAndScopeId(r pcommon.Resource, is pcommon.InstrumentationScope) string {
	return arrow.ResourceID(r) + "|" + arrow.ScopeID(is)
}

// ===== Fake metrics dataset =====

// FakeMetricsDataset is an implementation of MetricsDataset returning fake metrics.
type FakeMetricsDataset struct {
	len       int
	generator *datagen.MetricsGenerator
}

func NewFakeMetricsDataset(size int) *FakeMetricsDataset {
	//#nosec G404 -- This is a false positive, this random number generator is not used for test purposes
	entropy := datagen.NewTestEntropy(int64(rand.Uint64()))
	return &FakeMetricsDataset{len: size, generator: datagen.NewMetricsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())}
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

func NewFakeLogsDataset(size int) *FakeLogsDataset {
	//#nosec G404 -- This is a false positive, this random number generator is not used for test purposes
	entropy := datagen.NewTestEntropy(int64(rand.Uint64()))
	return &FakeLogsDataset{len: size, generator: datagen.NewLogsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())}
}

func (d *FakeLogsDataset) SizeInBytes() int {
	return 0
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

func NewFakeTraceDataset(size int) *FakeTraceDataset {
	//#nosec G404 -- This is a false positive, this random number generator is not used for test purposes
	entropy := datagen.NewTestEntropy(int64(rand.Uint64()))
	return &FakeTraceDataset{len: size, generator: datagen.NewTracesGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())}
}

func (d *FakeTraceDataset) Len() int {
	return d.len
}

func (d *FakeTraceDataset) Traces(_, size int) []ptrace.Traces {
	return []ptrace.Traces{d.generator.Generate(size, 100)}
}
