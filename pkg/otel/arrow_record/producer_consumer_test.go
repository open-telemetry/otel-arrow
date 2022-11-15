package arrow_record

import (
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/plog/plogotlp"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	arrowpb "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/air"
	cfg "github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/air/rfield"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

func TestProducerConsumerTraces(t *testing.T) {
	dg := datagen.NewTracesGenerator(
		datagen.DefaultResourceAttributes(),
		datagen.DefaultInstrumentationScopes(),
	)
	traces := dg.Generate(10, time.Minute)

	producer := NewProducer()

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
	dg := datagen.NewLogsGenerator(
		datagen.DefaultResourceAttributes(),
		datagen.DefaultInstrumentationScopes(),
	)
	logs := dg.Generate(10, time.Minute)

	producer := NewProducer()

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
	dg := datagen.NewMetricsGenerator(
		datagen.DefaultResourceAttributes(),
		datagen.DefaultInstrumentationScopes(),
	)
	metrics := dg.Generate(10, time.Minute)

	producer := NewProducer()

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

	producer := NewProducer()
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
