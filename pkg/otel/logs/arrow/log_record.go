/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

import (
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// Arrow Data Types describing log record and body.
var (
	// LogRecordDT is the Arrow Data Type describing a log record.
	LogRecordDT = arrow.StructOf([]arrow.Field{
		{Name: constants.ID, Type: arrow.PrimitiveTypes.Uint16, Metadata: schema.Metadata(schema.Optional, schema.DeltaEncoding)},
		{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		{Name: constants.ObservedTimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns},
		{Name: constants.TraceId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 16}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.SpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.SeverityNumber, Type: arrow.PrimitiveTypes.Int32, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.SeverityText, Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		{Name: constants.Body, Type: acommon.AnyValueDT, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
		{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32, Metadata: schema.Metadata(schema.Optional)},
	}...)
)

// LogRecordBuilder is a helper to build a log record.
type LogRecordBuilder struct {
	released bool

	builder *builder.StructBuilder

	ib    *builder.Uint16DeltaBuilder     //  id builder
	tunb  *builder.TimestampBuilder       // time unix nano builder
	otunb *builder.TimestampBuilder       // observed time unix nano builder
	tib   *builder.FixedSizeBinaryBuilder // trace id builder
	sib   *builder.FixedSizeBinaryBuilder // span id builder
	snb   *builder.Int32Builder           // severity number builder
	stb   *builder.StringBuilder          // severity text builder
	bb    *acommon.AnyValueBuilder        // body builder (LOL)
	dacb  *builder.Uint32Builder          // dropped attributes count builder
	fb    *builder.Uint32Builder          // flags builder
}

func LogRecordBuilderFrom(sb *builder.StructBuilder) *LogRecordBuilder {
	ib := sb.Uint16DeltaBuilder(constants.ID)
	// As the attributes are sorted before insertion, the delta between two
	// consecutive attributes ID should always be <=1.
	ib.SetMaxDelta(1)

	return &LogRecordBuilder{
		released: false,
		builder:  sb,
		ib:       ib,
		tunb:     sb.TimestampBuilder(constants.TimeUnixNano),
		otunb:    sb.TimestampBuilder(constants.ObservedTimeUnixNano),
		tib:      sb.FixedSizeBinaryBuilder(constants.TraceId),
		sib:      sb.FixedSizeBinaryBuilder(constants.SpanId),
		snb:      sb.Int32Builder(constants.SeverityNumber),
		stb:      sb.StringBuilder(constants.SeverityText),
		bb:       acommon.AnyValueBuilderFrom(sb.SparseUnionBuilder(constants.Body)),
		dacb:     sb.Uint32Builder(constants.DroppedAttributesCount),
		fb:       sb.Uint32Builder(constants.Flags),
	}
}

// Build builds the log record array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *LogRecordBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new log record to the builder.
func (b *LogRecordBuilder) Append(log *plog.LogRecord, relatedData *RelatedData) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(log, func() error {
		ID := relatedData.NextSpanID()

		b.ib.Append(ID)
		b.tunb.Append(arrow.Timestamp(log.Timestamp()))
		b.otunb.Append(arrow.Timestamp(log.ObservedTimestamp()))
		tib := log.TraceID()
		b.tib.Append(tib[:])
		sib := log.SpanID()
		b.sib.Append(sib[:])
		b.snb.AppendNonZero(int32(log.SeverityNumber()))
		b.stb.AppendNonEmpty(log.SeverityText())
		if err := b.bb.Append(log.Body()); err != nil {
			return werror.Wrap(err)
		}

		// Log record attributes
		attrsAccu := relatedData.AttrsBuilders().LogRecord().Accumulator()
		err := attrsAccu.AppendUniqueAttributesWithID(ID, log.Attributes(), nil, nil)
		if err != nil {
			return werror.Wrap(err)
		}
		b.dacb.AppendNonZero(log.DroppedAttributesCount())

		b.fb.Append(uint32(log.Flags()))

		return nil
	})
}

// Release releases the memory allocated by the builder.
func (b *LogRecordBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
