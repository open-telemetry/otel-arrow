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
	"math"
	"testing"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
)

func TestSchemaWithStruct(t *testing.T) {
	t.Parallel()

	schema := arrow.NewSchema([]arrow.Field{
		{Name: "field1", Type: &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: arrow.BinaryTypes.String,
			Ordered:   false,
		}},
		{Name: "field2", Type: arrow.StructOf(
			arrow.Field{Name: "field2_1", Type: &arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8,
				ValueType: arrow.BinaryTypes.String,
				Ordered:   false,
			}},
			arrow.Field{Name: "field2_2", Type: &arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8,
				ValueType: arrow.BinaryTypes.Binary,
				Ordered:   false,
			}},
		)},
	}, nil)

	pool := memory.NewGoAllocator()
	sm := NewAdaptiveSchema(schema, WithDictInitIndexSize(math.MaxUint8))
	recbldr := array.NewRecordBuilder(pool, sm.Schema())
	field1, _ := recbldr.Field(0).(*array.BinaryDictionaryBuilder)
	field2, _ := recbldr.Field(1).(*array.StructBuilder)
	field21, _ := recbldr.Field(1).(*array.StructBuilder).FieldBuilder(0).(*array.BinaryDictionaryBuilder)
	field22, _ := recbldr.Field(1).(*array.StructBuilder).FieldBuilder(1).(*array.BinaryDictionaryBuilder)

	for j := 0; j < 500; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(true)
		if err := field21.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		if err := field22.Append([]byte(fmt.Sprintf(`value_%d"`, j))); err != nil {
			t.Fatal(err)
		}
	}

	record := recbldr.NewRecord()

	overflowDetected, schemaUpdates := sm.Analyze(record)
	require.True(t, overflowDetected)
	require.Len(t, schemaUpdates, 2)

	sm.UpdateSchema(schemaUpdates)

	newSchema := sm.Schema()
	require.Equal(t, newSchema.Field(0).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint8)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.StructType).Field(0).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.StructType).Field(1).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
}

func TestSchemaWithMap(t *testing.T) {
	t.Parallel()

	schema := arrow.NewSchema([]arrow.Field{
		{Name: "field1", Type: &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: arrow.BinaryTypes.String,
			Ordered:   false,
		}},
		{Name: "field2", Type: arrow.MapOf(
			&arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8,
				ValueType: arrow.BinaryTypes.String,
				Ordered:   false,
			},
			&arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8,
				ValueType: arrow.BinaryTypes.Binary,
				Ordered:   false,
			},
		)},
	}, nil)

	pool := memory.NewGoAllocator()
	sm := NewAdaptiveSchema(schema, WithDictInitIndexSize(math.MaxUint8))
	recbldr := array.NewRecordBuilder(pool, sm.Schema())
	field1, _ := recbldr.Field(0).(*array.BinaryDictionaryBuilder)
	field2, _ := recbldr.Field(1).(*array.MapBuilder)
	field21, _ := recbldr.Field(1).(*array.MapBuilder).KeyBuilder().(*array.BinaryDictionaryBuilder)
	field22, _ := recbldr.Field(1).(*array.MapBuilder).ItemBuilder().(*array.BinaryDictionaryBuilder)

	for j := 0; j < 500; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(true)
		if err := field21.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		if err := field22.Append([]byte(fmt.Sprintf(`value_%d"`, j))); err != nil {
			t.Fatal(err)
		}
	}

	record := recbldr.NewRecord()

	overflowDetected, schemaUpdates := sm.Analyze(record)
	require.True(t, overflowDetected)
	require.Len(t, schemaUpdates, 2)

	sm.UpdateSchema(schemaUpdates)

	newSchema := sm.Schema()
	require.Equal(t, newSchema.Field(0).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint8)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.MapType).KeyField().Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.MapType).ItemField().Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
}

func TestSchemaWithUnion(t *testing.T) {
	t.Parallel()

	schema := arrow.NewSchema([]arrow.Field{
		{Name: "field1", Type: &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: arrow.BinaryTypes.String,
			Ordered:   false,
		}},
		{Name: "field2", Type: arrow.SparseUnionOf(
			[]arrow.Field{
				{Name: "field2_1", Type: &arrow.DictionaryType{
					IndexType: arrow.PrimitiveTypes.Uint8,
					ValueType: arrow.BinaryTypes.String,
					Ordered:   false,
				}},
				{Name: "field2_2", Type: &arrow.DictionaryType{
					IndexType: arrow.PrimitiveTypes.Uint8,
					ValueType: arrow.BinaryTypes.Binary,
					Ordered:   false,
				}},
			},
			[]arrow.UnionTypeCode{0, 1},
		)},
	}, nil)

	pool := memory.NewGoAllocator()
	sm := NewAdaptiveSchema(schema, WithDictInitIndexSize(math.MaxUint8))
	recbldr := array.NewRecordBuilder(pool, sm.Schema())
	field1, _ := recbldr.Field(0).(*array.BinaryDictionaryBuilder)
	field2, _ := recbldr.Field(1).(*array.SparseUnionBuilder)
	field21, _ := recbldr.Field(1).(*array.SparseUnionBuilder).Child(0).(*array.BinaryDictionaryBuilder)
	field22, _ := recbldr.Field(1).(*array.SparseUnionBuilder).Child(1).(*array.BinaryDictionaryBuilder)

	for j := 0; j < 500; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(0)
		if err := field21.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		field22.AppendNull()
	}

	record := recbldr.NewRecord()

	overflowDetected, schemaUpdates := sm.Analyze(record)
	require.True(t, overflowDetected)
	require.Len(t, schemaUpdates, 1)

	sm.UpdateSchema(schemaUpdates)

	newSchema := sm.Schema()
	require.Equal(t, newSchema.Field(0).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint8)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.SparseUnionType).Fields()[0].Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.SparseUnionType).Fields()[1].Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint8)
}

func TestBuilderCapacityWindow(t *testing.T) {
	window := NewBuilderCapacityWindow(10)

	for i := 0; i < 100; i++ {
		window.Record(10 * i)
		require.Equal(t, 10*i, window.Max())
	}
}

func TestInitSizeBuilders(t *testing.T) {
	t.Parallel()

	schema := arrow.NewSchema([]arrow.Field{
		{Name: "field1", Type: &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: arrow.BinaryTypes.String,
			Ordered:   false,
		}},
		{Name: "field2", Type: arrow.StructOf(
			arrow.Field{Name: "field2_1", Type: &arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8,
				ValueType: arrow.BinaryTypes.String,
				Ordered:   false,
			}},
			arrow.Field{Name: "field2_2", Type: &arrow.DictionaryType{
				IndexType: arrow.PrimitiveTypes.Uint8,
				ValueType: arrow.BinaryTypes.Binary,
				Ordered:   false,
			}},
		)},
	}, nil)

	pool := memory.NewGoAllocator()
	sm := NewAdaptiveSchema(schema, WithDictInitIndexSize(math.MaxUint8))
	recordBuilder := array.NewRecordBuilder(pool, sm.Schema())
	err := sm.InitDictionaryBuilders(recordBuilder)
	require.NoError(t, err)

	field1, _ := recordBuilder.Field(0).(*array.BinaryDictionaryBuilder)
	field2, _ := recordBuilder.Field(1).(*array.StructBuilder)
	field21, _ := recordBuilder.Field(1).(*array.StructBuilder).FieldBuilder(0).(*array.BinaryDictionaryBuilder)
	field22, _ := recordBuilder.Field(1).(*array.StructBuilder).FieldBuilder(1).(*array.BinaryDictionaryBuilder)

	require.Equal(t, 0, field1.Cap())
	require.Equal(t, 0, field2.Cap())
	require.Equal(t, 0, field21.Cap())
	require.Equal(t, 0, field22.Cap())

	for j := 0; j < 127; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(true)
		if err := field21.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		if err := field22.Append([]byte(fmt.Sprintf(`value_%d"`, j))); err != nil {
			t.Fatal(err)
		}
	}

	record := recordBuilder.NewRecord()

	// After this point, the builders should have a size based on the previous batch size, i.e. the next
	// power of 2 after 127 (127 being the last batch size).
	sm.Analyze(record)

	recordBuilder = array.NewRecordBuilder(pool, sm.Schema())
	err = sm.InitDictionaryBuilders(recordBuilder)
	require.NoError(t, err)

	require.Equal(t, 128, recordBuilder.Field(0).(*array.BinaryDictionaryBuilder).Cap())
	require.Equal(t, 128, recordBuilder.Field(1).(*array.StructBuilder).Cap())
	require.Equal(t, 128, recordBuilder.Field(1).(*array.StructBuilder).FieldBuilder(0).(*array.BinaryDictionaryBuilder).Cap())
	require.Equal(t, 128, recordBuilder.Field(1).(*array.StructBuilder).FieldBuilder(1).(*array.BinaryDictionaryBuilder).Cap())
}
