// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package netstats

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
	"go.opentelemetry.io/otel/sdk/resource"
	"google.golang.org/grpc/stats"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/receiver"
)

func metricValues(t *testing.T, rm metricdata.ResourceMetrics) map[string]interface{} {
	res := map[string]interface{}{}
	for _, sm := range rm.ScopeMetrics {
		for _, mm := range sm.Metrics {
			var value int64
			var method string
			for _, dp := range mm.Data.(metricdata.Sum[int64]).DataPoints {
				value = dp.Value
				for _, attr := range dp.Attributes.ToSlice() {
					if attr.Key == "method" {
						method = attr.Value.AsString()
					}
				}
			}
			// Require a method named "Hello"
			require.Equal(t, "Hello", method)
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
	for _, apiDirect := range []bool{true, false} {
		t.Run(func() string {
			if apiDirect {
				return "direct"
			}
			return "grpc"
		}(), func(t *testing.T) {
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
				if apiDirect {
					// use the direct API
					enr.CountSend(ctx, SizesStruct{
						Method:     "Hello",
						Length:     100,
						WireLength: 10,
					})
					enr.CountReceive(ctx, SizesStruct{
						Method:     "Hello",
						Length:     10,
						WireLength: 1,
					})
				} else {
					// simulate the RPC path
					enr.HandleRPC(enr.TagRPC(UncompressedSizeContext(ctx, 100), &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.OutPayload{
						WireLength: 10,
					})
					enr.HandleRPC(enr.TagRPC(UncompressedSizeContext(ctx, 10), &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.InPayload{
						WireLength: 1,
					})
				}
			}
			var rm metricdata.ResourceMetrics
			err = rdr.Collect(ctx, &rm)
			require.NoError(t, err)

			require.Equal(t, expect, metricValues(t, rm))
		})
	}
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
	for _, apiDirect := range []bool{true, false} {
		t.Run(func() string {
			if apiDirect {
				return "direct"
			}
			return "grpc"
		}(), func(t *testing.T) {
			rdr := metric.NewManualReader()
			mp := metric.NewMeterProvider(
				metric.WithResource(resource.Empty()),
				metric.WithReader(rdr),
			)
			rer, err := NewReceiverNetworkReporter(receiver.CreateSettings{
				ID: component.NewID("test"),
				TelemetrySettings: component.TelemetrySettings{
					MeterProvider: mp,
					MetricsLevel:  level,
				},
			})
			require.NoError(t, err)

			ctx := context.Background()
			for i := 0; i < 10; i++ {
				if apiDirect {
					// use the direct API
					rer.CountReceive(ctx, SizesStruct{
						Method:     "Hello",
						Length:     100,
						WireLength: 10,
					})
					rer.CountSend(ctx, SizesStruct{
						Method:     "Hello",
						Length:     10,
						WireLength: 1,
					})
				} else {
					// simulate the RPC path
					rer.HandleRPC(rer.TagRPC(UncompressedSizeContext(ctx, 100), &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.InPayload{
						WireLength: 10,
					})
					rer.HandleRPC(rer.TagRPC(UncompressedSizeContext(ctx, 10), &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.OutPayload{
						WireLength: 1,
					})
				}
			}
			var rm metricdata.ResourceMetrics
			err = rdr.Collect(ctx, &rm)
			require.NoError(t, err)

			require.Equal(t, expect, metricValues(t, rm))
		})
	}
}
