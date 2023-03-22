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

package netstats

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/receiver"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
	"go.opentelemetry.io/otel/sdk/resource"
)

func metricValues(rm metricdata.ResourceMetrics) map[string]interface{} {
	res := map[string]interface{}{}
	for _, sm := range rm.ScopeMetrics {
		for _, mm := range sm.Metrics {
			var value int64
			for _, dp := range mm.Data.(metricdata.Sum[int64]).DataPoints {
				value = dp.Value
			}
			res[mm.Name] = value
		}
	}
	return res
}

func TestNetStatsExporterNone(t *testing.T) {
	testNetStatsExporter(t, configtelemetry.LevelNone, map[string]interface{}{})
}

func TestNetStatsExporterNormal(t *testing.T) {
	testNetStatsExporter(t, configtelemetry.LevelNormal, map[string]interface{}{
		"exporter_sent":      int64(1000),
		"exporter_sent_wire": int64(100),
	})
}

func TestNetStatsExporterDetailed(t *testing.T) {
	testNetStatsExporter(t, configtelemetry.LevelDetailed, map[string]interface{}{
		"exporter_sent":      int64(1000),
		"exporter_sent_wire": int64(100),
		"exporter_recv":      int64(100),
		"exporter_recv_wire": int64(10),
	})
}

func testNetStatsExporter(t *testing.T, level configtelemetry.Level, expect map[string]interface{}) {

	rdr := metric.NewManualReader()
	mp := metric.NewMeterProvider(
		metric.WithResource(resource.Empty()),
		metric.WithReader(rdr),
	)
	enr, err := NewExporterNetworkReporter(exporter.CreateSettings{
		ID: component.NewID("test"),
		TelemetrySettings: component.TelemetrySettings{
			MeterProvider: mp,
			MetricsLevel:  level,
		},
	})
	require.NoError(t, err)

	ctx := context.Background()
	for i := 0; i < 10; i++ {
		enr.CountSend(ctx, SizesStruct{
			Length:     100,
			WireLength: 10,
		})
		enr.CountReceive(ctx, SizesStruct{
			Length:     10,
			WireLength: 1,
		})
	}
	rm, err := rdr.Collect(ctx)
	require.NoError(t, err)

	require.Equal(t, expect, metricValues(rm))
}

func TestNetStatsReceiverNone(t *testing.T) {
	testNetStatsReceiver(t, configtelemetry.LevelNone, map[string]interface{}{})
}

func TestNetStatsReceiverNormal(t *testing.T) {
	testNetStatsReceiver(t, configtelemetry.LevelNormal, map[string]interface{}{
		"receiver_recv":      int64(1000),
		"receiver_recv_wire": int64(100),
	})
}

func TestNetStatsReceiverDetailed(t *testing.T) {
	testNetStatsReceiver(t, configtelemetry.LevelDetailed, map[string]interface{}{
		"receiver_recv":      int64(1000),
		"receiver_recv_wire": int64(100),
		"receiver_sent":      int64(100),
		"receiver_sent_wire": int64(10),
	})
}

func testNetStatsReceiver(t *testing.T, level configtelemetry.Level, expect map[string]interface{}) {

	rdr := metric.NewManualReader()
	mp := metric.NewMeterProvider(
		metric.WithResource(resource.Empty()),
		metric.WithReader(rdr),
	)
	enr, err := NewReceiverNetworkReporter(receiver.CreateSettings{
		ID: component.NewID("test"),
		TelemetrySettings: component.TelemetrySettings{
			MeterProvider: mp,
			MetricsLevel:  level,
		},
	})
	require.NoError(t, err)

	ctx := context.Background()
	for i := 0; i < 10; i++ {
		enr.CountReceive(ctx, SizesStruct{
			Length:     100,
			WireLength: 10,
		})
		enr.CountSend(ctx, SizesStruct{
			Length:     10,
			WireLength: 1,
		})
	}
	rm, err := rdr.Collect(ctx)
	require.NoError(t, err)

	require.Equal(t, expect, metricValues(rm))
}
