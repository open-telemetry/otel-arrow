/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package otlp

import (
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/array"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"

	"github.com/open-telemetry/otel-arrow/go/pkg/otel/common/otlp"
	"github.com/open-telemetry/otel-arrow/go/pkg/otel/constants"
)

// TestExemplarDecoder_NullSpanIDAndTraceID verifies that the exemplar decoder
// handles null span_id and trace_id values gracefully. These fields are
// optional per the OTAP spec and OTLP proto, so a null value should be
// treated as "not provided" rather than causing an error.
func TestExemplarDecoder_NullSpanIDAndTraceID(t *testing.T) {
	pool := memory.NewGoAllocator()

	// Build a simplified exemplar schema with plain types (no dictionary
	// encoding) — the decoder handles both dictionary and non-dictionary
	// arrays via the arrowutils helpers.
	schema := arrow.NewSchema([]arrow.Field{
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint32, Nullable: true},
		{Name: constants.ParentID, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		{Name: constants.IntValue, Type: arrow.PrimitiveTypes.Int64, Nullable: true},
		{Name: constants.DoubleValue, Type: arrow.PrimitiveTypes.Float64, Nullable: true},
		{Name: constants.SpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}, Nullable: true},
		{Name: constants.TraceId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 16}, Nullable: true},
	}, nil)

	rb := array.NewRecordBuilder(pool, schema)
	defer rb.Release()

	// Append one row with valid required fields but null span_id and trace_id.
	rb.Field(0).(*array.Uint32Builder).AppendNull()          // id (nullable)
	rb.Field(1).(*array.Uint32Builder).Append(1)             // parent_id
	rb.Field(2).(*array.TimestampBuilder).Append(1000)       // time_unix_nano
	rb.Field(3).(*array.Int64Builder).Append(42)             // int_value
	rb.Field(4).(*array.Float64Builder).AppendNull()         // double_value (null)
	rb.Field(5).(*array.FixedSizeBinaryBuilder).AppendNull() // span_id (null)
	rb.Field(6).(*array.FixedSizeBinaryBuilder).AppendNull() // trace_id (null)

	record := rb.NewRecord()
	defer record.Release()

	attrsStore := otlp.NewAttributesStore[uint32]()

	store, err := ExemplarsStoreFrom(record, attrsStore)
	require.NoError(t, err, "decoding exemplar with null span_id and trace_id should not error")
	require.NotNil(t, store)

	// The exemplar should have been decoded with parent_id=1.
	exemplars := store.ExemplarsByID(1)
	require.Equal(t, 1, exemplars.Len(), "expected one exemplar for parent_id=1")

	ex := exemplars.At(0)
	// Null span_id/trace_id should result in zero-value IDs.
	require.True(t, ex.SpanID().IsEmpty(), "null span_id should decode as empty")
	require.True(t, ex.TraceID().IsEmpty(), "null trace_id should decode as empty")
	// The int_value should still be set.
	require.Equal(t, int64(42), ex.IntValue())
}
