// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package blockingmemorylimiterextension // import "github.com/open-telemetry/otel-arrow/collector/blockingmemorylimiterextension"

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/semaphore"
	"go.uber.org/zap"
	"google.golang.org/grpc"

	"go.opentelemetry.io/collector/component"
)

type blockingMLExtension struct {
	limitBytes int64
	sem        *semaphore.Weighted
	logger     *zap.Logger
	timeout    time.Duration
}

// This interface is meant to access the size of a
// ExportTraceServiceRequest, ExportMetricsServiceRequest, ExportLogsServicesRequest
type telemetryServiceRequest = interface { 
	Size() int
}


// newMemoryLimiter returns a new memorylimiter extension.
func newBlockingMLExtension(cfg *Config, logger *zap.Logger) (*blockingMLExtension, error) {
	limitBytes := int64(cfg.MemoryLimitMiB) << 20
	return &blockingMLExtension{
		limitBytes: limitBytes,
		sem: semaphore.NewWeighted(limitBytes),
		timeout: cfg.Timeout,
		logger: logger,
	}, nil
}

func (bml *blockingMLExtension) Start(ctx context.Context, host component.Host) error {
	return nil 
}

func (bml *blockingMLExtension) Shutdown(ctx context.Context) error {
	return nil 
}

func (bml *blockingMLExtension) MustRefuse() bool {
	return false
}

func (bml *blockingMLExtension) UnaryInterceptorGenerator() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp any, err error) {
		a := req.(telemetryServiceRequest)
		requestSize := int64(a.Size())
		
		semCtx, cancel := context.WithTimeout(context.Background(), bml.timeout)
		defer cancel()

		err = bml.sem.Acquire(semCtx, requestSize)
		if err != nil {
			return nil, fmt.Errorf("not enough memory available to process request, %w", err)
		}

		resp, err = handler(ctx, req)
		bml.sem.Release(requestSize)

		return resp, err
	}
}
