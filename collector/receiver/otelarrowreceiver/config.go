// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelarrowreceiver // import "github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver"

import (
	"errors"
	"fmt"

	"github.com/open-telemetry/otel-arrow/collector/compression/zstd"
	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/confmap"
)

const (
	// Confmap values.
	protoGRPC                = "protocols::grpc"
	protoArrowOldMemoryLimit = "protocols::arrow::memory_limit"
	protoArrowMemoryLimitMiB = "protocols::arrow::memory_limit_mib"
)

// Protocols is the configuration for the supported protocols.
type Protocols struct {
	GRPC  configgrpc.GRPCServerSettings `mapstructure:"grpc"`
	Arrow ArrowSettings                 `mapstructure:"arrow"`
}

// ArrowSettings support configuring the Arrow receiver.
type ArrowSettings struct {
	// DeprecatedMemoryLimit is deprecated, use MemoryLimitMiB.
	DeprecatedMemoryLimit uint64 `mapstructure:"memory_limit"`

	// MemoryLimitMiB is the size of a shared memory region used
	// by all Arrow streams, in MiB.  When too much load is
	// passing through, they will see ResourceExhausted errors.
	MemoryLimitMiB uint64 `mapstructure:"memory_limit_mib"`

	// Zstd settings apply to OTel-Arrow use of gRPC specifically.
	Zstd zstd.DecoderConfig `mapstructure:"zstd"`
}

// Config defines configuration for OTel Arrow receiver.
type Config struct {
	// Protocols is the configuration for gRPC and Arrow.
	Protocols `mapstructure:"protocols"`
}

var _ component.Config = (*Config)(nil)
var _ confmap.Unmarshaler = (*Config)(nil)

// Validate checks the receiver configuration is valid
func (cfg *Config) Validate() error {
	if err := cfg.Arrow.Validate(); err != nil {
		return err
	}
	return nil
}

func (cfg *ArrowSettings) Validate() error {
	if cfg.DeprecatedMemoryLimit != 0 && cfg.MemoryLimitMiB != 0 {
		return errors.New("memory_limit is deprecated, use only memory_limit_mib")
	}
	if cfg.DeprecatedMemoryLimit != 0 {
		// Round up
		cfg.MemoryLimitMiB = (cfg.DeprecatedMemoryLimit - 1 + 1<<20) >> 20
		cfg.DeprecatedMemoryLimit = 0
	}
	if err := cfg.Zstd.Validate(); err != nil {
		return fmt.Errorf("zstd decoder: invalid configuration: %w", err)
	}
	return nil
}

// Unmarshal a confmap.Conf into the config struct.
func (cfg *Config) Unmarshal(conf *confmap.Conf) error {
	// first load the config normally
	err := conf.Unmarshal(cfg, confmap.WithErrorUnused())
	if err != nil {
		return err
	}

	// Allow the deprecated field, when explicitly set, to unset
	// the new default value.
	if conf.IsSet(protoArrowOldMemoryLimit) && !conf.IsSet(protoArrowMemoryLimitMiB) {
		cfg.Arrow.MemoryLimitMiB = 0
	}

	return nil
}
