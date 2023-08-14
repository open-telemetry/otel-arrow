// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelarrowreceiver // import "github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver"

import (
	"errors"
	"fmt"
	"net/url"
	"path"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/config/confighttp"
	"go.opentelemetry.io/collector/confmap"
)

const (
	// Protocol values.
	protoGRPC = "protocols::grpc"
	protoHTTP = "protocols::http"
)

type httpServerSettings struct {
	*confighttp.HTTPServerSettings `mapstructure:",squash"`

	// The URL path to receive traces on. If omitted "/v1/traces" will be used.
	TracesURLPath string `mapstructure:"traces_url_path,omitempty"`

	// The URL path to receive metrics on. If omitted "/v1/metrics" will be used.
	MetricsURLPath string `mapstructure:"metrics_url_path,omitempty"`

	// The URL path to receive logs on. If omitted "/v1/logs" will be used.
	LogsURLPath string `mapstructure:"logs_url_path,omitempty"`
}

// Protocols is the configuration for the supported protocols.
type Protocols struct {
	GRPC  *configgrpc.GRPCServerSettings `mapstructure:"grpc"`
	HTTP  *httpServerSettings            `mapstructure:"http"`
	Arrow *ArrowSettings                 `mapstructure:"arrow"`
}

// ArrowSettings support disabling the Arrow receiver.
type ArrowSettings struct {
	Disabled bool `mapstructure:"disabled"`

	// DisableSeparateSignals when true prevents per-signal gRPC being served.
	DisableSeparateSignals bool `mapstructure:"disable_separate_signals"`

	// DisableMixedSignals when true prevents mixed-signal gRPC being served.
	DisableMixedSignals bool `mapstructure:"disable_mixed_signals"`
}

// Config defines configuration for OTLP receiver.
type Config struct {
	// Protocols is the configuration for the supported protocols, currently gRPC and HTTP (Proto and JSON).
	Protocols `mapstructure:"protocols"`
}

var _ component.Config = (*Config)(nil)
var _ confmap.Unmarshaler = (*Config)(nil)

// Validate checks the receiver configuration is valid
func (cfg *Config) Validate() error {
	if cfg.GRPC == nil && cfg.HTTP == nil {
		return errors.New("must specify at least one protocol when using the OTLP receiver")
	}
	if cfg.Arrow != nil && !cfg.Arrow.Disabled && cfg.GRPC == nil {
		return errors.New("must specify at gRPC protocol when using the OTLP+Arrow receiver")
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

	if !conf.IsSet(protoGRPC) {
		cfg.GRPC = nil
	}

	if !conf.IsSet(protoHTTP) {
		cfg.HTTP = nil
	} else {
		var err error

		if cfg.HTTP.TracesURLPath, err = sanitizeURLPath(cfg.HTTP.TracesURLPath); err != nil {
			return err
		}
		if cfg.HTTP.MetricsURLPath, err = sanitizeURLPath(cfg.HTTP.MetricsURLPath); err != nil {
			return err
		}
		if cfg.HTTP.LogsURLPath, err = sanitizeURLPath(cfg.HTTP.LogsURLPath); err != nil {
			return err
		}
	}

	return nil
}

// Verify signal URL path sanity
func sanitizeURLPath(urlPath string) (string, error) {
	u, err := url.Parse(urlPath)
	if err != nil {
		return "", fmt.Errorf("invalid HTTP URL path set for signal: %w", err)
	}

	if !path.IsAbs(u.Path) {
		u.Path = "/" + u.Path
	}
	return u.Path, nil
}
