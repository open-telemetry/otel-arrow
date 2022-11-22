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
	field2_1, _ := recbldr.Field(1).(*array.StructBuilder).FieldBuilder(0).(*array.BinaryDictionaryBuilder)
	field2_2, _ := recbldr.Field(1).(*array.StructBuilder).FieldBuilder(1).(*array.BinaryDictionaryBuilder)

	for j := 0; j < 500; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(true)
		if err := field2_1.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		if err := field2_2.Append([]byte(fmt.Sprintf(`value_%d"`, j))); err != nil {
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
	field2_1, _ := recbldr.Field(1).(*array.MapBuilder).KeyBuilder().(*array.BinaryDictionaryBuilder)
	field2_2, _ := recbldr.Field(1).(*array.MapBuilder).ItemBuilder().(*array.BinaryDictionaryBuilder)

	for j := 0; j < 500; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(true)
		if err := field2_1.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		if err := field2_2.Append([]byte(fmt.Sprintf(`value_%d"`, j))); err != nil {
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
	field2_1, _ := recbldr.Field(1).(*array.SparseUnionBuilder).Child(0).(*array.BinaryDictionaryBuilder)
	field2_2, _ := recbldr.Field(1).(*array.SparseUnionBuilder).Child(1).(*array.BinaryDictionaryBuilder)

	for j := 0; j < 500; j++ {
		if err := field1.AppendString(fmt.Sprintf(`value_%d"`, j%100)); err != nil {
			t.Fatal(err)
		}
		field2.Append(0)
		if err := field2_1.AppendString(fmt.Sprintf(`value_%d"`, j)); err != nil {
			t.Fatal(err)
		}
		field2_2.AppendNull()
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
