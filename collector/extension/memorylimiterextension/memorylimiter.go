// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package memorylimiterextension // import "go.opentelemetry.io/collector/extension/memorylimiterextension"

import (
	"context"
	"fmt"

	"golang.org/x/sync/semaphore"
	"go.uber.org/zap"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

type memoryLimiterExtension struct {
	limitBytes int64
	sem        *semaphore.Weighted
}

// newMemoryLimiter returns a new memorylimiter extension.
func newMemoryLimiter(cfg *Config, logger *zap.Logger) (*memoryLimiterExtension, error) {
	limitBytes := int64(cfg.MemoryLimitMiB) << 20
	return &memoryLimiterExtension{
		limitBytes: limitBytes,
		sem: semaphore.NewWeighted(limitBytes),
	}, nil
}

func (ml *memoryLimiterExtension) Start(ctx context.Context, host component.Host) error {
	return nil 
}

func (ml *memoryLimiterExtension) Shutdown(ctx context.Context) error {
	return nil 
}

// MustRefuse returns if the caller should deny because memory has reached it's configured limits
func (ml *memoryLimiterExtension) MustRefuse(req any) bool {
	fmt.Println("TYPE OF REQ")
	switch td := req.(type) {
	case ptrace.Traces:
		fmt.Println("thank God")
	
	default:
		fmt.Println("default")
		fmt.Println(td.(ptraceotlp.ExportRequest).MarshalJSON())
	}
	return true


	// check if there is room for another request. Note that the request might push us over limitBytes, but we won't know the request size until
	// the Handler() handles the request. If the request exceeds the limit then we will block until more room is available 
	// TODO: Should this always return false and have the handler handle the logic?
	return ml.sem.TryAcquire(1)
}