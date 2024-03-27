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

package experimentprocessor

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/component/componenttest"
	"go.opentelemetry.io/collector/consumer/consumertest"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.uber.org/zap"
)

type mockHost struct {
	component.Host
	exps map[component.DataType]map[component.ID]component.Component
}

type mockMetricsExporter struct {
	mockComponent
	consumertest.MetricsSink
}

type mockTracesExporter struct {
	mockComponent
	consumertest.TracesSink
}

type mockComponent struct {
	component.StartFunc
	component.ShutdownFunc
}

func newMockHost(exps map[component.DataType]map[component.ID]component.Component) component.Host {
	return &mockHost{
		Host: componenttest.NewNopHost(),
		exps: exps,
	}
}

func (m *mockHost) GetExporters() map[component.DataType]map[component.ID]component.Component {
	return m.exps
}

func TestTraces(t *testing.T) {
	runTest := func(send int, table []RoutingTableItem) []*mockTracesExporter {
		exps := []*mockTracesExporter{{}, {}, {}}

		host := newMockHost(map[component.DataType]map[component.ID]component.Component{
			component.DataTypeTraces: {
				component.NewIDWithName(component.MustNewType("otlp"), "1"): exps[0],
				component.NewIDWithName(component.MustNewType("otlp"), "2"): exps[1],
				component.NewID(component.MustNewType("otlp")):              exps[2],
			},
		})

		eproc := newTracesProcessor(component.TelemetrySettings{Logger: zap.NewNop()}, &Config{
			Table: table,
		})
		// Use a deterministic uniform random source.
		index := new(int)
		eproc.router.randIntn = func(d int) int {
			r := *index
			*index++
			return r % d
		}

		require.NoError(t, eproc.Start(context.Background(), host))

		for count := 0; count < send; count++ {
			tr := ptrace.NewTraces()

			rs := tr.ResourceSpans().AppendEmpty()
			span := rs.ScopeSpans().AppendEmpty().Spans().AppendEmpty()
			span.SetName("span")

			require.NoError(t, eproc.ConsumeTraces(context.Background(), tr))
		}
		return exps
	}

	t.Run("traces 1/9/90", func(t *testing.T) {
		exps := runTest(10000, []RoutingTableItem{
			{
				Weight:    1,
				Exporters: []string{"otlp/1"},
			},
			{
				Weight:    9,
				Exporters: []string{"otlp/2"},
			},
			{
				Weight:    90,
				Exporters: []string{"otlp"},
			},
		})

		assert.Len(t, exps[0].AllTraces(), 100)
		assert.Len(t, exps[1].AllTraces(), 900)
		assert.Len(t, exps[2].AllTraces(), 9000)
	})

	t.Run("traces 50/50/0", func(t *testing.T) {
		exps := runTest(10000, []RoutingTableItem{
			{
				Weight:    0,
				Exporters: []string{"otlp/1"},
			},
			{
				Weight:    50,
				Exporters: []string{"otlp/2"},
			},
			{
				Weight:    50,
				Exporters: []string{"otlp"},
			},
		})

		assert.Len(t, exps[0].AllTraces(), 0)
		assert.Len(t, exps[1].AllTraces(), 5000)
		assert.Len(t, exps[2].AllTraces(), 5000)
	})

	t.Run("traces 1/2/3", func(t *testing.T) {
		exps := runTest(6000, []RoutingTableItem{
			{
				Weight:    1,
				Exporters: []string{"otlp/1"},
			},
			{
				Weight:    2,
				Exporters: []string{"otlp/2"},
			},
			{
				Weight:    3,
				Exporters: []string{"otlp"},
			},
		})

		assert.Len(t, exps[0].AllTraces(), 1000)
		assert.Len(t, exps[1].AllTraces(), 2000)
		assert.Len(t, exps[2].AllTraces(), 3000)
	})
}
