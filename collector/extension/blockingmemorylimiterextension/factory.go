// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package blockingmemorylimiterextension // import "github.com/open-telemetry/otel-arrow/collector/blockingmemorylimiterextension"

//go:generate mdatagen metadata.yaml

import (
	"context"
	"time"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/extension"

	"github.com/open-telemetry/otel-arrow/collector/extension/blockingmemorylimiterextension/internal/metadata"
)

// NewFactory returns a new factory for the Memory Limiter extension.
func NewFactory() extension.Factory {
	return extension.NewFactory(
		metadata.Type,
		createDefaultConfig,
		createExtension,
		metadata.ExtensionStability)
}

// CreateDefaultConfig creates the default configuration for extension. Notice
// that the default configuration is expected to fail for this extension.
func createDefaultConfig() component.Config {
	return &Config{
		MemoryLimitMiB: 32,
		Timeout: 1 * time.Nanosecond,
	}
}

func createExtension(_ context.Context, set extension.CreateSettings, cfg component.Config) (extension.Extension, error) {
	return newBlockingMLExtension(cfg.(*Config), set.TelemetrySettings.Logger)
}