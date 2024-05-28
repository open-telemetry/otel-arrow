// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package netstats

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"google.golang.org/grpc/stats"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configtelemetry"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/receiver"
)

func metricValues(t *testing.T, rm metricdata.ResourceMetrics, expectMethod string) map[string]interface{} {
	res := map[string]interface{}{}
	for _, sm := range rm.ScopeMetrics {
		for _, mm := range sm.Metrics {
			var value int64
			var attrs attribute.Set
			switch t := mm.Data.(type) {
			case metricdata.Histogram[int64]:
				for _, dp := range t.DataPoints {
					value = dp.Sum // histogram tested as the sum
					attrs = dp.Attributes
				}
			case metricdata.Sum[int64]:
				for _, dp := range t.DataPoints {
					value = dp.Value
					attrs = dp.Attributes
				}
			}
			var method string
			for _, attr := range attrs.ToSlice() {
				if attr.Key == "method" {
					method = attr.Value.AsString()
				}
			}

			require.Equal(t, expectMethod, method)
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
		"exporter_sent":            int64(1000),
		"exporter_sent_wire":       int64(100),
		"exporter_recv_wire":       int64(10),
		"exporter_compressed_size": int64(100), // same as sent_wire b/c sum metricValue uses histogram sum
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
				ID: component.NewID(component.MustNewType("test")),
				TelemetrySettings: component.TelemetrySettings{
					MeterProvider: mp,
					MetricsLevel:  level,
				},
			})
			require.NoError(t, err)
			handler := enr.Handler()

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
					handler.HandleRPC(handler.TagRPC(ctx, &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.OutPayload{
						Length:     100,
						WireLength: 10,
					})
					handler.HandleRPC(handler.TagRPC(ctx, &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.InPayload{
						Length:     10,
						WireLength: 1,
					})
				}
			}
			var rm metricdata.ResourceMetrics
			err = rdr.Collect(ctx, &rm)
			require.NoError(t, err)

			require.Equal(t, expect, metricValues(t, rm, "Hello"))
		})
	}
}

func TestNetStatsSetSpanAttrs(t *testing.T) {
	tests := []struct {
		name       string
		attrs      []attribute.KeyValue
		isExporter bool
		length     int
		wireLength int
	}{
		{
			name:       "set exporter attributes",
			isExporter: true,
			length:     1234567,
			wireLength: 123,
			attrs: []attribute.KeyValue{
				attribute.Int("stream_client_uncompressed_bytes_sent", 1234567),
				attribute.Int("stream_client_compressed_bytes_sent", 123),
			},
		},
		{
			name:       "set receiver attributes",
			isExporter: false,
			length:     8901234,
			wireLength: 890,
			attrs: []attribute.KeyValue{
				attribute.Int("stream_server_uncompressed_bytes_recv", 8901234),
				attribute.Int("stream_server_compressed_bytes_recv", 890),
			},
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			enr := &NetworkReporter{
				isExporter: tc.isExporter,
			}

			tp := sdktrace.NewTracerProvider()
			ctx, sp := tp.Tracer("test/span").Start(context.Background(), "test-op")

			var sized SizesStruct
			sized.Method = "test"
			sized.Length = int64(tc.length)
			sized.WireLength = int64(tc.wireLength)
			enr.SetSpanSizeAttributes(ctx, sized)

			actualAttrs := sp.(sdktrace.ReadOnlySpan).Attributes()

			require.Equal(t, tc.attrs, actualAttrs)
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
		"receiver_recv":            int64(1000),
		"receiver_recv_wire":       int64(100),
		"receiver_sent_wire":       int64(10),
		"receiver_compressed_size": int64(100), // same as recv_wire b/c sum metricValue uses histogram sum
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
				ID: component.NewID(component.MustNewType("test")),
				TelemetrySettings: component.TelemetrySettings{
					MeterProvider: mp,
					MetricsLevel:  level,
				},
			})
			require.NoError(t, err)
			handler := rer.Handler()

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
					handler.HandleRPC(handler.TagRPC(ctx, &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.InPayload{
						Length:     100,
						WireLength: 10,
					})
					handler.HandleRPC(handler.TagRPC(ctx, &stats.RPCTagInfo{
						FullMethodName: "Hello",
					}), &stats.OutPayload{
						Length:     10,
						WireLength: 1,
					})
				}
			}
			var rm metricdata.ResourceMetrics
			err = rdr.Collect(ctx, &rm)
			require.NoError(t, err)

			require.Equal(t, expect, metricValues(t, rm, "Hello"))
		})
	}
}

func TestUncompressedSizeBypass(t *testing.T) {
	rdr := metric.NewManualReader()
	mp := metric.NewMeterProvider(
		metric.WithResource(resource.Empty()),
		metric.WithReader(rdr),
	)
	enr, err := NewExporterNetworkReporter(exporter.CreateSettings{
		ID: component.NewID(component.MustNewType("test")),
		TelemetrySettings: component.TelemetrySettings{
			MeterProvider: mp,
			MetricsLevel:  configtelemetry.LevelDetailed,
		},
	})
	require.NoError(t, err)
	handler := enr.Handler()

	ctx := context.Background()
	for i := 0; i < 10; i++ {
		// simulate the RPC path
		handler.HandleRPC(handler.TagRPC(ctx, &stats.RPCTagInfo{
			FullMethodName: "my.arrow.v1.method",
		}), &stats.OutPayload{
			Length:     9999,
			WireLength: 10,
		})
		handler.HandleRPC(handler.TagRPC(ctx, &stats.RPCTagInfo{
			FullMethodName: "my.arrow.v1.method",
		}), &stats.InPayload{
			Length:     9999,
			WireLength: 1,
		})
		// There would bo no uncompressed size metric w/o this call
		// and if the bypass didn't work, we would count the 9999s above.
		enr.CountSend(ctx, SizesStruct{
			Method: "my.arrow.v1.method",
			Length: 100,
		})
	}
	var rm metricdata.ResourceMetrics
	err = rdr.Collect(ctx, &rm)
	require.NoError(t, err)

	expect := map[string]interface{}{
		"exporter_sent":            int64(1000),
		"exporter_sent_wire":       int64(100),
		"exporter_recv_wire":       int64(10),
		"exporter_compressed_size": int64(100),
	}
	require.Equal(t, expect, metricValues(t, rm, "my.arrow.v1.method"))
}
