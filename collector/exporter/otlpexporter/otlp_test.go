// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package otlpexporter

import (
	"context"
	"net"
	"path/filepath"
	"runtime"
	"sync"
	"testing"
	"time"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrowpbMock "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1/mock"
	arrowRecord "github.com/f5/otel-arrow-adapter/pkg/otel/arrow_record"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/atomic"
	"go.uber.org/zap/zaptest"
	"google.golang.org/genproto/googleapis/rpc/errdetails"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"

	"go.opentelemetry.io/collector/component/componenttest"
	"go.opentelemetry.io/collector/config/configgrpc"
	"go.opentelemetry.io/collector/config/configtls"
	"go.opentelemetry.io/collector/exporter"
	"go.opentelemetry.io/collector/exporter/exportertest"
	"github.com/f5/otel-arrow-adapter/collector/exporter/otlpexporter/internal/arrow"
	"github.com/f5/otel-arrow-adapter/collector/internal/testdata"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
)

type mockReceiver struct {
	srv          *grpc.Server
	ln           net.Listener
	requestCount *atomic.Int32
	totalItems   *atomic.Int32
	mux          sync.Mutex
	metadata     metadata.MD
	exportError  error
}

func (r *mockReceiver) getMetadata() metadata.MD {
	r.mux.Lock()
	defer r.mux.Unlock()
	return r.metadata
}

func (r *mockReceiver) setExportError(err error) {
	r.mux.Lock()
	defer r.mux.Unlock()
	r.exportError = err
}

type mockTracesReceiver struct {
	mockReceiver
	lastRequest ptrace.Traces
}

func (r *mockTracesReceiver) Export(ctx context.Context, req ptraceotlp.ExportRequest) (ptraceotlp.ExportResponse, error) {
	r.requestCount.Inc()
	td := req.Traces()
	r.totalItems.Add(int32(td.SpanCount()))
	r.mux.Lock()
	defer r.mux.Unlock()
	r.lastRequest = td
	r.metadata, _ = metadata.FromIncomingContext(ctx)
	return ptraceotlp.NewExportResponse(), r.exportError
}

func (r *mockTracesReceiver) getLastRequest() ptrace.Traces {
	r.mux.Lock()
	defer r.mux.Unlock()
	return r.lastRequest
}

func otlpTracesReceiverOnGRPCServer(ln net.Listener, useTLS bool) (*mockTracesReceiver, error) {
	sopts := []grpc.ServerOption{}

	if useTLS {
		_, currentFile, _, _ := runtime.Caller(0)
		basepath := filepath.Dir(currentFile)
		certpath := filepath.Join(basepath, filepath.Join("testdata", "test_cert.pem"))
		keypath := filepath.Join(basepath, filepath.Join("testdata", "test_key.pem"))

		creds, err := credentials.NewServerTLSFromFile(certpath, keypath)
		if err != nil {
			return nil, err
		}
		sopts = append(sopts, grpc.Creds(creds))
	}

	rcv := &mockTracesReceiver{
		mockReceiver: mockReceiver{
			srv:          grpc.NewServer(sopts...),
			ln:           ln,
			requestCount: atomic.NewInt32(0),
			totalItems:   atomic.NewInt32(0),
		},
	}

	ptraceotlp.RegisterGRPCServer(rcv.srv, rcv)

	return rcv, nil
}

func (r *mockTracesReceiver) start() {
	go func() {
		_ = r.srv.Serve(r.ln)
	}()
}

type mockLogsReceiver struct {
	mockReceiver
	lastRequest plog.Logs
}

func (r *mockLogsReceiver) Export(ctx context.Context, req plogotlp.ExportRequest) (plogotlp.ExportResponse, error) {
	r.requestCount.Inc()
	ld := req.Logs()
	r.totalItems.Add(int32(ld.LogRecordCount()))
	r.mux.Lock()
	defer r.mux.Unlock()
	r.lastRequest = ld
	r.metadata, _ = metadata.FromIncomingContext(ctx)
	return plogotlp.NewExportResponse(), r.exportError
}

func (r *mockLogsReceiver) getLastRequest() plog.Logs {
	r.mux.Lock()
	defer r.mux.Unlock()
	return r.lastRequest
}

func otlpLogsReceiverOnGRPCServer(ln net.Listener) *mockLogsReceiver {
	rcv := &mockLogsReceiver{
		mockReceiver: mockReceiver{
			srv:          grpc.NewServer(),
			requestCount: atomic.NewInt32(0),
			totalItems:   atomic.NewInt32(0),
		},
	}

	// Now run it as a gRPC server
	plogotlp.RegisterGRPCServer(rcv.srv, rcv)
	go func() {
		_ = rcv.srv.Serve(ln)
	}()

	return rcv
}

type mockMetricsReceiver struct {
	mockReceiver
	lastRequest pmetric.Metrics
}

func (r *mockMetricsReceiver) Export(ctx context.Context, req pmetricotlp.ExportRequest) (pmetricotlp.ExportResponse, error) {
	md := req.Metrics()
	r.requestCount.Inc()
	r.totalItems.Add(int32(md.DataPointCount()))
	r.mux.Lock()
	defer r.mux.Unlock()
	r.lastRequest = md
	r.metadata, _ = metadata.FromIncomingContext(ctx)
	return pmetricotlp.NewExportResponse(), r.exportError
}

func (r *mockMetricsReceiver) getLastRequest() pmetric.Metrics {
	r.mux.Lock()
	defer r.mux.Unlock()
	return r.lastRequest
}

func otlpMetricsReceiverOnGRPCServer(ln net.Listener) *mockMetricsReceiver {
	rcv := &mockMetricsReceiver{
		mockReceiver: mockReceiver{
			srv:          grpc.NewServer(),
			requestCount: atomic.NewInt32(0),
			totalItems:   atomic.NewInt32(0),
		},
	}

	// Now run it as a gRPC server
	pmetricotlp.RegisterGRPCServer(rcv.srv, rcv)
	go func() {
		_ = rcv.srv.Serve(ln)
	}()

	return rcv
}

func TestSendTraces(t *testing.T) {
	// Start an OTLP-compatible receiver.
	ln, err := net.Listen("tcp", "localhost:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)
	rcv, _ := otlpTracesReceiverOnGRPCServer(ln, false)
	rcv.start()
	// Also closes the connection.
	defer rcv.srv.GracefulStop()

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
		Headers: map[string]string{
			"header": "header-value",
		},
	}
	set := exportertest.NewNopCreateSettings()
	set.BuildInfo.Description = "Collector"
	set.BuildInfo.Version = "1.2.3test"
	exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)

	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()
	assert.NoError(t, exp.Start(context.Background(), host))

	// Ensure that initially there is no data in the receiver.
	assert.EqualValues(t, 0, rcv.requestCount.Load())

	// Send empty trace.
	td := ptrace.NewTraces()
	assert.NoError(t, exp.ConsumeTraces(context.Background(), td))

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 0
	}, 10*time.Second, 5*time.Millisecond)

	// Ensure it was received empty.
	assert.EqualValues(t, 0, rcv.totalItems.Load())

	// A trace with 2 spans.
	td = testdata.GenerateTraces(2)

	err = exp.ConsumeTraces(context.Background(), td)
	assert.NoError(t, err)

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 1
	}, 10*time.Second, 5*time.Millisecond)

	expectedHeader := []string{"header-value"}

	// Verify received span.
	assert.EqualValues(t, 2, rcv.totalItems.Load())
	assert.EqualValues(t, 2, rcv.requestCount.Load())
	assert.EqualValues(t, td, rcv.getLastRequest())

	md := rcv.getMetadata()
	require.EqualValues(t, md.Get("header"), expectedHeader)
	require.Equal(t, len(md.Get("User-Agent")), 1)
	require.Contains(t, md.Get("User-Agent")[0], "Collector/1.2.3test")
}

func TestSendTracesWhenEndpointHasHttpScheme(t *testing.T) {
	tests := []struct {
		name               string
		useTLS             bool
		scheme             string
		gRPCClientSettings configgrpc.GRPCClientSettings
	}{
		{
			name:               "Use https scheme",
			useTLS:             true,
			scheme:             "https://",
			gRPCClientSettings: configgrpc.GRPCClientSettings{},
		},
		{
			name:   "Use http scheme",
			useTLS: false,
			scheme: "http://",
			gRPCClientSettings: configgrpc.GRPCClientSettings{
				TLSSetting: configtls.TLSClientSetting{
					Insecure: true,
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			// Start an OTLP-compatible receiver.
			ln, err := net.Listen("tcp", "localhost:")
			require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)
			rcv, err := otlpTracesReceiverOnGRPCServer(ln, test.useTLS)
			rcv.start()
			require.NoError(t, err, "Failed to start mock OTLP receiver")
			// Also closes the connection.
			defer rcv.srv.GracefulStop()

			// Start an OTLP exporter and point to the receiver.
			factory := NewFactory()
			cfg := factory.CreateDefaultConfig().(*Config)
			cfg.GRPCClientSettings = test.gRPCClientSettings
			cfg.GRPCClientSettings.Endpoint = test.scheme + ln.Addr().String()
			if test.useTLS {
				cfg.GRPCClientSettings.TLSSetting.InsecureSkipVerify = true
			}
			set := exportertest.NewNopCreateSettings()
			exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
			require.NoError(t, err)
			require.NotNil(t, exp)

			defer func() {
				assert.NoError(t, exp.Shutdown(context.Background()))
			}()

			host := componenttest.NewNopHost()
			assert.NoError(t, exp.Start(context.Background(), host))

			// Ensure that initially there is no data in the receiver.
			assert.EqualValues(t, 0, rcv.requestCount.Load())

			// Send empty trace.
			td := ptrace.NewTraces()
			assert.NoError(t, exp.ConsumeTraces(context.Background(), td))

			// Wait until it is received.
			assert.Eventually(t, func() bool {
				return rcv.requestCount.Load() > 0
			}, 10*time.Second, 5*time.Millisecond)

			// Ensure it was received empty.
			assert.EqualValues(t, 0, rcv.totalItems.Load())
		})
	}
}

func TestSendMetrics(t *testing.T) {
	// Start an OTLP-compatible receiver.
	ln, err := net.Listen("tcp", "localhost:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)
	rcv := otlpMetricsReceiverOnGRPCServer(ln)
	// Also closes the connection.
	defer rcv.srv.GracefulStop()

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
		Headers: map[string]string{
			"header": "header-value",
		},
	}
	set := exportertest.NewNopCreateSettings()
	set.BuildInfo.Description = "Collector"
	set.BuildInfo.Version = "1.2.3test"
	exp, err := factory.CreateMetricsExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)
	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()

	assert.NoError(t, exp.Start(context.Background(), host))

	// Ensure that initially there is no data in the receiver.
	assert.EqualValues(t, 0, rcv.requestCount.Load())

	// Send empty metric.
	md := pmetric.NewMetrics()
	assert.NoError(t, exp.ConsumeMetrics(context.Background(), md))

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 0
	}, 10*time.Second, 5*time.Millisecond)

	// Ensure it was received empty.
	assert.EqualValues(t, 0, rcv.totalItems.Load())

	// Send two metrics.
	md = testdata.GenerateMetrics(2)

	err = exp.ConsumeMetrics(context.Background(), md)
	assert.NoError(t, err)

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 1
	}, 10*time.Second, 5*time.Millisecond)

	expectedHeader := []string{"header-value"}

	// Verify received metrics.
	assert.EqualValues(t, 2, rcv.requestCount.Load())
	assert.EqualValues(t, 4, rcv.totalItems.Load())
	assert.EqualValues(t, md, rcv.getLastRequest())

	mdata := rcv.getMetadata()
	require.EqualValues(t, mdata.Get("header"), expectedHeader)
	require.Equal(t, len(mdata.Get("User-Agent")), 1)
	require.Contains(t, mdata.Get("User-Agent")[0], "Collector/1.2.3test")
}

func TestSendTraceDataServerDownAndUp(t *testing.T) {
	// Find the addr, but don't start the server.
	ln, err := net.Listen("tcp", "localhost:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	// Disable queuing to ensure that we execute the request when calling ConsumeTraces
	// otherwise we will not see the error.
	cfg.QueueSettings.Enabled = false
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
		// Need to wait for every request blocking until either request timeouts or succeed.
		// Do not rely on external retry logic here, if that is intended set InitialInterval to 100ms.
		WaitForReady: true,
	}
	set := exportertest.NewNopCreateSettings()
	exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)
	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()

	assert.NoError(t, exp.Start(context.Background(), host))

	// A trace with 2 spans.
	td := testdata.GenerateTraces(2)
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	assert.Error(t, exp.ConsumeTraces(ctx, td))
	assert.EqualValues(t, context.DeadlineExceeded, ctx.Err())
	cancel()

	ctx, cancel = context.WithTimeout(context.Background(), 1*time.Second)
	assert.Error(t, exp.ConsumeTraces(ctx, td))
	assert.EqualValues(t, context.DeadlineExceeded, ctx.Err())
	cancel()

	startServerAndMakeRequest(t, exp, td, ln)

	ctx, cancel = context.WithTimeout(context.Background(), 1*time.Second)
	assert.Error(t, exp.ConsumeTraces(ctx, td))
	assert.EqualValues(t, context.DeadlineExceeded, ctx.Err())
	cancel()

	// First call to startServerAndMakeRequest closed the connection. There is a race condition here that the
	// port may be reused, if this gets flaky rethink what to do.
	ln, err = net.Listen("tcp", ln.Addr().String())
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)
	startServerAndMakeRequest(t, exp, td, ln)

	ctx, cancel = context.WithTimeout(context.Background(), 1*time.Second)
	assert.Error(t, exp.ConsumeTraces(ctx, td))
	assert.EqualValues(t, context.DeadlineExceeded, ctx.Err())
	cancel()
}

func TestSendTraceDataServerStartWhileRequest(t *testing.T) {
	// Find the addr, but don't start the server.
	ln, err := net.Listen("tcp", "localhost:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
	}
	set := exportertest.NewNopCreateSettings()
	exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)
	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()

	assert.NoError(t, exp.Start(context.Background(), host))

	// A trace with 2 spans.
	td := testdata.GenerateTraces(2)
	done := make(chan bool, 1)
	defer close(done)
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	go func() {
		assert.NoError(t, exp.ConsumeTraces(ctx, td))
		done <- true
	}()

	time.Sleep(2 * time.Second)
	rcv, _ := otlpTracesReceiverOnGRPCServer(ln, false)
	rcv.start()
	defer rcv.srv.GracefulStop()
	// Wait until one of the conditions below triggers.
	select {
	case <-ctx.Done():
		t.Fail()
	case <-done:
		assert.NoError(t, ctx.Err())
	}
	cancel()
}

func TestSendTracesOnResourceExhaustion(t *testing.T) {
	ln, err := net.Listen("tcp", "localhost:")
	require.NoError(t, err)
	rcv, _ := otlpTracesReceiverOnGRPCServer(ln, false)
	rcv.setExportError(status.Error(codes.ResourceExhausted, "resource exhausted"))
	rcv.start()
	defer rcv.srv.GracefulStop()

	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.RetrySettings.InitialInterval = 0
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
	}
	set := exportertest.NewNopCreateSettings()
	exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)

	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()
	assert.NoError(t, exp.Start(context.Background(), host))

	assert.EqualValues(t, 0, rcv.requestCount.Load())

	td := ptrace.NewTraces()
	assert.NoError(t, exp.ConsumeTraces(context.Background(), td))

	assert.Never(t, func() bool {
		return rcv.requestCount.Load() > 1
	}, 1*time.Second, 5*time.Millisecond, "Should not retry if RetryInfo is not included into status details by the server.")

	rcv.requestCount.Swap(0)

	st := status.New(codes.ResourceExhausted, "resource exhausted")
	st, _ = st.WithDetails(&errdetails.RetryInfo{
		RetryDelay: durationpb.New(100 * time.Millisecond),
	})
	rcv.setExportError(st.Err())

	assert.NoError(t, exp.ConsumeTraces(context.Background(), td))

	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 1
	}, 10*time.Second, 5*time.Millisecond, "Should retry if RetryInfo is included into status details by the server.")
}

func startServerAndMakeRequest(t *testing.T, exp exporter.Traces, td ptrace.Traces, ln net.Listener) {
	rcv, _ := otlpTracesReceiverOnGRPCServer(ln, false)
	rcv.start()
	defer rcv.srv.GracefulStop()
	// Ensure that initially there is no data in the receiver.
	assert.EqualValues(t, 0, rcv.requestCount.Load())

	// Clone the request and store as expected.
	expectedData := ptrace.NewTraces()
	td.CopyTo(expectedData)

	// Resend the request, this should succeed.
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	assert.NoError(t, exp.ConsumeTraces(ctx, td))
	cancel()

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 0
	}, 10*time.Second, 5*time.Millisecond)

	// Verify received span.
	assert.EqualValues(t, 2, rcv.totalItems.Load())
	assert.EqualValues(t, expectedData, rcv.getLastRequest())
}

func TestSendLogData(t *testing.T) {
	// Start an OTLP-compatible receiver.
	ln, err := net.Listen("tcp", "localhost:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)
	rcv := otlpLogsReceiverOnGRPCServer(ln)
	// Also closes the connection.
	defer rcv.srv.GracefulStop()

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
	}
	set := exportertest.NewNopCreateSettings()
	set.BuildInfo.Description = "Collector"
	set.BuildInfo.Version = "1.2.3test"
	exp, err := factory.CreateLogsExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)
	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()

	assert.NoError(t, exp.Start(context.Background(), host))

	// Ensure that initially there is no data in the receiver.
	assert.EqualValues(t, 0, rcv.requestCount.Load())

	// Send empty request.
	ld := plog.NewLogs()
	assert.NoError(t, exp.ConsumeLogs(context.Background(), ld))

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 0
	}, 10*time.Second, 5*time.Millisecond)

	// Ensure it was received empty.
	assert.EqualValues(t, 0, rcv.totalItems.Load())

	// A request with 2 log entries.
	ld = testdata.GenerateLogs(2)

	err = exp.ConsumeLogs(context.Background(), ld)
	assert.NoError(t, err)

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 1
	}, 10*time.Second, 5*time.Millisecond)

	// Verify received logs.
	assert.EqualValues(t, 2, rcv.requestCount.Load())
	assert.EqualValues(t, 2, rcv.totalItems.Load())
	assert.EqualValues(t, ld, rcv.getLastRequest())

	md := rcv.getMetadata()
	require.Equal(t, len(md.Get("User-Agent")), 1)
	require.Contains(t, md.Get("User-Agent")[0], "Collector/1.2.3test")
}

// TestSendArrowTracesNotSupported tests a successful OTLP export w/
// and without Arrow, w/ WaitForReady and without.
func TestSendArrowTracesNotSupported(t *testing.T) {
	// With arrow unsupported
	t.Run("Service Unavailable", func(t *testing.T) {
		t.Run("WaitForReady true", func(t *testing.T) {
			testSendArrowTraces(t, true, false)
		})
		t.Run("WaitForReady false", func(t *testing.T) {
			testSendArrowTraces(t, false, false)
		})
	})
	// With arrow supported
	t.Run("Service Available", func(t *testing.T) {
		t.Run("WaitForReady true", func(t *testing.T) {
			testSendArrowTraces(t, true, true)
		})
		t.Run("WaitForReady false", func(t *testing.T) {
			testSendArrowTraces(t, false, true)
		})
	})
}

func testSendArrowTraces(t *testing.T, clientWaitForReady, streamServiceAvailable bool) {
	// Start an OTLP-compatible receiver.
	ln, err := net.Listen("tcp", "127.0.0.1:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
		WaitForReady: clientWaitForReady,
	}
	// Arrow client is enabled, but the server doesn't support it.
	cfg.Arrow = &arrow.Settings{
		Enabled:    true,
		NumStreams: 1,
	}

	set := exportertest.NewNopCreateSettings()
	set.TelemetrySettings.Logger = zaptest.NewLogger(t)
	exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)

	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()
	assert.NoError(t, exp.Start(context.Background(), host))

	rcv, _ := otlpTracesReceiverOnGRPCServer(ln, false)
	if streamServiceAvailable {
		rcv.startStreamMockArrowTraces(t, okStatusFor)
	}

	// Delay the server start, slightly.
	go func() {
		time.Sleep(100 * time.Millisecond)
		rcv.start()
	}()

	// Send two trace items.
	td := testdata.GenerateTraces(2)
	err = exp.ConsumeTraces(context.Background(), td)
	assert.NoError(t, err)

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 0
	}, 10*time.Second, 5*time.Millisecond)

	// Verify two items, one request received.
	assert.EqualValues(t, int32(2), rcv.totalItems.Load())
	assert.EqualValues(t, int32(1), rcv.requestCount.Load())
	assert.EqualValues(t, td, rcv.getLastRequest())
}

func okStatusFor(id string) *arrowpb.StatusMessage {
	return &arrowpb.StatusMessage{
		BatchId:    id,
		StatusCode: arrowpb.StatusCode_OK,
	}
}

func failedStatusFor(id string) *arrowpb.StatusMessage {
	return &arrowpb.StatusMessage{
		BatchId:      id,
		StatusCode:   arrowpb.StatusCode_ERROR,
		ErrorCode:    arrowpb.ErrorCode_INVALID_ARGUMENT,
		ErrorMessage: "test failed",
	}
}

func (r *mockTracesReceiver) startStreamMockArrowTraces(t *testing.T, statusFor func(string) *arrowpb.StatusMessage) {
	ctrl := gomock.NewController(t)

	type binding struct {
		arrowpb.UnsafeArrowStreamServiceServer
		*arrowpbMock.MockArrowStreamServiceServer
	}
	svc := arrowpbMock.NewMockArrowStreamServiceServer(ctrl)

	arrowpb.RegisterArrowStreamServiceServer(r.srv, binding{
		MockArrowStreamServiceServer: svc,
	})
	svc.EXPECT().ArrowStream(gomock.Any()).Times(1).DoAndReturn(func(realSrv arrowpb.ArrowStreamService_ArrowStreamServer) error {
		ctx := context.Background()
		consumer := arrowRecord.NewConsumer()
		for {
			records, err := realSrv.Recv()
			if status, ok := status.FromError(err); ok && status.Code() == codes.Canceled {
				break
			}
			require.NoError(t, err)
			got, err := consumer.TracesFrom(records)
			require.NoError(t, err)
			for _, traces := range got {
				_, err := r.Export(ctx, ptraceotlp.NewExportRequestFromTraces(traces))
				require.NoError(t, err)
			}
			require.NoError(t, realSrv.Send(&arrowpb.BatchStatus{
				Statuses: []*arrowpb.StatusMessage{
					statusFor(records.BatchId),
				},
			}))
		}
		return nil
	})

}

func TestSendArrowFailedTraces(t *testing.T) {
	// Start an OTLP-compatible receiver.
	ln, err := net.Listen("tcp", "127.0.0.1:")
	require.NoError(t, err, "Failed to find an available address to run the gRPC server: %v", err)

	// Start an OTLP exporter and point to the receiver.
	factory := NewFactory()
	cfg := factory.CreateDefaultConfig().(*Config)
	cfg.GRPCClientSettings = configgrpc.GRPCClientSettings{
		Endpoint: ln.Addr().String(),
		TLSSetting: configtls.TLSClientSetting{
			Insecure: true,
		},
		WaitForReady: true,
	}
	// Arrow client is enabled, but the server doesn't support it.
	cfg.Arrow = &arrow.Settings{
		Enabled:    true,
		NumStreams: 1,
	}
	cfg.QueueSettings.Enabled = false

	set := exportertest.NewNopCreateSettings()
	set.TelemetrySettings.Logger = zaptest.NewLogger(t)
	exp, err := factory.CreateTracesExporter(context.Background(), set, cfg)
	require.NoError(t, err)
	require.NotNil(t, exp)

	defer func() {
		assert.NoError(t, exp.Shutdown(context.Background()))
	}()

	host := componenttest.NewNopHost()
	assert.NoError(t, exp.Start(context.Background(), host))

	rcv, _ := otlpTracesReceiverOnGRPCServer(ln, false)
	rcv.startStreamMockArrowTraces(t, failedStatusFor)

	// Delay the server start, slightly.
	go func() {
		time.Sleep(100 * time.Millisecond)
		rcv.start()
	}()

	// Send two trace items.
	td := testdata.GenerateTraces(2)
	err = exp.ConsumeTraces(context.Background(), td)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "test failed")

	// Wait until it is received.
	assert.Eventually(t, func() bool {
		return rcv.requestCount.Load() > 0
	}, 10*time.Second, 5*time.Millisecond)

	// Verify two items, one request received.
	assert.EqualValues(t, int32(2), rcv.totalItems.Load())
	assert.EqualValues(t, int32(1), rcv.requestCount.Load())
	assert.EqualValues(t, td, rcv.getLastRequest())
}
