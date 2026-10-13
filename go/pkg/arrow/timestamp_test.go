/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow

import (
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/array"
	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// timestampRecord builds a single-column, single-row record whose column has
// the given timestamp type.
func timestampRecord(t *testing.T, dt *arrow.TimestampType, value arrow.Timestamp) arrow.Record {
	t.Helper()

	schema := arrow.NewSchema([]arrow.Field{{Name: "ts", Type: dt, Nullable: true}}, nil)
	b := array.NewRecordBuilder(memory.NewGoAllocator(), schema)
	defer b.Release()

	b.Field(0).(*array.TimestampBuilder).Append(value)
	return b.NewRecord()
}

// Scenario: The producer-side schema alias used for every OTAP timestamp column
// is inspected.
// Guarantees: Go producers emit nanosecond precision tagged with UTC, which is
// what the OTAP specification requires of producers.
func TestProducerTimestampTypeIsNanosecondUTC(t *testing.T) {
	dt, ok := arrow.FixedWidthTypes.Timestamp_ns.(*arrow.TimestampType)
	require.True(t, ok, "Timestamp_ns should be a TimestampType")

	assert.Equal(t, arrow.Nanosecond, dt.Unit)
	assert.Equal(t, TimestampTimeZone, dt.TimeZone)
	assert.NoError(t, ValidateTimestampType(dt))
}

// Scenario: A timestamp column carries each time zone spelling a consumer
// accepts: "UTC", "+00:00", and -- transitionally -- an empty time zone.
// Guarantees: All three validate and decode, so a producer that has not yet
// upgraded to tagging its columns stays interoperable during a rolling upgrade.
func TestTimestampAcceptsUTCEquivalentTimeZones(t *testing.T) {
	for _, timeZone := range []string{TimestampTimeZone, TimestampTimeZoneOffset, ""} {
		dt := &arrow.TimestampType{Unit: arrow.Nanosecond, TimeZone: timeZone}

		assert.True(t, ValidTimestampTimeZone(timeZone), "time zone %q should be accepted", timeZone)
		assert.NoError(t, ValidateTimestampType(dt), "time zone %q should be accepted", timeZone)

		record := timestampRecord(t, dt, 1234)
		defer record.Release()

		value, err := TimestampFromRecord(record, 0, 0)
		require.NoError(t, err, "time zone %q should decode", timeZone)
		assert.Equal(t, arrow.Timestamp(1234), value)

		value, err = TimestampFromArray(record.Column(0), 0)
		require.NoError(t, err, "time zone %q should decode", timeZone)
		assert.Equal(t, arrow.Timestamp(1234), value)
	}
}

// Scenario: A timestamp column carries a time zone that is not UTC, or uses a
// time unit coarser than nanoseconds.
// Guarantees: Decoding fails rather than silently reinterpreting the raw int64
// against the wrong zone or unit.
func TestTimestampRejectsNonUTCTimeZonesAndOtherUnits(t *testing.T) {
	for _, timeZone := range []string{"America/Denver", "+05:30", "-07:00", "utc"} {
		dt := &arrow.TimestampType{Unit: arrow.Nanosecond, TimeZone: timeZone}

		assert.False(t, ValidTimestampTimeZone(timeZone), "time zone %q should be rejected", timeZone)
		assert.ErrorIs(t, ValidateTimestampType(dt), ErrInvalidTimestampType)

		record := timestampRecord(t, dt, 1234)
		defer record.Release()

		_, err := TimestampFromRecord(record, 0, 0)
		assert.ErrorIs(t, err, ErrInvalidTimestampType, "time zone %q should be rejected", timeZone)

		_, err = TimestampFromArray(record.Column(0), 0)
		assert.ErrorIs(t, err, ErrInvalidTimestampType, "time zone %q should be rejected", timeZone)
	}

	for _, unit := range []arrow.TimeUnit{arrow.Second, arrow.Millisecond, arrow.Microsecond} {
		dt := &arrow.TimestampType{Unit: unit, TimeZone: TimestampTimeZone}

		assert.ErrorIs(t, ValidateTimestampType(dt), ErrInvalidTimestampType)

		record := timestampRecord(t, dt, 1234)
		defer record.Release()

		_, err := TimestampFromRecord(record, 0, 0)
		assert.ErrorIs(t, err, ErrInvalidTimestampType, "unit %s should be rejected", unit)
	}
}

// Scenario: A null timestamp value sits in an otherwise conformant column, and
// an absent field id is requested.
// Guarantees: Both keep returning a zero timestamp without error, so adding
// type validation does not change how missing data is handled.
func TestTimestampNullAndAbsentFieldsUnaffected(t *testing.T) {
	dt := &arrow.TimestampType{Unit: arrow.Nanosecond, TimeZone: TimestampTimeZone}
	schema := arrow.NewSchema([]arrow.Field{{Name: "ts", Type: dt, Nullable: true}}, nil)
	b := array.NewRecordBuilder(memory.NewGoAllocator(), schema)
	defer b.Release()
	b.Field(0).(*array.TimestampBuilder).AppendNull()
	record := b.NewRecord()
	defer record.Release()

	value, err := TimestampFromRecord(record, 0, 0)
	require.NoError(t, err)
	assert.Equal(t, arrow.Timestamp(0), value)

	value, err = TimestampFromRecord(record, AbsentFieldID, 0)
	require.NoError(t, err)
	assert.Equal(t, arrow.Timestamp(0), value)
}
