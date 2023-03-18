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
	"fmt"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
)

// Constants used to identify the type of value in the union.
const (
	StrCode    int8 = 0
	I64Code    int8 = 1
	F64Code    int8 = 2
	BoolCode   int8 = 3
	BinaryCode int8 = 4
	CborCode   int8 = 5
)

var (
	// AnyValueDT is an Arrow Data Type representing an OTLP Any Value.
	// Any values are represented as a sparse union of the following variants: str, i64, f64, bool, binary.
	AnyValueDT = arrow.SparseUnionOf([]arrow.Field{
		{Name: "str", Type: arrow.BinaryTypes.String, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: "i64", Type: arrow.PrimitiveTypes.Int64, Metadata: schema.Metadata(schema.Optional)},
		{Name: "f64", Type: arrow.PrimitiveTypes.Float64, Metadata: schema.Metadata(schema.Optional)},
		{Name: "bool", Type: arrow.FixedWidthTypes.Boolean, Metadata: schema.Metadata(schema.Optional)},
		{Name: "binary", Type: arrow.BinaryTypes.Binary, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
		{Name: "cbor", Type: arrow.BinaryTypes.Binary, Metadata: schema.Metadata(schema.Optional, schema.Dictionary)},
	}, []int8{
		StrCode,
		I64Code,
		F64Code,
		BoolCode,
		BinaryCode,
		CborCode,
	})
)

// AnyValueBuilder is a helper to build an Arrow array containing a collection of OTLP Any Value.
type AnyValueBuilder struct {
	released bool

	builder *builder.SparseUnionBuilder // any value builder

	strBuilder    *builder.StringBuilder  // string builder
	i64Builder    *builder.Int64Builder   // int64 builder
	f64Builder    *builder.Float64Builder // float64 builder
	boolBuilder   *builder.BooleanBuilder // bool builder
	binaryBuilder *builder.BinaryBuilder  // binary builder
	cborBuilder   *builder.BinaryBuilder  // cbor builder
}

func ValueID(v pcommon.Value) string {
	switch v.Type() {
	case pcommon.ValueTypeStr:
		return v.Str()
	case pcommon.ValueTypeInt:
		return fmt.Sprintf("%d", v.Int())
	case pcommon.ValueTypeDouble:
		return fmt.Sprintf("%f", v.Double())
	case pcommon.ValueTypeBool:
		return fmt.Sprintf("%t", v.Bool())
	case pcommon.ValueTypeMap:
		return AttributesId(v.Map())
	case pcommon.ValueTypeBytes:
		return fmt.Sprintf("%x", v.Bytes().AsRaw())
	case pcommon.ValueTypeSlice:
		values := v.Slice()
		valueID := "["
		for i := 0; i < values.Len(); i++ {
			if len(valueID) > 1 {
				valueID += ","
			}
			valueID += ValueID(values.At(i))
		}
		valueID += "]"
		return valueID
	case pcommon.ValueTypeEmpty:
		return ""
	default:
		// includes pcommon.ValueTypeEmpty
		panic("unsupported value type")
	}
}

// AnyValueBuilderFrom creates a new AnyValueBuilder from an existing SparseUnionBuilder.
func AnyValueBuilderFrom(av *builder.SparseUnionBuilder) *AnyValueBuilder {
	return &AnyValueBuilder{
		released:      false,
		builder:       av,
		strBuilder:    av.StringBuilder(StrCode),
		i64Builder:    av.Int64Builder(I64Code),
		f64Builder:    av.Float64Builder(F64Code),
		boolBuilder:   av.BooleanBuilder(BoolCode),
		binaryBuilder: av.BinaryBuilder(BinaryCode),
		cborBuilder:   av.BinaryBuilder(CborCode),
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
		b.appendI64(av.Int())
	case pcommon.ValueTypeDouble:
		b.appendF64(av.Double())
	case pcommon.ValueTypeBool:
		b.appendBool(av.Bool())
	case pcommon.ValueTypeBytes:
		err = b.appendBinary(av.Bytes().AsRaw())
	case pcommon.ValueTypeSlice:
		cborData, err := common.Serialize(av)
		if err != nil {
			break
		}
		err = b.appendBinary(cborData)
	case pcommon.ValueTypeMap:
		cborData, err := common.Serialize(av)
		if err != nil {
			break
		}
		err = b.appendCbor(cborData)
	}

	return err
}

// Release releases the memory allocated by the builder.
func (b *AnyValueBuilder) Release() {
	if !b.released {
		b.builder.Release()

		b.released = true
	}
}

// appendStr appends a new string value to the builder.
func (b *AnyValueBuilder) appendStr(v string) error {
	b.builder.Append(StrCode)
	b.strBuilder.Append(v)
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()
	b.cborBuilder.AppendNull()

	return nil
}

// appendI64 appends a new int64 value to the builder.
func (b *AnyValueBuilder) appendI64(v int64) {
	b.builder.Append(I64Code)
	b.i64Builder.Append(v)
	b.strBuilder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()
	b.cborBuilder.AppendNull()
}

// appendF64 appends a new double value to the builder.
func (b *AnyValueBuilder) appendF64(v float64) {
	b.builder.Append(F64Code)
	b.f64Builder.Append(v)
	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()
	b.cborBuilder.AppendNull()
}

// appendBool appends a new bool value to the builder.
func (b *AnyValueBuilder) appendBool(v bool) {
	b.builder.Append(BoolCode)
	b.boolBuilder.Append(v)
	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.binaryBuilder.AppendNull()
	b.cborBuilder.AppendNull()
}

// appendBinary appends a new binary value to the builder.
func (b *AnyValueBuilder) appendBinary(v []byte) error {
	b.builder.Append(BinaryCode)
	b.binaryBuilder.Append(v)
	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.cborBuilder.AppendNull()

	return nil
}

// appendCbor appends a new cbor binary value to the builder.
func (b *AnyValueBuilder) appendCbor(v []byte) error {
	b.builder.Append(CborCode)
	b.cborBuilder.Append(v)
	b.strBuilder.AppendNull()
	b.i64Builder.AppendNull()
	b.f64Builder.AppendNull()
	b.boolBuilder.AppendNull()
	b.binaryBuilder.AppendNull()

	return nil
}
