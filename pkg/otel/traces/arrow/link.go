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
	"go.opentelemetry.io/collector/pdata/ptrace"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// LinkDT is the Arrow Data Type describing a link event.
var (
	LinkDT = arrow.StructOf([]arrow.Field{
		{Name: constants.TraceId, Type: acommon.DefaultDictFixed16Binary},
		// TODO: Not sure a dictionary if needed here
		{Name: constants.SpanId, Type: acommon.DefaultDictFixed8Binary},
		{Name: constants.TraceState, Type: acommon.DefaultDictString},
		{Name: constants.Attributes, Type: acommon.AttributesDT},
		{Name: constants.DroppedAttributesCount, Type: arrow.PrimitiveTypes.Uint32},
	}...)
)

type LinkBuilder struct {
	released bool
	builder  *array.StructBuilder
	tib      *acommon.AdaptiveDictionaryBuilder // trace id builder
	sib      *acommon.AdaptiveDictionaryBuilder // span id builder
	tsb      *acommon.AdaptiveDictionaryBuilder // trace state builder
	ab       *acommon.AttributesBuilder         // attributes builder
	dacb     *array.Uint32Builder               // dropped attributes count builder
}

func NewLinkBuilder(pool memory.Allocator) *LinkBuilder {
	return LinkBuilderFrom(array.NewStructBuilder(pool, LinkDT))
}

func LinkBuilderFrom(lb *array.StructBuilder) *LinkBuilder {
	return &LinkBuilder{
		released: false,
		builder:  lb,
		tib:      acommon.AdaptiveDictionaryBuilderFrom(lb.FieldBuilder(0)),
		sib:      acommon.AdaptiveDictionaryBuilderFrom(lb.FieldBuilder(1)),
		tsb:      acommon.AdaptiveDictionaryBuilderFrom(lb.FieldBuilder(2)),
		ab:       acommon.AttributesBuilderFrom(lb.FieldBuilder(3).(*array.MapBuilder)),
		dacb:     lb.FieldBuilder(4).(*array.Uint32Builder),
	}
}

// Append appends a new link to the builder.
func (b *LinkBuilder) Append(link ptrace.SpanLink) error {
	if b.released {
		return fmt.Errorf("link builder already released")
	}

	b.builder.Append(true)
	tid := link.TraceID()
	if err := b.tib.AppendBinary(tid[:]); err != nil {
		return err
	}
	sid := link.SpanID()
	if err := b.sib.AppendBinary(sid[:]); err != nil {
		return err
	}
	traceState := link.TraceState().AsRaw()
	if traceState == "" {
		b.tsb.AppendNull()
	} else {
		if err := b.tsb.AppendString(traceState); err != nil {
			return err
		}
	}
	if err := b.ab.Append(link.Attributes()); err != nil {
		return err
	}
	b.dacb.Append(link.DroppedAttributesCount())
	return nil
}

// Build builds the link array struct.
//
// Once the array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *LinkBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("link builder already released")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the memory allocated by the builder.
func (b *LinkBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
