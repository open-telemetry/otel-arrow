package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"
)

// Constants used to identify the type of value in the union.
const (
	StrCode    int8 = 0
	I64Code    int8 = 1
	F64Code    int8 = 2
	BoolCode   int8 = 3
	BinaryCode int8 = 4
	// Future extension CborCode int8 = 5
)

var (
	// AnyValueDT is an Arrow Data Type representing an OTLP Any Value.
	// Any values are represented as a sparse union of the following variants: str, i64, f64, bool, binary.
	AnyValueDT = arrow.SparseUnionOf([]arrow.Field{
		// TODO manage case where the cardinality of the dictionary is too high (> 2^16).
		{Name: "str", Type: DefaultDictString},
		{Name: "i64", Type: arrow.PrimitiveTypes.Int64},
		{Name: "f64", Type: arrow.PrimitiveTypes.Float64},
		{Name: "bool", Type: arrow.FixedWidthTypes.Boolean},
		// TODO manage case where the cardinality of the dictionary is too high (> 2^16).
		{Name: "binary", Type: DefaultDictBinary},
		// Future extension {Name: "cbor", Type: DefaultDictBinary},
	}, []int8{
		StrCode,
		I64Code,
		F64Code,
		BoolCode,
		BinaryCode,
		// Future extension CborCode,
	})
)

// AnyValueBuilder is a helper to build an Arrow array containing a collection of OTLP Any Value.
type AnyValueBuilder struct {
	released bool

	builder *array.SparseUnionBuilder // any value builder

	strBuilder    *AdaptiveDictionaryBuilder // string builder
	i64Builder    *array.Int64Builder        // int64 builder
	f64Builder    *array.Float64Builder      // float64 builder
	boolBuilder   *array.BooleanBuilder      // bool builder
	binaryBuilder *AdaptiveDictionaryBuilder // binary builder
}

// AnyValueBuilderFrom creates a new AnyValueBuilder from an existing SparseUnionBuilder.
func AnyValueBuilderFrom(av *array.SparseUnionBuilder) *AnyValueBuilder {
	return &AnyValueBuilder{
		released:      false,
		builder:       av,
		strBuilder:    AdaptiveDictionaryBuilderFrom(av.Child(0)),
		i64Builder:    av.Child(1).(*array.Int64Builder),
		f64Builder:    av.Child(2).(*array.Float64Builder),
		boolBuilder:   av.Child(3).(*array.BooleanBuilder),
		binaryBuilder: AdaptiveDictionaryBuilderFrom(av.Child(4)),
	}
}

// Build builds the "any value" Arrow array.
//
// Once the returned array is no longer needed, Release() must be called to free the
// memory allocated by the array.
func (b *AnyValueBuilder) Build() (*array.SparseUnion, error) {
	if b.released {
		return nil, fmt.Errorf("any value builder already released")
	}

	defer b.Release()
	return b.builder.NewSparseUnionArray(), nil
}

// Append appends a new any value to the builder.
func (b *AnyValueBuilder) Append(av pcommon.Value) error {
	if b.released {
		return fmt.Errorf("any value builder already released")
	}

	var err error
	switch av.Type() {
	case pcommon.ValueTypeEmpty:
		b.builder.AppendNull()
	case pcommon.ValueTypeStr:
		err = b.appendStr(av.Str())
	case pcommon.ValueTypeInt:
		err = b.appendI64(av.Int())
	case pcommon.ValueTypeDouble:
		err = b.appendF64(av.Double())
	case pcommon.ValueTypeBool:
		err = b.appendBool(av.Bool())
	case pcommon.ValueTypeBytes:
		err = b.appendBinary(av.Bytes().AsRaw())
	case pcommon.ValueTypeSlice:
		// TODO implement this with CBOR encoding
		err = fmt.Errorf("slice value type not yet supported")
	case pcommon.ValueTypeMap:
		// TODO implement this with CBOR encoding
		err = fmt.Errorf("map value type not yet supported")
	}

	return err
}

// Release releases the memory allocated by the builder.
func (b *AnyValueBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.strBuilder.Release()
		b.i64Builder.Release()
		b.f64Builder.Release()
		b.boolBuilder.Release()
		b.binaryBuilder.Release()

		b.released = true
	}
}

// appendStr appends a new string value to the builder.
func (b *AnyValueBuilder) appendStr(v string) error {
	b.builder.Append(StrCode)
	if v == "" {
		b.strBuilder.AppendNull()
	} else {
		if err := b.strBuilder.AppendString(v); err != nil {
			return err
		}
	}
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()

	return nil
}

// appendI64 appends a new int64 value to the builder.
func (b *AnyValueBuilder) appendI64(v int64) error {
	b.builder.Append(I64Code)
	b.i64Builder.Append(v)

	b.strBuilder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()

	return nil
}

// appendF64 appends a new double value to the builder.
func (b *AnyValueBuilder) appendF64(v float64) error {
	b.builder.Append(F64Code)
	b.f64Builder.Append(v)

	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()

	return nil
}

// appendBool appends a new bool value to the builder.
func (b *AnyValueBuilder) appendBool(v bool) error {
	b.builder.Append(BoolCode)
	b.boolBuilder.Append(v)

	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.binaryBuilder.AppendNull()

	return nil
}

// appendBinary appends a new binary value to the builder.
func (b *AnyValueBuilder) appendBinary(v []byte) error {
	b.builder.Append(BinaryCode)
	if v == nil {
		b.binaryBuilder.AppendNull()
	} else {
		if err := b.binaryBuilder.AppendBinary(v); err != nil {
			return err
		}
	}

	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()

	return nil
}
