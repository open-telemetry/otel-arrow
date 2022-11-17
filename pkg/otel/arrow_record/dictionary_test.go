package arrow_record

import (
	"bytes"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/ipc"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

// TestDictionaryOverflow tests the manage of dictionary overflow both side producer and consumer.
// Dictionary keys are configured as uint16 in the schema, so the maximum number of values is 65536.
func TestDictionaryOverflow(t *testing.T) {
	producer := NewProducer()
	consumer := NewConsumer()

	for i := 0; i < 70000; i++ {
		trace := ptrace.NewTraces()
		rss := trace.ResourceSpans()
		rss.EnsureCapacity(1)
		rs := rss.AppendEmpty()
		rs.SetSchemaUrl(fmt.Sprintf("schema_%d", i))

		batch, err := producer.BatchArrowRecordsFromTraces(trace)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(trace)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	for i := 0; i < 10000; i++ {
		trace := ptrace.NewTraces()
		rss := trace.ResourceSpans()
		rss.EnsureCapacity(1)
		rs := rss.AppendEmpty()
		rs.SetSchemaUrl(fmt.Sprintf("schema_%d", i))

		batch, err := producer.BatchArrowRecordsFromTraces(trace)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(trace)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}
}

func TestDictionaryOverflow2(t *testing.T) {
	producer := NewProducer()
	consumer := NewConsumer()

	for i := 0; i < 10; i++ {
		trace := ptrace.NewTraces()
		rss := trace.ResourceSpans()
		rss.EnsureCapacity(i)
		rs := rss.AppendEmpty()
		rs.SetSchemaUrl(fmt.Sprintf("schema_%d", i))

		batch, err := producer.BatchArrowRecordsFromTraces(trace)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(trace)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}

	for i := 0; i < 10; i++ {
		trace := ptrace.NewTraces()
		rss := trace.ResourceSpans()
		rss.EnsureCapacity(1)
		rs := rss.AppendEmpty()
		rs.SetSchemaUrl(fmt.Sprintf("schema_%d", i))

		batch, err := producer.BatchArrowRecordsFromTraces(trace)
		require.NoError(t, err)
		require.NotNil(t, batch)

		received, err := consumer.TracesFrom(batch)
		require.Equal(t, 1, len(received))

		assert.Equiv(
			t,
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(trace)},
			[]json.Marshaler{ptraceotlp.NewExportRequestFromTraces(received[0])},
		)
	}
}

// ToDo faire un exemple avec uint8 comme index pour voir si ça marche
// ARROW-18326
func TestDictionaryDeltas2(t *testing.T) {
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	// A schema with a single dictionary field
	schema := arrow.NewSchema([]arrow.Field{{Name: "field", Type: &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint8,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}}}, nil)

	// IPC writer and reader
	var bufWriter bytes.Buffer
	ipcWriter := ipc.NewWriter(
		&bufWriter,
		ipc.WithSchema(schema),
		ipc.WithAllocator(pool),
		ipc.WithDictionaryDeltas(true),
	)
	defer ipcWriter.Close()

	bufReader := bytes.NewReader([]byte{})
	var ipcReader *ipc.Reader

	for i := 0; i < 300; i++ {
		bldr := array.NewBuilder(pool, schema.Field(0).Type).(*array.BinaryDictionaryBuilder)
		defer bldr.Release()
		for j := 0; j < i+1; j++ {
			if j == 257 {
				println("stop")
			}
			bldr.AppendString(fmt.Sprintf(`value_%d"`, j))

		}

		arr := bldr.NewArray()
		defer arr.Release()

		// Create a first record with field = "value_0"
		record := array.NewRecord(schema, []arrow.Array{arr}, 1)
		defer record.Release()

		expectedJson, err := record.MarshalJSON()
		require.NoError(t, err)
		// Serialize and deserialize the record via an IPC stream
		json, reader, err := encodeDecodeIpcStream(t, record, &bufWriter, ipcWriter, bufReader, ipcReader)
		ipcReader = reader
		require.NoError(t, err)
		// Compare the expected JSON with the actual JSON
		require.JSONEq(t, string(expectedJson), string(json))
	}
	ipcReader.Release()
}

// ARROW-18326
func TestDictionaryDeltas(t *testing.T) {
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	// A schema with a single dictionary field
	schema := arrow.NewSchema([]arrow.Field{{Name: "field", Type: &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}}}, nil)

	// IPC writer and reader
	var bufWriter bytes.Buffer
	ipcWriter := ipc.NewWriter(&bufWriter, ipc.WithSchema(schema), ipc.WithAllocator(pool), ipc.WithDictionaryDeltas(true))
	defer ipcWriter.Close()

	bufReader := bytes.NewReader([]byte{})
	var ipcReader *ipc.Reader

	bldr := array.NewBuilder(pool, schema.Field(0).Type)
	defer bldr.Release()
	require.NoError(t, bldr.UnmarshalJSON([]byte(`["value_0"]`)))

	arr := bldr.NewArray()
	defer arr.Release()
	// Create a first record with field = "value_0"
	record := array.NewRecord(schema, []arrow.Array{arr}, 1)
	defer record.Release()

	expectedJson, err := record.MarshalJSON()
	require.NoError(t, err)
	// Serialize and deserialize the record via an IPC stream
	json, ipcReader, err := encodeDecodeIpcStream(t, record, &bufWriter, ipcWriter, bufReader, ipcReader)
	require.NoError(t, err)
	// Compare the expected JSON with the actual JSON
	require.JSONEq(t, string(expectedJson), string(json))

	// Create a second record with field = "value_1"
	require.NoError(t, bldr.UnmarshalJSON([]byte(`["value_1"]`)))
	arr = bldr.NewArray()
	defer arr.Release()
	record = array.NewRecord(schema, []arrow.Array{arr}, 1)
	defer record.Release()

	expectedJson, err = record.MarshalJSON()
	require.NoError(t, err)
	// Serialize and deserialize the record via an IPC stream
	json, ipcReader, err = encodeDecodeIpcStream(t, record, &bufWriter, ipcWriter, bufReader, ipcReader)
	require.NoError(t, err)
	// Compare the expected JSON with the actual JSON
	// field = "value_0" but should be "value_1"
	require.JSONEq(t, string(expectedJson), string(json))
	require.NoError(t, ipcReader.Err())
	ipcReader.Release()
}

// Encode and decode a record over a tuple of IPC writer and reader.
// IPC writer and reader are the same from one call to another.
func encodeDecodeIpcStream(t *testing.T,
	record arrow.Record,
	bufWriter *bytes.Buffer, ipcWriter *ipc.Writer,
	bufReader *bytes.Reader, ipcReader *ipc.Reader) ([]byte, *ipc.Reader, error) {
	// Serialize the record via an ipc writer
	if err := ipcWriter.Write(record); err != nil {
		return nil, ipcReader, err
	}
	serializedRecord := bufWriter.Bytes()
	bufWriter.Reset()
	// Deserialize the record via an ipc reader
	bufReader.Reset(serializedRecord)
	if ipcReader == nil {
		newIpcReader, err := ipc.NewReader(bufReader)
		if err != nil {
			return nil, newIpcReader, err
		}
		ipcReader = newIpcReader
	}
	ipcReader.Next()
	record = ipcReader.Record()
	// Return the decoded record as a json string
	json, err := record.MarshalJSON()
	if err != nil {
		return nil, ipcReader, err
	}
	return json, ipcReader, nil
}
