// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package memorylimiterextension // import "go.opentelemetry.io/collector/extension/memorylimiterextension"

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/semaphore"
	"go.uber.org/zap"
	"google.golang.org/grpc"

	"go.opentelemetry.io/collector/component"
)

type wrappedHandlerFn func(ctx context.Context, req any, handler grpc.UnaryHandler) (any, error)
type memoryLimiterExtension struct {
	limitBytes int64
	sem        *semaphore.Weighted
	logger     *zap.Logger
	timeout    time.Duration
	UnaryHandlerFn wrappedHandlerFn 
}

// This interface is meant to access the size of a
// ExportTraceServiceRequest, ExportMetricsServiceRequest, ExportLogsServicesRequest
type telemetryServiceRequest = interface { Size() int }

func (ml *memoryLimiterExtension) UnaryInterceptor(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp any, err error) {
	return nil, nil
}

func (ml *memoryLimiterExtension) UnaryHandle(handlerCtx context.Context, req any, handler grpc.UnaryHandler) (any, error) {
	a := req.(telemetryServiceRequest)
	requestSize := int64(a.Size())
	fmt.Println("ACQUIRING")
	fmt.Println(requestSize)
	semCtx, cancel := context.WithTimeout(context.Background(), ml.timeout)
	defer cancel()

	err := ml.sem.Acquire(semCtx, requestSize)
	if err != nil {
		return nil, fmt.Errorf("not enough memory available to process request, %w", err)
	}

	resp, err := handler(handlerCtx, req)

	ml.sem.Release(requestSize)

	return resp, err
}

// newMemoryLimiter returns a new memorylimiter extension.
func newMemoryLimiter(cfg *Config, logger *zap.Logger) (*memoryLimiterExtension, error) {
	limitBytes := int64(cfg.MemoryLimitMiB) << 20
	return &memoryLimiterExtension{
		limitBytes: limitBytes,
		sem: semaphore.NewWeighted(limitBytes),
		timeout: cfg.Timeout,
		logger: logger,
	}, nil
}

func (ml *memoryLimiterExtension) Start(ctx context.Context, host component.Host) error {
	return nil 
}

func (ml *memoryLimiterExtension) Shutdown(ctx context.Context) error {
	return nil 
}

// MustRefuse returns if the caller should deny because memory has reached it's configured limits
func (ml *memoryLimiterExtension) MustRefuse() bool {
	return false
}

func (ml *memoryLimiterExtension) ReleaseMemory(sizeBytes int64) {
	fmt.Println("RELEASING")
	fmt.Println(sizeBytes)
	ml.sem.Release(sizeBytes)
}