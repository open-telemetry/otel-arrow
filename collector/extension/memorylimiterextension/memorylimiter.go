// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package memorylimiterextension // import "go.opentelemetry.io/collector/extension/memorylimiterextension"

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/semaphore"
	"go.uber.org/zap"

	"go.opentelemetry.io/collector/component"
)

type memoryLimiterExtension struct {
	limitBytes int64
	sem        *semaphore.Weighted
	logger     *zap.Logger
	timeout    time.Duration
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
func (ml *memoryLimiterExtension) MustRefuse(sizeBytes int64) bool {
	fmt.Println("TYPE OF REQ")

	fmt.Println("ACQUIRING")
	fmt.Println(sizeBytes)
	ctx, cancel := context.WithTimeout(context.Background(), ml.timeout)
	defer cancel()

	err := ml.sem.Acquire(ctx, sizeBytes)
	if err != nil {
		ml.logger.Debug("rejecting request exceeded memory limit", zap.Error(err))
		return true
	}

	return false
}

func (ml *memoryLimiterExtension) ReleaseMemory(sizeBytes int64) {
	fmt.Println("RELEASING")
	fmt.Println(sizeBytes)
	ml.sem.Release(sizeBytes)
}