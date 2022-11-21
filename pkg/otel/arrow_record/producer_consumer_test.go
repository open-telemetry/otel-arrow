package arrow_record

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"
	"google.golang.org/protobuf/proto"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/air"
	cfg "github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
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
	ent := datagen.NewTestEntropy(int64(rand.Uint64()))

	dg := datagen.NewTracesGenerator(
		ent,
		ent.NewStandardResourceAttributes(),
		ent.NewStandardInstrumentationScopes(),
	)
	traces := dg.Generate(10, time.Minute)

	// Check memory leak issue.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(WithAllocator(pool))
	defer func() {
		if err := producer.Close(); err != nil {
			t.Error("unexpected fail", err)
		}
	}()

	batch, err := producer.BatchArrowRecordsFromTraces(traces)
	require.NoError(t, err)
	require.Equal(t, arrowpb.OtlpArrowPayloadType_SPANS, batch.OtlpArrowPayloads[0].Type)

	consumer := NewConsumer()
	received, err := consumer.TracesFrom(batch)
	require.Equal(t, 1, len(received))

	assert.Equiv(
		t,
		[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(traces)},
		[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
	)
}

func TestProducerConsumerLogs(t *testing.T) {
	ent := datagen.NewTestEntropy(int64(rand.Uint64()))

	dg := datagen.NewLogsGenerator(
		ent,
		ent.NewStandardResourceAttributes(),
		ent.NewStandardInstrumentationScopes(),
	)
	logs := dg.Generate(10, time.Minute)

	// Check memory leak issue.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(WithAllocator(pool))
	defer func() {
		if err := producer.Close(); err != nil {
			t.Error("unexpected fail", err)
		}
	}()

	batch, err := producer.BatchArrowRecordsFromLogs(logs)
	require.NoError(t, err)
	require.Equal(t, arrowpb.OtlpArrowPayloadType_LOGS, batch.OtlpArrowPayloads[0].Type)

	consumer := NewConsumer()
	received, err := consumer.LogsFrom(batch)
	require.Equal(t, 1, len(received))

	assert.Equiv(
		t,
		[]json.Marshaler{plogotlp.NewExportRequestFromLogs(logs)},
		[]json.Marshaler{plogotlp.NewExportRequestFromLogs(received[0])},
	)
}

func TestProducerConsumerMetrics(t *testing.T) {
	ent := datagen.NewTestEntropy(int64(rand.Uint64()))

	dg := datagen.NewMetricsGenerator(
		ent,
		ent.NewStandardResourceAttributes(),
		ent.NewStandardInstrumentationScopes(),
	)
	metrics := dg.Generate(10, time.Minute)

	// Check memory leak issue.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(WithAllocator(pool))
	defer func() {
		if err := producer.Close(); err != nil {
			t.Error("unexpected fail", err)
		}
	}()

	batch, err := producer.BatchArrowRecordsFromMetrics(metrics)
	require.NoError(t, err)
	require.Equal(t, arrowpb.OtlpArrowPayloadType_METRICS, batch.OtlpArrowPayloads[0].Type)

	consumer := NewConsumer()
	received, err := consumer.MetricsFrom(batch)
	require.Equal(t, 1, len(received))

	assert.Equiv(
		t,
		[]json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(metrics)},
		[]json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(received[0])},
	)
}

func TestProducerConsumer(t *testing.T) {
	t.Parallel()

	// Check memory leak issue.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(WithAllocator(pool))
	defer func() {
		if err := producer.Close(); err != nil {
			t.Error("unexpected fail", err)
		}
	}()

	consumer := NewConsumer()
	config := cfg.NewUint8DefaultConfig()
	rr := air.NewRecordRepository(config)

	recordCount := 10
	tsValues := make([]int64, 0, recordCount)
	for i := 0; i < recordCount; i++ {
		tsValues = append(tsValues, int64(i))
	}
	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	records, err := rr.BuildRecords()
	if err != nil {
		t.Fatal(err)
	}

	rms := make([]*RecordMessage, len(records))
	for i, record := range records {
		rms[i] = NewTraceMessage(record, arrowpb.DeliveryType_BEST_EFFORT)
	}

	bar, err := producer.Produce(rms, arrowpb.DeliveryType_BEST_EFFORT)
	if err != nil {
		t.Fatal(err)
	}
	recordMessages, err := consumer.Consume(bar)
	if err != nil {
		t.Fatal(err)
	}
	if len(recordMessages) != 1 {
		t.Errorf("Expected 1 record message, got %d", len(recordMessages))
	}
	if recordMessages[0].Record().NumCols() != 5 {
		t.Errorf("Expected 5 columns, got %d", recordMessages[0].Record().NumCols())
	}
	if recordMessages[0].Record().NumRows() != int64(recordCount) {
		t.Errorf("Expected %d rows, got %d", recordCount, recordMessages[0].Record().NumRows())
	}

	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	records, err = rr.BuildRecords()
	if err != nil {
		t.Fatal(err)
	}

	rms = make([]*RecordMessage, len(records))
	for i, record := range records {
		rms[i] = NewTraceMessage(record, arrowpb.DeliveryType_BEST_EFFORT)
	}

	bar, err = producer.Produce(rms, arrowpb.DeliveryType_BEST_EFFORT)
	if err != nil {
		t.Fatal(err)
	}
	recordMessages, err = consumer.Consume(bar)
	if err != nil {
		t.Fatal(err)
	}
	if len(recordMessages) != 1 {
		t.Errorf("Expected 1 record message, got %d", len(recordMessages))
	}
	if recordMessages[0].Record().NumCols() != 5 {
		t.Errorf("Expected 5 columns, got %d", recordMessages[0].Record().NumCols())
	}
	if recordMessages[0].Record().NumRows() != int64(recordCount) {
		t.Errorf("Expected %d rows, got %d", recordCount, recordMessages[0].Record().NumRows())
	}
}

func GenRecord(ts int64, value_a, value_b, value_c int) *air.Record {
	record := air.NewRecord()
	record.I64Field("ts", ts)
	record.StringField("c", fmt.Sprintf("c_%d", value_c))
	record.StringField("a", fmt.Sprintf("a___%d", value_a))
	record.StringField("b", fmt.Sprintf("b__%d", value_b))
	record.StructField("d", rfield.Struct{
		Fields: []*rfield.Field{
			{Name: "a", Value: rfield.NewString(fmt.Sprintf("a_%d", value_a))},
			{Name: "b", Value: rfield.NewString(fmt.Sprintf("b_%d", value_b))},
			{Name: "c", Value: &rfield.List{Values: []rfield.Value{
				rfield.NewI64(1),
				rfield.NewI64(2),
				rfield.NewI64(3),
			}}},
			{Name: "d", Value: &rfield.List{Values: []rfield.Value{
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 1),
					rfield.NewF64Field("b", 2.0),
					rfield.NewStringField("c", "3"),
				}},
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 4),
					rfield.NewF64Field("b", 5.0),
					rfield.NewStringField("c", "6"),
				}},
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 7),
					rfield.NewF64Field("b", 8.0),
					rfield.NewStringField("c", "9"),
				}},
			}}},
		},
	})
	return record
}
