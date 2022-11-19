package arrow

import (
	"math"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
)

// AdaptiveSchema is a wrapper around arrow.Schema that can be used to detect
// dictionary overflow and update the schema accordingly.
type AdaptiveSchema struct {
	schema       *arrow.Schema
	dictionaries []dictionaryField
}

type dictionaryField struct {
	name       string
	ids        []int
	upperLimit uint64
	dictionary *arrow.DictionaryType
}

// SchemaUpdate is a struct that contains the information needed to update a schema.
// It contains the index of the dictionary field that needs to be updated, the old
// dictionary type and the new dictionary
type SchemaUpdate struct {
	dictIdx int
	oldDict *arrow.DictionaryType
	newDict *arrow.DictionaryType
}

// NewAdaptiveSchema creates a new AdaptiveSchema from an arrow.Schema.
func NewAdaptiveSchema(schema *arrow.Schema) *AdaptiveSchema {
	var dictionaries []dictionaryField
	fields := schema.Fields()
	for i := 0; i < len(fields); i++ {
		ids := []int{i}
		dictionaries = append(dictionaries, collectDictionaries(fields[i].Name, ids, &fields[i], dictionaries)...)
	}
	return &AdaptiveSchema{schema: schema, dictionaries: dictionaries}
}

// Schema returns the current schema.
func (m *AdaptiveSchema) Schema() *arrow.Schema {
	return m.schema
}

// DetectDictionaryOverflow detects if any of the dictionary fields in the schema
// have overflowed and returns a list of updates that need to be applied to the schema.
// Returns true if any of the dictionaries have overflowed and false otherwise.
func (m *AdaptiveSchema) DetectDictionaryOverflow(record arrow.Record) (overflowDetected bool, updates []*SchemaUpdate) {
	arrays := record.Columns()
	overflowDetected = false

	for dictIdx, d := range m.dictionaries {
		dict := getDictionary(arrays[d.ids[0]], d.ids[1:])
		observedSize := uint64(dict.Dictionary().Len())
		if observedSize > d.upperLimit {
			overflowDetected = true
			updates = append(updates, &SchemaUpdate{
				dictIdx: dictIdx,
				oldDict: d.dictionary,
				newDict: promoteDictionaryType(observedSize, d.dictionary),
			})
			println("overflow detected for field `" + d.name + "`")
		}
	}
	return
}

// UpdateSchema updates the schema with the provided updates.
func (m *AdaptiveSchema) UpdateSchema(updates []*SchemaUpdate) {
	for _, u := range updates {
		m.dictionaries[u.dictIdx].dictionary = u.newDict
	}
	m.schema = rebuildSchema(m.schema, updates)
}

func rebuildSchema(schema *arrow.Schema, updates []*SchemaUpdate) *arrow.Schema {
	oldToNewDicts := make(map[*arrow.DictionaryType]*arrow.DictionaryType)
	for _, u := range updates {
		oldToNewDicts[u.oldDict] = u.newDict
	}
	oldFields := schema.Fields()
	newFields := make([]arrow.Field, len(oldFields))
	for i := 0; i < len(oldFields); i++ {
		newFields[i] = updateField(&oldFields[i], oldToNewDicts)
	}
	metadata := schema.Metadata()
	return arrow.NewSchema(newFields, &metadata)
}

func updateField(f *arrow.Field, dictMap map[*arrow.DictionaryType]*arrow.DictionaryType) arrow.Field {
	switch t := f.Type.(type) {
	case *arrow.DictionaryType:
		if newDict, ok := dictMap[t]; ok {
			return arrow.Field{Name: f.Name, Type: newDict, Nullable: f.Nullable, Metadata: f.Metadata}
		} else {
			return *f
		}
	case *arrow.StructType:
		oldFields := t.Fields()
		newFields := make([]arrow.Field, len(oldFields))
		for i := 0; i < len(oldFields); i++ {
			newFields[i] = updateField(&oldFields[i], dictMap)
		}
		return arrow.Field{Name: f.Name, Type: arrow.StructOf(newFields...), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.ListType:
		elemField := t.ElemField()
		newField := updateField(&elemField, dictMap)
		return arrow.Field{Name: f.Name, Type: arrow.ListOf(newField.Type), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.SparseUnionType:
		oldFields := t.Fields()
		newFields := make([]arrow.Field, len(oldFields))
		for i := 0; i < len(oldFields); i++ {
			newFields[i] = updateField(&oldFields[i], dictMap)
		}
		return arrow.Field{Name: f.Name, Type: arrow.SparseUnionOf(newFields, t.TypeCodes()), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.DenseUnionType:
		oldFields := t.Fields()
		newFields := make([]arrow.Field, len(oldFields))
		for i := 0; i < len(oldFields); i++ {
			newFields[i] = updateField(&oldFields[i], dictMap)
		}
		return arrow.Field{Name: f.Name, Type: arrow.DenseUnionOf(newFields, t.TypeCodes()), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.MapType:
		keyField := t.KeyField()
		newKeyField := updateField(&keyField, dictMap)
		valueField := t.ItemField()
		newValueField := updateField(&valueField, dictMap)
		return arrow.Field{Name: f.Name, Type: arrow.MapOf(newKeyField.Type, newValueField.Type), Nullable: f.Nullable, Metadata: f.Metadata}
	default:
		return *f
	}
}

func getDictionary(arr arrow.Array, ids []int) *array.Dictionary {
	if len(ids) == 0 {
		return arr.(*array.Dictionary)
	}

	switch arr := arr.(type) {
	case *array.Struct:
		return getDictionary(arr.Field(ids[0]), ids[1:])
	case *array.List:
		return getDictionary(arr.ListValues(), ids)
	case *array.SparseUnion:
		return getDictionary(arr.Field(ids[0]), ids[1:])
	case *array.DenseUnion:
		return getDictionary(arr.Field(ids[0]), ids[1:])
	case *array.Map:
		switch ids[0] {
		case 0: // key
			return getDictionary(arr.Keys(), ids[1:])
		case 1: // value
			return getDictionary(arr.Items(), ids[1:])
		default:
			panic("getDictionary: invalid map field id")
		}
	default:
		panic("unsupported array type `" + arr.DataType().Name() + "`")
	}
}

// collectDictionaries collects recursively all dictionary fields in the schema and returns a list of them.
func collectDictionaries(prefix string, ids []int, field *arrow.Field, dictionaries []dictionaryField) []dictionaryField {
	switch t := field.Type.(type) {
	case *arrow.DictionaryType:
		dictionaries = append(dictionaries, dictionaryField{name: prefix, ids: ids, upperLimit: indexUpperLimit(t.IndexType), dictionary: field.Type.(*arrow.DictionaryType)})
	case *arrow.StructType:
		fields := t.Fields()
		for i := 0; i < len(fields); i++ {
			childIds := make([]int, len(ids)+1)
			copy(childIds, ids)
			childIds[len(ids)] = i
			dictionaries = collectDictionaries(prefix+"."+fields[i].Name, childIds, &fields[i], dictionaries)
		}
	case *arrow.ListType:
		field := t.ElemField()
		dictionaries = collectDictionaries(prefix, ids, &field, dictionaries)
	case *arrow.SparseUnionType:
		fields := t.Fields()
		for i := 0; i < len(fields); i++ {
			childIds := make([]int, len(ids)+1)
			copy(childIds, ids)
			childIds[len(ids)] = i
			dictionaries = collectDictionaries(prefix+"."+fields[i].Name, childIds, &fields[i], dictionaries)
		}
	case *arrow.DenseUnionType:
		fields := t.Fields()
		for i := 0; i < len(fields); i++ {
			childIds := make([]int, len(ids)+1)
			copy(childIds, ids)
			childIds[len(ids)] = i
			dictionaries = collectDictionaries(prefix+"."+fields[i].Name, childIds, &fields[i], dictionaries)
		}
	case *arrow.MapType:
		childIds := make([]int, len(ids)+1)
		copy(childIds, ids)
		childIds[len(ids)] = 0
		keyField := t.KeyField()
		dictionaries = collectDictionaries(prefix+".key", childIds, &keyField, dictionaries)

		childIds = make([]int, len(ids)+1)
		copy(childIds, ids)
		childIds[len(ids)] = 1
		itemField := t.ItemField()
		dictionaries = collectDictionaries(prefix+".value", childIds, &itemField, dictionaries)
	}

	return dictionaries
}

func indexUpperLimit(dt arrow.DataType) uint64 {
	switch dt {
	case arrow.PrimitiveTypes.Uint8:
		return math.MaxUint8
	case arrow.PrimitiveTypes.Uint16:
		return math.MaxUint16
	case arrow.PrimitiveTypes.Uint32:
		return math.MaxUint32
	case arrow.PrimitiveTypes.Uint64:
		return math.MaxUint64
	case arrow.PrimitiveTypes.Int8:
		return math.MaxInt8
	case arrow.PrimitiveTypes.Int16:
		return math.MaxInt16
	case arrow.PrimitiveTypes.Int32:
		return math.MaxInt32
	case arrow.PrimitiveTypes.Int64:
		return math.MaxInt64
	default:
		panic("unsupported index type `" + dt.Name() + "`")
	}
}

func promoteDictionaryType(observedSize uint64, existingDT *arrow.DictionaryType) *arrow.DictionaryType {
	if observedSize <= math.MaxUint8 {
		return &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
	} else if observedSize <= math.MaxUint16 {
		return &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint16,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
	} else if observedSize <= math.MaxUint32 {
		return &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint32,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
	} else {
		return &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint64,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
	}
}
