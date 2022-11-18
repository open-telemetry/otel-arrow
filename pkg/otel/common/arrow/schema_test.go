package arrow

import (
	"fmt"
	"testing"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
	"github.com/apache/arrow/go/v11/arrow/memory"
	"github.com/stretchr/testify/require"
)

func TestSchema(t *testing.T) {
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
	sm := NewAdaptiveSchema(schema)
	recbldr := array.NewRecordBuilder(pool, sm.Schema())
	field1 := recbldr.Field(0).(*array.BinaryDictionaryBuilder)
	field2 := recbldr.Field(1).(*array.StructBuilder)
	field2_1 := recbldr.Field(1).(*array.StructBuilder).FieldBuilder(0).(*array.BinaryDictionaryBuilder)
	field2_2 := recbldr.Field(1).(*array.StructBuilder).FieldBuilder(1).(*array.BinaryDictionaryBuilder)

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

	overflowDetected, schemaUpdates := sm.DetectDictionaryOverflow(record)
	require.True(t, overflowDetected)
	require.Len(t, schemaUpdates, 2)

	sm.UpdateSchema(schemaUpdates)

	newSchema := sm.Schema()
	require.Equal(t, newSchema.Field(0).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint8)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.StructType).Field(0).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
	require.Equal(t, newSchema.Field(1).Type.(*arrow.StructType).Field(1).Type.(*arrow.DictionaryType).IndexType, arrow.PrimitiveTypes.Uint16)
}
