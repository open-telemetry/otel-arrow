// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package memorylimiterextension // import "go.opentelemetry.io/collector/extension/memorylimiterextension"

import (
	"context"
	"fmt"
	"time"

	arrowpb "github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1"
	"golang.org/x/sync/semaphore"
	"go.uber.org/zap"
	"github.com/golang/protobuf/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"

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

type requestWrappedServerStream struct {
	wrappedCtx context.Context
	stream grpc.ServerStream
	pendingReqs chan any 
	errCh chan error
	timeout time.Duration
}

func (rss *requestWrappedServerStream) SetHeader(md metadata.MD) error {
	return rss.stream.SetHeader(md)
}

func (rss *requestWrappedServerStream) SendHeader(md metadata.MD) error {
	return rss.stream.SendHeader(md)
}

func (rss *requestWrappedServerStream) SetTrailer(md metadata.MD) {
	rss.stream.SetTrailer(md)
}

func (rss *requestWrappedServerStream) Context() context.Context {
	return rss.stream.Context()
}

func (rss *requestWrappedServerStream) SendMsg(m any) error {
	return rss.stream.SendMsg(m)
}

func (rss *requestWrappedServerStream) RecvMsg(m any) error {
	// return rss.stream.RecvMsg(m)
	fmt.Println("ENTERED RECVMSG")

	select {
	case a := <-rss.pendingReqs:
		src := a.(*arrowpb.BatchArrowRecords)
		dest := m.(*arrowpb.BatchArrowRecords)
		*dest = *src 
	case err := <-rss.errCh:
		return err
	}
	return nil
}

// This interface is meant to access the size of a
// ExportTraceServiceRequest, ExportMetricsServiceRequest, ExportLogsServicesRequest
type telemetryServiceRequest = interface { 
	Size() int
}

func (ml *memoryLimiterExtension) UnaryInterceptorGenerator() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp any, err error) {
		a := req.(telemetryServiceRequest)
		requestSize := int64(a.Size())
		
		semCtx, cancel := context.WithTimeout(context.Background(), ml.timeout)
		defer cancel()

		err = ml.sem.Acquire(semCtx, requestSize)
		if err != nil {
			return nil, fmt.Errorf("not enough memory available to process request, %w", err)
		}

		resp, err = handler(ctx, req)
		ml.sem.Release(requestSize)

		return resp, err
	}
}

type streamTelemetryServiceRequest = interface { 
	Size() int
	arrowpb.BatchArrowRecords
}
func (ml *memoryLimiterExtension) StreamInterceptorGenerator() grpc.StreamServerInterceptor {
	return func(srv any, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		rss := &requestWrappedServerStream{
			stream: ss,
			pendingReqs: make(chan any, 5),
			errCh: make(chan error, 5),
			timeout: ml.timeout,
		}
		a := new(arrowpb.BatchArrowRecords)
		var err error
		err = ss.RecvMsg(a)
		if err != nil {
			rss.errCh <- err
			return err
		}

		requestSize := int64(proto.Size(a))
		fmt.Println("ACQUIRING")
		fmt.Println(requestSize)	
		
		semCtx, cancel := context.WithTimeout(context.Background(), ml.timeout)
		defer cancel()

		err = ml.sem.Acquire(semCtx, requestSize)
		if err != nil {
			return fmt.Errorf("not enough memory available to process request, %w", err)
		}

		rss.pendingReqs <- a

		err = handler(srv, rss)

		fmt.Println("after handle")
		fmt.Println(err)
		ml.sem.Release(requestSize)

		return err
	}
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
