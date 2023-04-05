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
	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"go.opentelemetry.io/collector/pdata/pmetric"

	acommon "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

var (
	// ExemplarDT is an Arrow Data Type representing an OTLP metric exemplar.
	ExemplarDT = arrow.StructOf(
		arrow.Field{Name: constants.Attributes, Type: acommon.AttributesDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.TimeUnixNano, Type: arrow.FixedWidthTypes.Timestamp_ns, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.MetricValue, Type: MetricValueDT, Metadata: schema.Metadata(schema.Optional)},
		arrow.Field{Name: constants.SpanId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 8}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
		arrow.Field{Name: constants.TraceId, Type: &arrow.FixedSizeBinaryType{ByteWidth: 16}, Metadata: schema.Metadata(schema.Optional, schema.Dictionary8)},
	)
)

// ExemplarBuilder is a helper to build an Arrow array containing a collection of OTLP metric exemplar.
type ExemplarBuilder struct {
	released bool

	builder *builder.StructBuilder // `exemplar` value builder

	ab   *acommon.AttributesBuilder      // attributes builder
	tunb *builder.TimestampBuilder       // time unix nano builder
	mvb  *MetricValueBuilder             // metric value builder
	sib  *builder.FixedSizeBinaryBuilder // span id builder
	tib  *builder.FixedSizeBinaryBuilder // trace id builder
}

// ExemplarBuilderFrom creates a new ExemplarBuilder from an existing StructBuilder.
func ExemplarBuilderFrom(ex *builder.StructBuilder) *ExemplarBuilder {
	return &ExemplarBuilder{
		released: false,
		builder:  ex,

		ab:   acommon.AttributesBuilderFrom(ex.MapBuilder(constants.Attributes)),
		tunb: ex.TimestampBuilder(constants.TimeUnixNano),
		mvb:  MetricValueBuilderFrom(ex.SparseUnionBuilder(constants.MetricValue)),
		sib:  ex.FixedSizeBinaryBuilder(constants.SpanId),
		tib:  ex.FixedSizeBinaryBuilder(constants.TraceId),
	}
}

// Build builds the exemplar Arrow array.
//
// Once the returned array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *ExemplarBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Append appends an exemplar to the builder.
func (b *ExemplarBuilder) Append(ex pmetric.Exemplar) error {
	if b.released {
		return werror.Wrap(acommon.ErrBuilderAlreadyReleased)
	}

	return b.builder.Append(ex, func() error {
		if err := b.ab.Append(ex.FilteredAttributes()); err != nil {
			return werror.Wrap(err)
		}
		b.tunb.Append(arrow.Timestamp(ex.Timestamp()))
		if err := b.mvb.AppendExemplarValue(ex); err != nil {
			return werror.Wrap(err)
		}

		sid := ex.SpanID()
		b.sib.Append(sid[:])

		tid := ex.TraceID()
		b.tib.Append(tid[:])

		return nil
	})
}

// Release releases the memory allocated by the builder.
func (b *ExemplarBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}
