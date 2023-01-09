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

// Constants used to identify the type of value in the union.
const (
	I64Code int8 = 0
	F64Code int8 = 1
)

var (
	// MetricValueDT is an Arrow Data Type representing an OTLP metric value.
	MetricValueDT = arrow.SparseUnionOf([]arrow.Field{
		{Name: constants.I64MetricValue, Type: arrow.PrimitiveTypes.Int64},
		{Name: constants.F64MetricValue, Type: arrow.PrimitiveTypes.Float64},
	}, []int8{
		I64Code,
		F64Code,
	})
)

// MetricValueBuilder is a helper to build an Arrow array containing a collection of OTLP metric value.
type MetricValueBuilder struct {
	released bool

	builder *array.SparseUnionBuilder // metric value builder

	i64Builder *array.Int64Builder   // int64 builder
	f64Builder *array.Float64Builder // float64 builder
}

// NewMetricValueBuilder creates a new MetricValueBuilder with a given memory allocator.
func NewMetricValueBuilder(pool memory.Allocator) *MetricValueBuilder {
	return MetricValueBuilderFrom(array.NewSparseUnionBuilder(pool, MetricValueDT))
}

// MetricValueBuilderFrom creates a new MetricValueBuilder from an existing SparseUnionBuilder.
func MetricValueBuilderFrom(mv *array.SparseUnionBuilder) *MetricValueBuilder {
	return &MetricValueBuilder{
		released:   false,
		builder:    mv,
		i64Builder: mv.Child(0).(*array.Int64Builder),
		f64Builder: mv.Child(1).(*array.Float64Builder),
	}
}

// Build builds the "metric value" Arrow array.
//
// Once the returned array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *MetricValueBuilder) Build() (*array.SparseUnion, error) {
	if b.released {
		return nil, fmt.Errorf("metric value builder already released")
	}

	defer b.Release()
	return b.builder.NewSparseUnionArray(), nil
}

// AppendNumberDataPointValue appends a new metric value to the builder.
func (b *MetricValueBuilder) AppendNumberDataPointValue(mdp pmetric.NumberDataPoint) error {
	if b.released {
		return fmt.Errorf("metric value builder already released")
	}

	var err error
	switch mdp.ValueType() {
	case pmetric.NumberDataPointValueTypeDouble:
		b.appendF64(mdp.DoubleValue())
	case pmetric.NumberDataPointValueTypeInt:
		b.appendI64(mdp.IntValue())
	case pmetric.NumberDataPointValueTypeEmpty:
		// ignore empty data point.
	}
	return err
}

// AppendExemplarValue appends a new exemplar value to the builder.
func (b *MetricValueBuilder) AppendExemplarValue(ex pmetric.Exemplar) error {
	if b.released {
		return fmt.Errorf("metric value builder already released")
	}

	var err error
	switch ex.ValueType() {
	case pmetric.ExemplarValueTypeDouble:
		b.appendF64(ex.DoubleValue())
	case pmetric.ExemplarValueTypeInt:
		b.appendI64(ex.IntValue())
	case pmetric.ExemplarValueTypeEmpty:
		// ignore empty exemplar.
	}
	return err
}

// Release releases the memory allocated by the builder.
func (b *MetricValueBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}

// appendI64 appends a new int64 value to the builder.
func (b *MetricValueBuilder) appendI64(v int64) {
	b.builder.Append(I64Code)
	b.i64Builder.Append(v)
	b.f64Builder.AppendNull()
}

// appendF64 appends a new double value to the builder.
func (b *MetricValueBuilder) appendF64(v float64) {
	b.builder.Append(F64Code)
	b.f64Builder.Append(v)
	b.i64Builder.AppendNull()
}
