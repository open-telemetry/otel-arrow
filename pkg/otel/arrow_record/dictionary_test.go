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
	"github.com/davecgh/go-spew/spew"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/ptrace"
	"go.opentelemetry.io/collector/pdata/ptrace/ptraceotlp"

	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
)

func TestDictionaryOverflow(t *testing.T) {
	producer := NewProducer()
	consumer := NewConsumer()

	for i := 0; i < 1000; i++ {
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

		if i%100 == 0 {
			spew.Dump(received)
		}
	}
}

// NOTE: Release methods are not managed in this test for simplicity.
func TestDictionary(t *testing.T) {
	pool := memory.NewGoAllocator()
	// A schema with a single dictionary field
	schema := arrow.NewSchema([]arrow.Field{{Name: "field", Type: &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}}}, nil)
	// IPC writer and reader
	var bufWriter bytes.Buffer
	ipcWriter := ipc.NewWriter(&bufWriter, ipc.WithSchema(schema))
	bufReader := bytes.NewReader([]byte{})
	var ipcReader *ipc.Reader

	// Create a first record with field = "value_0"
	record := CreateRecord(t, pool, schema, 0)
	expectedJson, err := record.MarshalJSON()
	require.NoError(t, err)
	// Serialize and deserialize the record via an IPC stream
	json, ipcReader, err := EncodeDecodeIpcStream(t, record, &bufWriter, ipcWriter, bufReader, ipcReader)
	require.NoError(t, err)
	// Compare the expected JSON with the actual JSON
	require.JSONEq(t, string(expectedJson), string(json))

	// Create a second record with field = "value_1"
	record = CreateRecord(t, pool, schema, 1)
	expectedJson, err = record.MarshalJSON()
	require.NoError(t, err)
	// Serialize and deserialize the record via an IPC stream
	json, ipcReader, err = EncodeDecodeIpcStream(t, record, &bufWriter, ipcWriter, bufReader, ipcReader)
	require.NoError(t, err)
	// Compare the expected JSON with the actual JSON
	// field = "value_0" but should be "value_1"
	require.JSONEq(t, string(expectedJson), string(json))
}

// Create a record with a single field.
// The value of field `field` depends on the value passed in parameter.
func CreateRecord(t *testing.T, pool memory.Allocator, schema *arrow.Schema, value int) arrow.Record {
	rb := array.NewRecordBuilder(pool, schema)
	fieldB := rb.Field(0).(*array.BinaryDictionaryBuilder)
	err := fieldB.AppendString(fmt.Sprintf("value_%d", value))
	if err != nil {
		t.Fatal(err)
	}
	return rb.NewRecord()
}

// Encode and decode a record over a tuple of IPC writer and reader.
// IPC writer and reader are the same from one call to another.
func EncodeDecodeIpcStream(t *testing.T,
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
