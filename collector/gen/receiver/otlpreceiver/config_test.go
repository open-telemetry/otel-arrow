// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otlpreceiver

import (
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/config/confighttp"
	"go.opentelemetry.io/collector/config/confignet"
	"go.opentelemetry.io/collector/config/configtls"
	"go.opentelemetry.io/collector/confmap"
	"go.opentelemetry.io/collector/confmap/confmaptest"
)

func TestUnmarshalDefaultConfig(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "default.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))
	assert.Equal(t, factory.CreateDefaultConfig(), cfg)
}

func TestUnmarshalConfigOnlyGRPC(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "only_grpc.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))

	defaultOnlyGRPC := factory.CreateDefaultConfig().(*Config)
	defaultOnlyGRPC.HTTP = nil
	assert.Equal(t, defaultOnlyGRPC, cfg)
}

func TestUnmarshalConfigOnlyHTTP(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "only_http.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))

	defaultOnlyHTTP := factory.CreateDefaultConfig().(*Config)
	defaultOnlyHTTP.GRPC = nil
	assert.Equal(t, defaultOnlyHTTP, cfg)
}

func TestUnmarshalConfigOnlyHTTPNull(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "only_http_null.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))

	defaultOnlyHTTP := factory.CreateDefaultConfig().(*Config)
	defaultOnlyHTTP.GRPC = nil
	assert.Equal(t, defaultOnlyHTTP, cfg)
}

func TestUnmarshalConfigOnlyHTTPEmptyMap(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "only_http_empty_map.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))

	defaultOnlyHTTP := factory.CreateDefaultConfig().(*Config)
	defaultOnlyHTTP.GRPC = nil
	assert.Equal(t, defaultOnlyHTTP, cfg)
}

func TestUnmarshalConfig(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "config.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))
	assert.Equal(t,
		&Config{
			Protocols: Protocols{
				GRPC: &configgrpc.GRPCServerSettings{
					NetAddr: confignet.NetAddr{
						Endpoint:  "0.0.0.0:4317",
						Transport: "tcp",
					},
					TLSSetting: &configtls.TLSServerSetting{
						TLSSetting: configtls.TLSSetting{
							CertFile: "test.crt",
							KeyFile:  "test.key",
						},
					},
					MaxRecvMsgSizeMiB:    32,
					MaxConcurrentStreams: 16,
					ReadBufferSize:       1024,
					WriteBufferSize:      1024,
					Keepalive: &configgrpc.KeepaliveServerConfig{
						ServerParameters: &configgrpc.KeepaliveServerParameters{
							MaxConnectionIdle:     11 * time.Second,
							MaxConnectionAge:      12 * time.Second,
							MaxConnectionAgeGrace: 13 * time.Second,
							Time:                  30 * time.Second,
							Timeout:               5 * time.Second,
						},
						EnforcementPolicy: &configgrpc.KeepaliveEnforcementPolicy{
							MinTime:             10 * time.Second,
							PermitWithoutStream: true,
						},
					},
				},
				HTTP: &httpServerSettings{
					HTTPServerSettings: &confighttp.HTTPServerSettings{
						Endpoint: "0.0.0.0:4318",
						TLSSetting: &configtls.TLSServerSetting{
							TLSSetting: configtls.TLSSetting{
								CertFile: "test.crt",
								KeyFile:  "test.key",
							},
						},
						CORS: &confighttp.CORSSettings{
							AllowedOrigins: []string{"https://*.test.com", "https://test.com"},
							MaxAge:         7200,
						},
					},
					TracesURLPath:  "/traces",
					MetricsURLPath: "/v2/metrics",
					LogsURLPath:    "/log/ingest",
				},
				Arrow: &ArrowSettings{
					Disabled: false,
				},
			},
		}, cfg)

}

func TestUnmarshalConfigUnix(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "uds.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))
	assert.Equal(t,
		&Config{
			Protocols: Protocols{
				GRPC: &configgrpc.GRPCServerSettings{
					NetAddr: confignet.NetAddr{
						Endpoint:  "/tmp/grpc_otlp.sock",
						Transport: "unix",
					},
					ReadBufferSize: 512 * 1024,
				},
				HTTP: &httpServerSettings{
					HTTPServerSettings: &confighttp.HTTPServerSettings{
						Endpoint: "/tmp/http_otlp.sock",
					},
					TracesURLPath:  defaultTracesURLPath,
					MetricsURLPath: defaultMetricsURLPath,
					LogsURLPath:    defaultLogsURLPath,
				},
				Arrow: &ArrowSettings{},
			},
		}, cfg)
}

func TestUnmarshalConfigTypoDefaultProtocol(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "typo_default_proto_config.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.EqualError(t, component.UnmarshalConfig(cm, cfg), "1 error(s) decoding:\n\n* 'protocols' has invalid keys: htttp")
}

func TestUnmarshalConfigInvalidProtocol(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "bad_proto_config.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.EqualError(t, component.UnmarshalConfig(cm, cfg), "1 error(s) decoding:\n\n* 'protocols' has invalid keys: thrift")
}

func TestUnmarshalConfigEmptyProtocols(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "bad_no_proto_config.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))
	assert.EqualError(t, component.ValidateConfig(cfg), "must specify at least one protocol when using the OTLP receiver")
}

func TestUnmarshalConfigArrowWithoutGRPC(t *testing.T) {
	cm, err := confmaptest.LoadConf(filepath.Join("testdata", "arrow_without_grpc.yaml"))
	require.NoError(t, err)
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(cm, cfg))
	assert.EqualError(t, component.ValidateConfig(cfg), "must specify at gRPC protocol when using the OTLP+Arrow receiver")
}

func TestUnmarshalConfigInvalidSignalPath(t *testing.T) {
	tests := []struct {
		name       string
		testDataFn string
	}{
		{
			name:       "Invalid traces URL path",
			testDataFn: "invalid_traces_path.yaml",
		},
		{
			name:       "Invalid metrics URL path",
			testDataFn: "invalid_metrics_path.yaml",
		},
		{
			name:       "Invalid logs URL path",
			testDataFn: "invalid_logs_path.yaml",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			cm, err := confmaptest.LoadConf(filepath.Join("testdata", test.testDataFn))
			require.NoError(t, err)
			factory := NewFactory()
			cfg := factory.CreateDefaultConfig()
			assert.EqualError(t, component.UnmarshalConfig(cm, cfg), "invalid HTTP URL path set for signal: parse \":invalid\": missing protocol scheme")
		})
	}
}

func TestUnmarshalConfigEmpty(t *testing.T) {
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig()
	assert.NoError(t, component.UnmarshalConfig(confmap.New(), cfg))
	assert.EqualError(t, component.ValidateConfig(cfg), "must specify at least one protocol when using the OTLP receiver")
}
