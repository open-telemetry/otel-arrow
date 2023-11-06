// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelarrowexporter // import "github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter"

import (
	"context"
	"runtime"
	"time"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"google.golang.org/grpc"

	"github.com/open-telemetry/otel-arrow/collector/compression/zstd"
	"github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter/internal/arrow"
	"github.com/open-telemetry/otel-arrow/collector/netstats"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configcompression"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/config/configopaque"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/exporter/exporterhelper"
)

const (
	// The value of "type" key in configuration.
	typeStr = "otelarrow"
)

// NewFactory creates a factory for OTLP exporter.
func NewFactory() exporter.Factory {
	return exporter.NewFactory(
		typeStr,
		createDefaultConfig,
		exporter.WithTraces(createTracesExporter, component.StabilityLevelStable),
		exporter.WithMetrics(createMetricsExporter, component.StabilityLevelStable),
		exporter.WithLogs(createLogsExporter, component.StabilityLevelBeta),
	)
}

func createDefaultConfig() component.Config {
	return &Config{
		TimeoutSettings: exporterhelper.NewDefaultTimeoutSettings(),
		RetrySettings:   exporterhelper.NewDefaultRetrySettings(),
		QueueSettings:   exporterhelper.NewDefaultQueueSettings(),
		GRPCClientSettings: configgrpc.GRPCClientSettings{
			Headers: map[string]configopaque.String{},
			// Default to zstd compression
			Compression: configcompression.Zstd,
			// We almost read 0 bytes, so no need to tune ReadBufferSize.
			WriteBufferSize: 512 * 1024,
			// The `configgrpc` default is pick_first,
			// which is not great for OTel Arrow exporters
			// because it concentrates load at a single
			// destination.
			BalancerName: "round_robin",
		},
		Arrow: ArrowSettings{
			NumStreams:        runtime.NumCPU(),
			MaxStreamLifetime: time.Hour,

			Zstd: zstd.DefaultEncoderConfig(),

			// PayloadCompression is off by default because gRPC
			// compression is on by default, above.
			PayloadCompression: "",
		},
	}
}

func (oce *baseExporter) helperOptions() []exporterhelper.Option {
	return []exporterhelper.Option{
		exporterhelper.WithCapabilities(consumer.Capabilities{MutatesData: false}),
		exporterhelper.WithTimeout(oce.config.TimeoutSettings),
		exporterhelper.WithRetry(oce.config.RetrySettings),
		exporterhelper.WithQueue(oce.config.QueueSettings),
		exporterhelper.WithStart(oce.start),
		exporterhelper.WithShutdown(oce.shutdown),
	}
}

func gRPCName(desc grpc.ServiceDesc) string {
	return netstats.GRPCStreamMethodName(desc, desc.Streams[0])
}

var (
	arrowStreamMethod  = gRPCName(arrowpb.ArrowStreamService_ServiceDesc)
	arrowTracesMethod  = gRPCName(arrowpb.ArrowTracesService_ServiceDesc)
	arrowMetricsMethod = gRPCName(arrowpb.ArrowMetricsService_ServiceDesc)
	arrowLogsMethod    = gRPCName(arrowpb.ArrowLogsService_ServiceDesc)
)

func createArrowTracesStream(cfg *Config, conn *grpc.ClientConn) arrow.StreamClientFunc {
	if cfg.Arrow.EnableMixedSignals {
		return arrow.MakeAnyStreamClient(arrowStreamMethod, arrowpb.NewArrowStreamServiceClient(conn).ArrowStream)
	}
	return arrow.MakeAnyStreamClient(arrowTracesMethod, arrowpb.NewArrowTracesServiceClient(conn).ArrowTraces)
}

func createTracesExporter(
	ctx context.Context,
	set exporter.CreateSettings,
	cfg component.Config,
) (exporter.Traces, error) {
	oce, err := newExporter(cfg, set, createArrowTracesStream)
	if err != nil {
		return nil, err
	}
	return exporterhelper.NewTracesExporter(ctx, oce.settings, oce.config,
		oce.pushTraces,
		oce.helperOptions()...,
	)
}

func createArrowMetricsStream(cfg *Config, conn *grpc.ClientConn) arrow.StreamClientFunc {
	if cfg.Arrow.EnableMixedSignals {
		return arrow.MakeAnyStreamClient(arrowStreamMethod, arrowpb.NewArrowStreamServiceClient(conn).ArrowStream)
	}
	return arrow.MakeAnyStreamClient(arrowMetricsMethod, arrowpb.NewArrowMetricsServiceClient(conn).ArrowMetrics)
}

func createMetricsExporter(
	ctx context.Context,
	set exporter.CreateSettings,
	cfg component.Config,
) (exporter.Metrics, error) {
	oce, err := newExporter(cfg, set, createArrowMetricsStream)
	if err != nil {
		return nil, err
	}
	return exporterhelper.NewMetricsExporter(ctx, oce.settings, oce.config,
		oce.pushMetrics,
		oce.helperOptions()...,
	)
}

func createArrowLogsStream(cfg *Config, conn *grpc.ClientConn) arrow.StreamClientFunc {
	if cfg.Arrow.EnableMixedSignals {
		return arrow.MakeAnyStreamClient(arrowStreamMethod, arrowpb.NewArrowStreamServiceClient(conn).ArrowStream)
	}
	return arrow.MakeAnyStreamClient(arrowLogsMethod, arrowpb.NewArrowLogsServiceClient(conn).ArrowLogs)
}

func createLogsExporter(
	ctx context.Context,
	set exporter.CreateSettings,
	cfg component.Config,
) (exporter.Logs, error) {
	oce, err := newExporter(cfg, set, createArrowLogsStream)
	if err != nil {
		return nil, err
	}
	return exporterhelper.NewLogsExporter(ctx, oce.settings, oce.config,
		oce.pushLogs,
		oce.helperOptions()...,
	)
}
