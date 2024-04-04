// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package blockingmemorylimiterextension

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/multierr"
	"go.uber.org/zap"
	"google.golang.org/grpc"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/config/confignet"
	"go.opentelemetry.io/collector/config/configtls"
	"github.com/open-telemetry/otel-arrow/collector/config/configgrpc"
	"github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor/testdata"
	"go.opentelemetry.io/collector/component/componenttest"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

type mockMemoryLimiterExtension struct {
	*blockingMLExtension
	componentID component.ID
	blockCh chan struct{}
}

func (mockmle *mockMemoryLimiterExtension) UnaryInterceptorGenerator() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp any, err error) {
		inner := mockmle.blockingMLExtension.UnaryInterceptorGenerator()
		blockHandler := func(ctx context.Context, req any) (any, error) {
			<-mockmle.blockCh
			return handler(ctx, req)
		}
		return inner(ctx, req, info, blockHandler)
	}
}

type mockHost struct {
	component.Host
	ext map[component.ID]component.Component
}

func (nh *mockHost) GetExtensions() map[component.ID]component.Component {
	return nh.ext
}
type grpcTraceServer struct {
	ptraceotlp.UnimplementedGRPCServer
	recordedContext context.Context
}

func (gts *grpcTraceServer) Export(ctx context.Context, _ ptraceotlp.ExportRequest) (ptraceotlp.ExportResponse, error) {
	gts.recordedContext = ctx
	return ptraceotlp.NewExportResponse(), nil
}

func TestUnaryInterceptorMemoryLimited(t *testing.T) {
	ctx := context.Background()

	tests := []struct {
		name        string
		mlCfg       *Config
		numTraces   int
		numIter     int
		expectError bool
		sleep     time.Duration
	}{
		{
			name: "below memory limit",
			mlCfg: &Config{
				MemoryLimitMiB: 1,
				Timeout: 1 * time.Nanosecond,
			},
			sleep: 1 * time.Second,
			numTraces: 100,
			numIter: 1,
			expectError: false,
		},
		{
			name: "above memory limit",
			mlCfg: &Config{
				MemoryLimitMiB: 1,
				Timeout: 1 * time.Nanosecond,
			},
			sleep: 1 * time.Second,
			numTraces: 30000,
			numIter: 1,
			expectError: true,
		},
		{
			name: "multiple requests timeout short",
			mlCfg: &Config{
				MemoryLimitMiB: 1,
				Timeout: 1 * time.Nanosecond,
			},
			sleep: 1 * time.Second,
			numTraces: 5000,
			numIter: 2,
			expectError: true,
		},
		{
			name: "multiple requests timeout long",
			mlCfg: &Config{
				MemoryLimitMiB: 1,
				Timeout: 2 * time.Second,
			},
			sleep: 1 * time.Second,
			numTraces: 5000,
			numIter: 2,
			expectError: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ml, err := newBlockingMLExtension(tt.mlCfg, zap.NewNop())
			comID := component.NewID(component.MustNewType("memorylimiter"))
			mockML := &mockMemoryLimiterExtension{
				componentID: comID, 
				blockingMLExtension: ml,
				blockCh: make(chan struct{}),
			}
			assert.NoError(t, err)

			assert.NoError(t, ml.Start(ctx, &mockHost{}))
			
			gss := &configgrpc.ServerConfig{
				NetAddr: confignet.AddrConfig{
					Endpoint:  "localhost:0",
					Transport: "tcp",
				},
				MemoryLimiter: &comID,
			}
			extList := map[component.ID]component.Component{
				comID: mockML,
			}

			host := &mockHost{
				ext: extList,
			}

			srv, err := gss.ToServer(context.Background(), host, componenttest.NewNopTelemetrySettings())

			// found extension so finish setting up server and client to test interceptor.
			assert.NoError(t, err)
			mock := &grpcTraceServer{}

			ptraceotlp.RegisterGRPCServer(srv, mock)

			defer srv.Stop()

			l, err := gss.NetAddr.Listen(context.Background())
			require.NoError(t, err)

			go func() {
				_ = srv.Serve(l)
			}()

			// setup client
			gcs := &configgrpc.ClientConfig{
				Endpoint: l.Addr().String(),
				TLSSetting: configtls.TLSClientSetting{
					Insecure: true,
				},
			}

			tel, err := componenttest.SetupTelemetry(comID)
			require.NoError(t, err)
			defer func() {
				require.NoError(t, tel.Shutdown(context.Background()))
			}()

			grpcClientConn, errClient := gcs.ToClientConn(context.Background(), componenttest.NewNopHost(), tel.TelemetrySettings())
			require.NoError(t, errClient)
			defer func() { assert.NoError(t, grpcClientConn.Close()) }()

			basecl := ptraceotlp.NewGRPCClient(grpcClientConn)

			ctx, cancelFunc := context.WithTimeout(context.Background(), 20*time.Second)
			defer cancelFunc()

			var wg sync.WaitGroup
			var retErr error
			for i := 0; i < tt.numIter; i++ {
				wg.Add(1)
				go func() {
					traces := testdata.GenerateTraces(tt.numTraces)
					_, errResp := basecl.Export(ctx, ptraceotlp.NewExportRequestFromTraces(traces))

					retErr = multierr.Append(retErr, errResp)
					wg.Done()
				}()
			}

			// sleep so multiple requests have time to be blocked after calling sem.Acquire()
			time.Sleep(tt.sleep)
			close(mockML.blockCh)
			wg.Wait()

			if tt.expectError {
				assert.ErrorContains(t, retErr, "not enough memory available to process request")
			} else {
				assert.NoError(t, retErr)
			}

			assert.NoError(t, ml.Shutdown(ctx))
		})
	}
}