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

package arrow_record

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
	"google.golang.org/protobuf/proto"

	arrowpb "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/config"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// Fuzz-tests the consumer on a sequence of two OTLP protobuf inputs.
// Note: This is basically a fuzz-tester of the Arrow IPC library.
// TODO: Note we can add a memory allocator to limit some of the memory
// allocated by Arrow, but not all of it.
func FuzzConsumerTraces(f *testing.F) {
	const numSeeds = 5

	ent := datagen.NewTestEntropy(12345)

	for i := 0; i < numSeeds; i++ {
		func() {
			dg := datagen.NewTracesGenerator(
				ent,
				ent.NewStandardResourceAttributes(),
				ent.NewStandardInstrumentationScopes(),
			)
			traces1 := dg.Generate(i+1, time.Minute)
			traces2 := dg.Generate(i+1, time.Minute)

			producer := NewProducer()
			defer func() {
				if err := producer.Close(); err != nil {
					f.Error("unexpected fail", err)
				}
			}()

			batch1, err1 := producer.BatchArrowRecordsFromTraces(traces1)
			require.NoError(f, err1)
			batch2, err2 := producer.BatchArrowRecordsFromTraces(traces2)
			require.NoError(f, err2)

			b1b, err1 := proto.Marshal(batch1)
			b2b, err2 := proto.Marshal(batch2)
			require.NoError(f, err1)
			require.NoError(f, err2)

			f.Add(b1b, b2b)
		}()
	}

	f.Fuzz(func(t *testing.T, b1, b2 []byte) {
		var b1b arrowpb.BatchArrowRecords
		var b2b arrowpb.BatchArrowRecords

		if err := proto.Unmarshal(b1, &b1b); err != nil {
			return
		}
		if err := proto.Unmarshal(b2, &b1b); err != nil {
			return
		}

		consumer := NewConsumer()

		if _, err := consumer.TracesFrom(&b1b); err != nil {
			return
		}
		if _, err := consumer.TracesFrom(&b2b); err != nil {
			return
		}
	})
}

// Fuzz-tests the producer on a sequence of two OTLP protobuf inputs.
func FuzzProducerTraces2(f *testing.F) {
	const numSeeds = 5

	ent := datagen.NewTestEntropy(12345)

	for i := 0; i < numSeeds; i++ {
		dg := datagen.NewTracesGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		)
		traces1 := dg.Generate(i+1, time.Minute)
		traces2 := dg.Generate(i+1, time.Minute)

		b1b, err1 := ptraceotlp.NewExportRequestFromTraces(traces1).MarshalProto()
		if err1 != nil {
			panic(err1)
		}
		b2b, err2 := ptraceotlp.NewExportRequestFromTraces(traces2).MarshalProto()
		if err2 != nil {
			panic(err2)
		}

		f.Add(b1b, b2b)
	}

	f.Fuzz(func(t *testing.T, b1, b2 []byte) {
		e1 := ptraceotlp.NewExportRequest()
		e2 := ptraceotlp.NewExportRequest()

		if err := e1.UnmarshalProto(b1); err != nil {
			return
		}
		if err := e2.UnmarshalProto(b2); err != nil {
			return
		}

		producer := NewProducer()
		defer func() {
			if err := producer.Close(); err != nil {
				t.Error("unexpected fail", err)
			}
		}()

		if _, err := producer.BatchArrowRecordsFromTraces(e1.Traces()); err != nil {
			return
		}
		if _, err := producer.BatchArrowRecordsFromTraces(e2.Traces()); err != nil {
			return
		}
	})
}

// Fuzz-tests the producer on the second in sequence of two OTLP protobuf inputs.
func FuzzProducerTraces1(f *testing.F) {
	const numSeeds = 5

	ent := datagen.NewTestEntropy(12345)

	dg := datagen.NewTracesGenerator(
		ent,
		ent.NewStandardResourceAttributes(),
		ent.NewStandardInstrumentationScopes(),
	)
	traces1 := dg.Generate(1, time.Minute)

	for i := 0; i < numSeeds; i++ {
		traces2 := dg.Generate(11, time.Minute)

		b2b, err2 := ptraceotlp.NewExportRequestFromTraces(traces2).MarshalProto()
		if err2 != nil {
			panic(err2)
		}

		f.Add(b2b)
	}

	f.Fuzz(func(t *testing.T, b2 []byte) {
		e2 := ptraceotlp.NewExportRequest()

		if err := e2.UnmarshalProto(b2); err != nil {
			return
		}

		producer := NewProducer()
		defer func() {
			if err := producer.Close(); err != nil {
				t.Error("unexpected fail", err)
			}
		}()

		if _, err := producer.BatchArrowRecordsFromTraces(traces1); err != nil {
			t.Error("unexpected fail", err)
		}
		if _, err := producer.BatchArrowRecordsFromTraces(e2.Traces()); err != nil {
			return
		}
	})
}

func TestProducerConsumerTraces(t *testing.T) {
	ent := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	stdTesting := assert.NewStdUnitTest(t)

	for idx, dg := range []*datagen.TraceGenerator{
		datagen.NewTracesGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		),
		datagen.NewTracesGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewSingleInstrumentationScopes(),
		),
		datagen.NewTracesGenerator(
			ent,
			ent.NewSingleResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		),
	} {
		t.Run(fmt.Sprint("traces/", idx), func(t *testing.T) {
			traces := dg.Generate(10, time.Minute)

			// Check memory leak issue.
			pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
			defer pool.AssertSize(t, 0)

			producer := NewProducerWithOptions(config.WithAllocator(pool))
			defer func() {
				if err := producer.Close(); err != nil {
					t.Error("unexpected fail", err)
				}
			}()

			batch, err := producer.BatchArrowRecordsFromTraces(traces)
			require.NoError(t, err)
			if len(batch.ArrowPayloads) == 7 {
				// Traces with resource attributes.
				require.Equal(t, arrowpb.ArrowPayloadType_SPANS, batch.ArrowPayloads[0].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_RESOURCE_ATTRS, batch.ArrowPayloads[1].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_ATTRS, batch.ArrowPayloads[2].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_EVENTS, batch.ArrowPayloads[3].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_LINKS, batch.ArrowPayloads[4].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_EVENT_ATTRS, batch.ArrowPayloads[5].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_LINK_ATTRS, batch.ArrowPayloads[6].Type)
			} else if len(batch.ArrowPayloads) == 6 {
				// Traces without resource attributes.
				require.Equal(t, arrowpb.ArrowPayloadType_SPANS, batch.ArrowPayloads[0].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_ATTRS, batch.ArrowPayloads[1].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_EVENTS, batch.ArrowPayloads[2].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_LINKS, batch.ArrowPayloads[3].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_EVENT_ATTRS, batch.ArrowPayloads[4].Type)
				require.Equal(t, arrowpb.ArrowPayloadType_SPAN_LINK_ATTRS, batch.ArrowPayloads[5].Type)
			} else {
				t.Error("unexpected fail")
			}

			consumer := NewConsumer()
			received, err := consumer.TracesFrom(batch)
			require.NoError(t, err)
			require.Equal(t, 1, len(received))

			assert.Equiv(
				stdTesting,
				[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
				[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
			)
		})
	}
}

func TestProducerConsumerLogs(t *testing.T) {
	ent := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	stdTesting := assert.NewStdUnitTest(t)

	for idx, dg := range []*datagen.LogsGenerator{
		datagen.NewLogsGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		),
		datagen.NewLogsGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewSingleInstrumentationScopes(),
		),
		datagen.NewLogsGenerator(
			ent,
			ent.NewSingleResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		),
	} {
		t.Run(fmt.Sprint("logs/", idx), func(t *testing.T) {
			logs := dg.Generate(10, time.Minute)

			// Check memory leak issue.
			pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
			defer pool.AssertSize(t, 0)

			producer := NewProducerWithOptions(config.WithAllocator(pool))
			defer func() {
				if err := producer.Close(); err != nil {
					t.Error("unexpected fail", err)
				}
			}()

			batch, err := producer.BatchArrowRecordsFromLogs(logs)
			require.NoError(t, err)
			require.Equal(t, arrowpb.ArrowPayloadType_LOGS, batch.ArrowPayloads[0].Type)

			consumer := NewConsumer()
			received, err := consumer.LogsFrom(batch)
			require.NoError(t, err)
			require.Equal(t, 1, len(received))

			assert.Equiv(
				stdTesting,
				[]json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)},
				[]json.Marshaler{plogotlp.NewExportRequestFromLogs(received[0])},
			)
		})
	}
}

func TestProducerConsumerMetrics(t *testing.T) {
	ent := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	stdTesting := assert.NewStdUnitTest(t)

	for idx, dg := range []*datagen.MetricsGenerator{
		datagen.NewMetricsGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		),
		datagen.NewMetricsGenerator(
			ent,
			ent.NewStandardResourceAttributes(),
			ent.NewSingleInstrumentationScopes(),
		),
		datagen.NewMetricsGenerator(
			ent,
			ent.NewSingleResourceAttributes(),
			ent.NewStandardInstrumentationScopes(),
		),
	} {
		t.Run(fmt.Sprint("metrics/", idx), func(t *testing.T) {
			metrics := dg.GenerateAllKindOfMetrics(10, time.Minute)

			// Check memory leak issue.
			pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
			defer pool.AssertSize(t, 0)

			producer := NewProducerWithOptions(config.WithAllocator(pool))
			defer func() {
				if err := producer.Close(); err != nil {
					t.Error("unexpected fail", err)
				}
			}()

			// First round.
			batch, err := producer.BatchArrowRecordsFromMetrics(metrics)
			require.NoError(t, err)
			require.Equal(t, arrowpb.ArrowPayloadType_METRICS, batch.ArrowPayloads[0].Type)

			consumer := NewConsumer()
			received, err := consumer.MetricsFrom(batch)
			require.NoError(t, err)
			require.Equal(t, 1, len(received))

			assert.Equiv(
				stdTesting,
				[]json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(metrics)},
				[]json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(received[0])},
			)

			// Second round (emit same data).
			batch, err = producer.BatchArrowRecordsFromMetrics(metrics)
			require.NoError(t, err)
			require.Equal(t, arrowpb.ArrowPayloadType_METRICS, batch.ArrowPayloads[0].Type)

			received, err = consumer.MetricsFrom(batch)
			require.NoError(t, err)
			require.Equal(t, 1, len(received))

			assert.Equiv(
				stdTesting,
				[]json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(metrics)},
				[]json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(received[0])},
			)
		})
	}
}
