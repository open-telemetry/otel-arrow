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

package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/plog"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// Arrow Data Types describing log record and body.
var (
	// LogRecordDT is the Arrow Data Type describing a log record.
	LogRecordDT = arrow.StructOf([]arrow.Field{
		{Name: constants.TimeUnixNano, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.ObservedTimeUnixNano, Type: arrow.PrimitiveTypes.Uint64},
		{Name: constants.TraceId, Type: acommon.DefaultDictFixed16Binary},
		{Name: constants.SpanId, Type: acommon.DefaultDictFixed8Binary},
		{Name: constants.SeverityNumber, Type: arrow.PrimitiveTypes.Int32},
		{Name: constants.SeverityText, Type: acommon.DefaultDictString},
		{Name: constants.Body, Type: acommon.AnyValueDT},
		{Name: constants.Attributes, Type: acommon.AttributesDT},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32},
		{Name: constants.Flags, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

// LogRecordBuilder is a helper to build a log record.
type LogRecordBuilder struct {
	released bool

	builder *array.StructBuilder

	tunb  *array.Uint64Builder               // time unix nano builder
	otunb *array.Uint64Builder               // observed time unix nano builder
	tib   *acommon.AdaptiveDictionaryBuilder // trace id builder
	sib   *acommon.AdaptiveDictionaryBuilder // span id builder
	snb   *array.Int32Builder                // severity number builder
	stb   *acommon.AdaptiveDictionaryBuilder // severity text builder
	bb    *acommon.AnyValueBuilder           // body builder (LOL)
	ab    *acommon.AttributesBuilder         // attributes builder
	dacb  *array.Uint32Builder               // dropped attributes count builder
	fb    *array.Uint32Builder               // flags builder
}

// NewLogRecordBuilder creates a new LogRecordBuilder with a given allocator.
//
// Once the builder is no longer needed, Release() must be called to free the
// memory allocated by the builder.
func NewLogRecordBuilder(pool memory.Allocator) *LogRecordBuilder {
	sb := array.NewStructBuilder(pool, LogRecordDT)
	return LogRecordBuilderFrom(sb)
}

func LogRecordBuilderFrom(sb *array.StructBuilder) *LogRecordBuilder {
	return &LogRecordBuilder{
		released: false,
		builder:  sb,
		tunb:     sb.FieldBuilder(0).(*array.Uint64Builder),
		otunb:    sb.FieldBuilder(1).(*array.Uint64Builder),
		tib:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(2)),
		sib:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(3)),
		snb:      sb.FieldBuilder(4).(*array.Int32Builder),
		stb:      acommon.AdaptiveDictionaryBuilderFrom(sb.FieldBuilder(5)),
		bb:       acommon.AnyValueBuilderFrom(sb.FieldBuilder(6).(*array.SparseUnionBuilder)),
		ab:       acommon.AttributesBuilderFrom(sb.FieldBuilder(7).(*array.MapBuilder)),
		dacb:     sb.FieldBuilder(8).(*array.Uint32Builder),
		fb:       sb.FieldBuilder(9).(*array.Uint32Builder),
	}
}

// Build builds the log record array.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *LogRecordBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("log builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends a new log record to the builder.
func (b *LogRecordBuilder) Append(log plog.LogRecord) error {
	if b.released {
		return fmt.Errorf("log record builder already released")
	}

	b.builder.Append(true)
	b.tunb.Append(uint64(log.Timestamp()))
	b.otunb.Append(uint64(log.ObservedTimestamp()))
	tib := log.TraceID()
	if err := b.tib.AppendBinary(tib[:]); err != nil {
		return err
	}
	sib := log.SpanID()
	if err := b.sib.AppendBinary(sib[:]); err != nil {
		return err
	}
	b.snb.Append(int32(log.SeverityNumber()))
	severityText := log.SeverityText()
	if severityText == "" {
		b.stb.AppendNull()
	} else {
		if err := b.stb.AppendString(severityText); err != nil {
			return err
		}
	}
	if err := b.bb.Append(log.Body()); err != nil {
		return err
	}
	if err := b.ab.Append(log.Attributes()); err != nil {
		return err
	}
	b.dacb.Append(log.DroppedAttributesCount())
	b.fb.Append(uint32(log.Flags()))

	return nil
}

// Release releases the memory allocated by the builder.
func (b *LogRecordBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
