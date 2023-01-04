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
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

// EHistogramDataPointBucketsDT is the Arrow Data Type describing an exponential histogram data point buckets.
var (
	EHistogramDataPointBucketsDT = arrow.StructOf(
		arrow.Field{Name: constants.ExpHistogramOffset, Type: arrow.PrimitiveTypes.Int32},
		arrow.Field{Name: constants.ExpHistogramBucketCounts, Type: arrow.ListOf(arrow.PrimitiveTypes.Uint64)},
	)
)

// EHistogramDataPointBucketsBuilder is a builder for exponential histogram data point buckets.
type EHistogramDataPointBucketsBuilder struct {
	released bool

	builder *array.StructBuilder

	ob   *array.Int32Builder  // offset builder
	bclb *array.ListBuilder   // exp histogram bucket counts list builder
	bcb  *array.Uint64Builder // exp histogram bucket counts builder
}

// NewEHistogramDataPointBucketsBuilder creates a new EHistogramDataPointBucketsBuilderFrom with a given memory allocator.
func NewEHistogramDataPointBucketsBuilder(pool memory.Allocator) *EHistogramDataPointBucketsBuilder {
	return EHistogramDataPointBucketsBuilderFrom(array.NewStructBuilder(pool, EHistogramDataPointBucketsDT))
}

// EHistogramDataPointBucketsBuilderFrom creates a new EHistogramDataPointBucketsBuilder from an existing StructBuilder.
func EHistogramDataPointBucketsBuilderFrom(b *array.StructBuilder) *EHistogramDataPointBucketsBuilder {
	return &EHistogramDataPointBucketsBuilder{
		released: false,
		builder:  b,

		ob:   b.FieldBuilder(0).(*array.Int32Builder),
		bclb: b.FieldBuilder(1).(*array.ListBuilder),
		bcb:  b.FieldBuilder(1).(*array.ListBuilder).ValueBuilder().(*array.Uint64Builder),
	}
}

// Build builds the underlying array.
//
// Once the array is no longer needed, Release() should be called to free the memory.
func (b *EHistogramDataPointBucketsBuilder) Build() (*array.Struct, error) {
	if b.released {
		return nil, fmt.Errorf("EHistogramDataPointBucketsBuilder: Build() called after Release()")
	}

	defer b.Release()
	return b.builder.NewStructArray(), nil
}

// Release releases the underlying memory.
func (b *EHistogramDataPointBucketsBuilder) Release() {
	if b.released {
		return
	}

	b.released = true
	b.builder.Release()
}

// Append appends a new histogram data point to the builder.
func (b *EHistogramDataPointBucketsBuilder) Append(hdpb pmetric.ExponentialHistogramDataPointBuckets) error {
	if b.released {
		return fmt.Errorf("EHistogramDataPointBucketsBuilder: Append() called after Release()")
	}

	b.builder.Append(true)
	b.ob.Append(hdpb.Offset())

	bc := hdpb.BucketCounts()
	bcc := bc.Len()
	if bcc > 0 {
		b.bclb.Append(true)
		b.bclb.Reserve(bcc)
		for i := 0; i < bcc; i++ {
			b.bcb.Append(bc.At(i))
		}
	} else {
		b.bclb.Append(false)
	}

	return nil
}
