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
	MetricValueDT = arrow.DenseUnionOf([]arrow.Field{
		{Name: constants.I64_METRIC_VALUE, Type: arrow.PrimitiveTypes.Int64},
		{Name: constants.F64_METRIC_VALUE, Type: arrow.PrimitiveTypes.Float64},
	}, []int8{
		I64Code,
		F64Code,
	})
)

// MetricValueBuilder is a helper to build an Arrow array containing a collection of OTLP metric value.
type MetricValueBuilder struct {
	released bool

	builder *array.DenseUnionBuilder // metric value builder

	i64Builder *array.Int64Builder   // int64 builder
	f64Builder *array.Float64Builder // float64 builder
}

// NewMetricValueBuilder creates a new MetricValueBuilder with a given memory allocator.
func NewMetricValueBuilder(pool memory.Allocator) *MetricValueBuilder {
	return MetricValueBuilderFrom(array.NewDenseUnionBuilder(pool, MetricValueDT))
}

// MetricValueBuilderFrom creates a new MetricValueBuilder from an existing DenseUnionBuilder.
func MetricValueBuilderFrom(mv *array.DenseUnionBuilder) *MetricValueBuilder {
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
func (b *MetricValueBuilder) Build() (*array.DenseUnion, error) {
	if b.released {
		return nil, fmt.Errorf("metric value builder already released")
	}

	defer b.Release()
	return b.builder.NewDenseUnionArray(), nil
}

// AppendNumberDataPointValue appends a new metric value to the builder.
func (b *MetricValueBuilder) AppendNumberDataPointValue(mdp pmetric.NumberDataPoint) error {
	if b.released {
		return fmt.Errorf("metric value builder already released")
	}

	var err error
	switch mdp.ValueType() {
	case pmetric.NumberDataPointValueTypeDouble:
		err = b.appendF64(mdp.DoubleValue())
	case pmetric.NumberDataPointValueTypeInt:
		err = b.appendI64(mdp.IntValue())
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
		err = b.appendF64(ex.DoubleValue())
	case pmetric.ExemplarValueTypeInt:
		err = b.appendI64(ex.IntValue())
	}
	return err
}

// Release releases the memory allocated by the builder.
func (b *MetricValueBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.i64Builder.Release()
		b.f64Builder.Release()

		b.released = true
	}
}

// appendI64 appends a new int64 value to the builder.
func (b *MetricValueBuilder) appendI64(v int64) error {
	b.builder.Append(I64Code)
	b.i64Builder.Append(v)

	return nil
}

// appendF64 appends a new double value to the builder.
func (b *MetricValueBuilder) appendF64(v float64) error {
	b.builder.Append(F64Code)
	b.f64Builder.Append(v)

	return nil
}
