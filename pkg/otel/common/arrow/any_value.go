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

	"github.com/HdrHistogram/hdrhistogram-go"
	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"go.opentelemetry.io/collector/pdata/pcommon"

	"github.com/f5/otel-arrow-adapter/pkg/otel/common"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/schema/builder"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
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

type AnyValueStats struct {
	StrHistogram    *hdrhistogram.Histogram
	I64Histogram    *hdrhistogram.Histogram
	F64Histogram    *hdrhistogram.Histogram
	BoolHistogram   *hdrhistogram.Histogram
	BinaryHistogram *hdrhistogram.Histogram
	ListHistogram   *hdrhistogram.Histogram
	MapHistogram    *hdrhistogram.Histogram
}

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
		return nil, werror.Wrap(ErrBuilderAlreadyReleased)
	}

	defer b.Release()
	return b.builder.NewSparseUnionArray(), nil
}

// Append appends a new any value to the builder.
func (b *AnyValueBuilder) Append(av pcommon.Value) error {
	if b.released {
		return werror.Wrap(ErrBuilderAlreadyReleased)
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

	return werror.Wrap(err)
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

func NewAnyValueStats() *AnyValueStats {
	return &AnyValueStats{
		StrHistogram:    hdrhistogram.New(0, 100000, 1),
		I64Histogram:    hdrhistogram.New(0, 100000, 1),
		F64Histogram:    hdrhistogram.New(0, 100000, 1),
		BoolHistogram:   hdrhistogram.New(0, 100000, 1),
		BinaryHistogram: hdrhistogram.New(0, 100000, 1),
		ListHistogram:   hdrhistogram.New(0, 100000, 1),
		MapHistogram:    hdrhistogram.New(0, 100000, 1),
	}
}

func (a *AnyValueStats) UpdateStats(v pcommon.Value) {
	var strCount int64
	var i64Count int64
	var f64Count int64
	var boolCount int64
	var binaryCount int64
	var listCount int64
	var mapCount int64

	switch v.Type() {
	case pcommon.ValueTypeStr:
		strCount++
	case pcommon.ValueTypeInt:
		i64Count++
	case pcommon.ValueTypeDouble:
		f64Count++
	case pcommon.ValueTypeBool:
		boolCount++
	case pcommon.ValueTypeBytes:
		binaryCount++
	case pcommon.ValueTypeSlice:
		listCount++
	case pcommon.ValueTypeMap:
		mapCount++
	default: // ignore
	}

	if err := a.StrHistogram.RecordValue(strCount); err != nil {
		panic(fmt.Sprintf("failed to record str count: %v", err))
	}
	if err := a.I64Histogram.RecordValue(i64Count); err != nil {
		panic(fmt.Sprintf("failed to record i64 count: %v", err))
	}
	if err := a.F64Histogram.RecordValue(f64Count); err != nil {
		panic(fmt.Sprintf("failed to record f64 count: %v", err))
	}
	if err := a.BoolHistogram.RecordValue(boolCount); err != nil {
		panic(fmt.Sprintf("failed to record bool count: %v", err))
	}
	if err := a.BinaryHistogram.RecordValue(binaryCount); err != nil {
		panic(fmt.Sprintf("failed to record binary count: %v", err))
	}
	if err := a.ListHistogram.RecordValue(listCount); err != nil {
		panic(fmt.Sprintf("failed to record list count: %v", err))
	}
	if err := a.MapHistogram.RecordValue(mapCount); err != nil {
		panic(fmt.Sprintf("failed to record map count: %v", err))
	}
}

func (a *AnyValueStats) Show(prefix string) {
	if a.StrHistogram.Mean() > 0 {
		fmt.Printf("%sstr      -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.StrHistogram.Mean(), a.StrHistogram.Min(), a.StrHistogram.Max(), a.StrHistogram.StdDev(), a.StrHistogram.ValueAtQuantile(50), a.StrHistogram.ValueAtQuantile(99))
	}
	if a.I64Histogram.Mean() > 0 {
		fmt.Printf("%si64      -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.I64Histogram.Mean(), a.I64Histogram.Min(), a.I64Histogram.Max(), a.I64Histogram.StdDev(), a.I64Histogram.ValueAtQuantile(50), a.I64Histogram.ValueAtQuantile(99))
	}
	if a.F64Histogram.Mean() > 0 {
		fmt.Printf("%sf64      -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.F64Histogram.Mean(), a.F64Histogram.Min(), a.F64Histogram.Max(), a.F64Histogram.StdDev(), a.F64Histogram.ValueAtQuantile(50), a.F64Histogram.ValueAtQuantile(99))
	}
	if a.BoolHistogram.Mean() > 0 {
		fmt.Printf("%sbool     -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.BoolHistogram.Mean(), a.BoolHistogram.Min(), a.BoolHistogram.Max(), a.BoolHistogram.StdDev(), a.BoolHistogram.ValueAtQuantile(50), a.BoolHistogram.ValueAtQuantile(99))
	}
	if a.BinaryHistogram.Mean() > 0 {
		fmt.Printf("%sbinary   -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.BinaryHistogram.Mean(), a.BinaryHistogram.Min(), a.BinaryHistogram.Max(), a.BinaryHistogram.StdDev(), a.BinaryHistogram.ValueAtQuantile(50), a.BinaryHistogram.ValueAtQuantile(99))
	}
	if a.ListHistogram.Mean() > 0 {
		fmt.Printf("%slist     -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.ListHistogram.Mean(), a.ListHistogram.Min(), a.ListHistogram.Max(), a.ListHistogram.StdDev(), a.ListHistogram.ValueAtQuantile(50), a.ListHistogram.ValueAtQuantile(99))
	}
	if a.MapHistogram.Mean() > 0 {
		fmt.Printf("%smap      -> mean: %8.2f, min: %8d, max: %8d, std-dev: %8.2f, p50: %8d, p99: %8d\n",
			prefix, a.MapHistogram.Mean(), a.MapHistogram.Min(), a.MapHistogram.Max(), a.MapHistogram.StdDev(), a.MapHistogram.ValueAtQuantile(50), a.MapHistogram.ValueAtQuantile(99))
	}
}
