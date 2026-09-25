/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow_record

import (
	"bytes"
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/array"
	"github.com/apache/arrow-go/v18/arrow/ipc"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/plog"

	colarspb "github.com/open-telemetry/otel-arrow/go/api/experimental/arrow/v1"
)

// logsWithTimestamps builds a logs payload whose records carry non-zero
// timestamps, so the encoder actually emits timestamp columns. The shared
// GenerateLogs fixture leaves timestamps at zero, which the adaptive encoder
// then omits entirely.
func logsWithTimestamps(count int) plog.Logs {
	logs := plog.NewLogs()
	rl := logs.ResourceLogs().AppendEmpty()
	rl.SetSchemaUrl("schema")
	records := rl.ScopeLogs().AppendEmpty().LogRecords()

	base := pcommon.Timestamp(1_700_000_000_000_000_000)
	for i := 0; i < count; i++ {
		lr := records.AppendEmpty()
		lr.SetTimestamp(base + pcommon.Timestamp(i))
		lr.SetObservedTimestamp(base + pcommon.Timestamp(i) + 1)
		lr.Body().SetStr("body")
		lr.Attributes().PutStr("attr", "value")
	}
	return logs
}

// retagTimestamps rewrites every timestamp field in the schema to carry the
// given time zone, rebuilding each record against the rewritten schema. The
// underlying int64 values are untouched, so this reproduces exactly what
// differs on the wire between a producer that tags its timestamp columns and
// one that does not.
func retagTimestamps(t *testing.T, rec arrow.Record, timeZone string) arrow.Record {
	t.Helper()

	oldSchema := rec.Schema()
	fields := make([]arrow.Field, oldSchema.NumFields())
	columns := make([]arrow.Array, rec.NumCols())
	rewrote := false

	for i := 0; i < oldSchema.NumFields(); i++ {
		f := oldSchema.Field(i)
		col := rec.Column(i)

		if ts, ok := f.Type.(*arrow.TimestampType); ok {
			f.Type = &arrow.TimestampType{Unit: ts.Unit, TimeZone: timeZone}

			// Rebuild the array against the retagged type, reusing the
			// existing value and validity buffers.
			data := col.Data()
			newData := array.NewData(
				f.Type, data.Len(), data.Buffers(), data.Children(),
				data.NullN(), data.Offset(),
			)
			col = array.MakeFromData(newData)
			newData.Release()
			defer col.Release()
			rewrote = true
		}

		fields[i] = f
		columns[i] = col
	}

	require.True(t, rewrote, "fixture must contain at least one timestamp column")

	newSchema := arrow.NewSchema(fields, nil)
	return array.NewRecord(newSchema, columns, rec.NumRows())
}

// reencode serializes a record as a standalone Arrow IPC stream, matching the
// framing the OTAP consumer expects in an ArrowPayload.
func reencode(t *testing.T, rec arrow.Record) []byte {
	t.Helper()

	var buf bytes.Buffer
	w := ipc.NewWriter(&buf, ipc.WithSchema(rec.Schema()), ipc.WithAllocator(memory.NewGoAllocator()))
	require.NoError(t, w.Write(rec))
	require.NoError(t, w.Close())
	return buf.Bytes()
}

// decodeLogPayloadWithTimeZone takes a real OTAP logs batch, rewrites the time
// zone on every timestamp column of the LOGS payload, and runs the result back
// through a Consumer. It returns the number of log records recovered.
func decodeLogPayloadWithTimeZone(t *testing.T, timeZone string) (int, error) {
	t.Helper()

	producer := NewProducer()
	defer func() { require.NoError(t, producer.Close()) }()

	logs := logsWithTimestamps(10)
	batch, err := producer.BatchArrowRecordsFromLogs(logs)
	require.NoError(t, err)

	// Decode the batch once so we can reach the decoded LOGS record.
	inspect := NewConsumer()
	messages, err := inspect.Consume(batch)
	require.NoError(t, err)

	rebuilt := make([]*colarspb.ArrowPayload, 0, len(batch.ArrowPayloads))
	retagged := false

	for _, payload := range batch.ArrowPayloads {
		if payload.Type != colarspb.ArrowPayloadType_LOGS {
			rebuilt = append(rebuilt, payload)
			continue
		}

		for _, m := range messages {
			if m.PayloadType() != colarspb.ArrowPayloadType_LOGS {
				continue
			}
			newRec := retagTimestamps(t, m.Record(), timeZone)
			defer newRec.Release()

			rebuilt = append(rebuilt, &colarspb.ArrowPayload{
				SchemaId: payload.SchemaId + "/retagged/" + timeZone,
				Type:     payload.Type,
				Record:   reencode(t, newRec),
			})
			retagged = true
			break
		}
	}
	require.NoError(t, inspect.Close())
	require.True(t, retagged, "expected a LOGS payload to retag")

	consumer := NewConsumer()
	defer func() { _ = consumer.Close() }()

	received, err := consumer.LogsFrom(&colarspb.BatchArrowRecords{
		BatchId:       batch.BatchId,
		ArrowPayloads: rebuilt,
		Headers:       batch.Headers,
	})
	if err != nil {
		return 0, err
	}

	count := 0
	for _, l := range received {
		count += l.LogRecordCount()
	}
	return count, nil
}

// Scenario: An OTAP logs payload arrives with its timestamp columns tagged
// "UTC", tagged "+00:00", or left untagged, and is decoded by the current
// consumer.
// Guarantees: All three decode, so a peer that has not yet upgraded to tagging
// its timestamp columns stays interoperable with this consumer during a
// rolling upgrade.
func TestConsumerAcceptsUTCEquivalentTimeZonesOverIPC(t *testing.T) {
	for _, timeZone := range []string{"UTC", "+00:00", ""} {
		label := timeZone
		if label == "" {
			label = "(untagged)"
		}

		count, err := decodeLogPayloadWithTimeZone(t, timeZone)
		require.NoError(t, err, "time zone %s should decode", label)
		assert.Positive(t, count, "time zone %s should yield log records", label)
	}
}

// Scenario: An OTAP logs payload arrives with its timestamp columns tagged with
// a time zone that is not UTC.
// Guarantees: Decoding fails rather than silently reinterpreting the raw int64
// values against the wrong zone.
func TestConsumerRejectsNonUTCTimeZoneOverIPC(t *testing.T) {
	_, err := decodeLogPayloadWithTimeZone(t, "America/Denver")
	require.Error(t, err, "a non-UTC time zone must be rejected")
}
