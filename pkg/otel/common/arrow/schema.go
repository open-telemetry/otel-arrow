package arrow

import (
	"fmt"
	"math"

	"github.com/apache/arrow/go/v11/arrow"
	"github.com/apache/arrow/go/v11/arrow/array"
)

// AdaptiveSchema is a wrapper around [arrow.Schema] that can be used to detect
// dictionary overflow and update the schema accordingly. It also maintains the
// dictionary values for each dictionary field so that the dictionary builders
// can be initialized with the initial dictionary values.
type AdaptiveSchema struct {
	cfg          config             // configuration
	schema       *arrow.Schema      // current schema
	dictionaries []*dictionaryField // list of all dictionary fields
}

type dictionaryField struct {
	path       string                // string path to the dictionary field (mostly for debugging)
	ids        []int                 // numerical path to the dictionary field (fast access)
	upperLimit uint64                // upper limit of the dictionary index
	dictionary *arrow.DictionaryType // dictionary type
	init       arrow.Array           // initial dictionary values
}

// SchemaUpdate is a struct that contains the information needed to update a schema.
// It contains the index of the dictionary field that needs to be updated, the old
// dictionary type and the new dictionary
type SchemaUpdate struct {
	// index of the dictionary field in the adaptive schema
	dictIdx int
	// old dictionary type
	oldDict *arrow.DictionaryType
	// new dictionary type (promoted to a larger index type or string/binary)
	// or nil if the dictionary field has to be replaced by a string or binary.
	newDict *arrow.DictionaryType
	// new upper limit of the dictionary index
	newUpperLimit uint64
}

type config struct {
	maxIndexSize uint64
}

// Option is a function that configures the AdaptiveSchema.
type Option func(*config)

// NewAdaptiveSchema creates a new AdaptiveSchema from an [arrow.Schema]
// and a list of options.
func NewAdaptiveSchema(schema *arrow.Schema, options ...Option) *AdaptiveSchema {
	cfg := config{
		maxIndexSize: math.MaxUint16, // default to uint16
	}
	var dictionaries []*dictionaryField

	for _, opt := range options {
		opt(&cfg)
	}

	fields := schema.Fields()
	for i := 0; i < len(fields); i++ {
		ids := []int{i}
		dictionaries = append(dictionaries, collectDictionaries(fields[i].Name, ids, &fields[i], dictionaries)...)
	}
	return &AdaptiveSchema{cfg: cfg, schema: schema, dictionaries: dictionaries}
}

// Schema returns the current schema.
func (m *AdaptiveSchema) Schema() *arrow.Schema {
	return m.schema
}

// Analyze detects if any of the dictionary fields in the schema have
// overflowed and returns a list of updates that need to be applied to
// the schema. The content of each dictionary array (unique values) is
// also stored in the AdaptiveSchema so that the dictionary builders
// can be initialized with the initial dictionary values.
//
// Returns true if any of the dictionaries have overflowed and false
// otherwise.
func (m *AdaptiveSchema) Analyze(record arrow.Record) (overflowDetected bool, updates []*SchemaUpdate) {
	arrays := record.Columns()
	overflowDetected = false

	for dictIdx, d := range m.dictionaries {
		dict := getDictionaryArray(arrays[d.ids[0]], d.ids[1:])
		if d.init != nil {
			d.init.Release()
		}
		d.init = dict.Dictionary()
		d.init.Retain()
		observedSize := uint64(d.init.Len())
		if observedSize > d.upperLimit {
			overflowDetected = true
			newDict, newUpperLimit := m.promoteDictionaryType(observedSize, d.dictionary)
			updates = append(updates, &SchemaUpdate{
				dictIdx:       dictIdx,
				oldDict:       d.dictionary,
				newDict:       newDict,
				newUpperLimit: newUpperLimit,
			})
			if newDict == nil {
				println("replacing dictionary field  `" + d.path + "` with string/binary")
			} else {
				println("overflow detected for field `" + d.path + "` promoted to " + newDict.IndexType.Name())
			}
		}
	}
	return
}

// UpdateSchema updates the schema with the provided updates.
func (m *AdaptiveSchema) UpdateSchema(updates []*SchemaUpdate) {
	m.rebuildSchema(updates)

	// update dictionaries based on the updates
	newDicts := make([]*dictionaryField, 0, len(m.dictionaries))
	for _, u := range updates {
		if u.newDict != nil {
			m.dictionaries[u.dictIdx].upperLimit = u.newUpperLimit
			m.dictionaries[u.dictIdx].dictionary = u.newDict
			newDicts = append(newDicts, m.dictionaries[u.dictIdx])
		}
	}
	m.dictionaries = newDicts
}

// InitDictionaryBuilders initializes the dictionary builders with the initial dictionary values
// extracted for the previous processed records.
func (m *AdaptiveSchema) InitDictionaryBuilders(builder *array.RecordBuilder) (err error) {
	builders := builder.Fields()
	for _, d := range m.dictionaries {
		dict := getDictionaryBuilder(builders[d.ids[0]], d.ids[1:])
		if d.init != nil {
			switch init := d.init.(type) {
			case *array.String:
				err = dict.(*array.BinaryDictionaryBuilder).InsertStringDictValues(init)
			case *array.Binary:
				err = dict.(*array.BinaryDictionaryBuilder).InsertDictValues(init)
			case *array.FixedSizeBinary:
				err = dict.(*array.FixedSizeBinaryDictionaryBuilder).InsertDictValues(init)
			default:
				panic("InitDictionaryBuilders: unsupported dictionary type " + init.DataType().Name())
			}
			if err != nil {
				return
			}
		}
	}
	return
}

// Release releases all the dictionary arrays that were stored in the AdaptiveSchema.
func (m *AdaptiveSchema) Release() {
	for _, d := range m.dictionaries {
		if d.init != nil {
			d.init.Release()
		}
	}
}

func WithUint8MaxIndexSize() Option {
	return func(cfg *config) {
		cfg.maxIndexSize = math.MaxUint8
	}
}

func WithUint16MaxIndexSize() Option {
	return func(cfg *config) {
		cfg.maxIndexSize = math.MaxUint16
	}
}

func WithUint32MaxIndexSize() Option {
	return func(cfg *config) {
		cfg.maxIndexSize = math.MaxUint32
	}
}

func WithUint64MaxIndexSize() Option {
	return func(cfg *config) {
		cfg.maxIndexSize = math.MaxUint64
	}
}

func (m *AdaptiveSchema) promoteDictionaryType(observedSize uint64, existingDT *arrow.DictionaryType) (dictType *arrow.DictionaryType, upperLimit uint64) {
	if observedSize <= math.MaxUint8 {
		dictType = &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint8,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
		upperLimit = math.MaxUint8
	} else if observedSize <= math.MaxUint16 {
		dictType = &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint16,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
		upperLimit = math.MaxUint16
	} else if observedSize <= math.MaxUint32 {
		dictType = &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint32,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
		upperLimit = math.MaxUint32
	} else {
		dictType = &arrow.DictionaryType{
			IndexType: arrow.PrimitiveTypes.Uint64,
			ValueType: existingDT.ValueType,
			Ordered:   false,
		}
		upperLimit = math.MaxUint64
	}

	if upperLimit > m.cfg.maxIndexSize {
		dictType = nil
	}
	return
}

func (m *AdaptiveSchema) rebuildSchema(updates []*SchemaUpdate) {
	// Mapping old dictionary type to new dictionary type
	// Used to identify the dictionary builders that need to be updated
	oldToNewDicts := make(map[*arrow.DictionaryType]*arrow.DictionaryType)
	for _, u := range updates {
		oldToNewDicts[u.oldDict] = u.newDict
	}

	oldFields := m.schema.Fields()
	newFields := make([]arrow.Field, len(oldFields))
	for i := 0; i < len(oldFields); i++ {
		newFields[i] = updateField(&oldFields[i], oldToNewDicts)
	}

	metadata := m.schema.Metadata()
	m.schema = arrow.NewSchema(newFields, &metadata)
}

func updateField(f *arrow.Field, dictMap map[*arrow.DictionaryType]*arrow.DictionaryType) arrow.Field {
	switch t := f.Type.(type) {
	case *arrow.DictionaryType:
		if newDict, ok := dictMap[t]; ok {
			if newDict != nil {
				return arrow.Field{Name: f.Name, Type: newDict, Nullable: f.Nullable, Metadata: f.Metadata}
			} else {
				fmt.Printf("updateField: replacing dictionary field %q with string or binary field\n", f.Name)
				return arrow.Field{Name: f.Name, Type: t.ValueType, Nullable: f.Nullable, Metadata: f.Metadata}
			}
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

func getDictionaryArray(arr arrow.Array, ids []int) *array.Dictionary {
	if len(ids) == 0 {
		return arr.(*array.Dictionary)
	}

	switch arr := arr.(type) {
	case *array.Struct:
		return getDictionaryArray(arr.Field(ids[0]), ids[1:])
	case *array.List:
		return getDictionaryArray(arr.ListValues(), ids)
	case *array.SparseUnion:
		return getDictionaryArray(arr.Field(ids[0]), ids[1:])
	case *array.DenseUnion:
		return getDictionaryArray(arr.Field(ids[0]), ids[1:])
	case *array.Map:
		switch ids[0] {
		case 0: // key
			return getDictionaryArray(arr.Keys(), ids[1:])
		case 1: // value
			return getDictionaryArray(arr.Items(), ids[1:])
		default:
			panic("getDictionaryArray: invalid map field id")
		}
	default:
		panic("getDictionaryArray: unsupported array type `" + arr.DataType().Name() + "`")
	}
}

func getDictionaryBuilder(builder array.Builder, ids []int) array.DictionaryBuilder {
	if len(ids) == 0 {
		return builder.(array.DictionaryBuilder)
	}

	switch b := builder.(type) {
	case *array.StructBuilder:
		return getDictionaryBuilder(b.FieldBuilder(ids[0]), ids[1:])
	case *array.ListBuilder:
		return getDictionaryBuilder(b.ValueBuilder(), ids)
	case *array.SparseUnionBuilder:
		return getDictionaryBuilder(b.Child(ids[0]), ids[1:])
	case *array.DenseUnionBuilder:
		return getDictionaryBuilder(b.Child(ids[0]), ids[1:])
	case *array.MapBuilder:
		switch ids[0] {
		case 0: // key
			return getDictionaryBuilder(b.KeyBuilder(), ids[1:])
		case 1: // value
			return getDictionaryBuilder(b.ItemBuilder(), ids[1:])
		default:
			panic("getDictionaryBuilder: invalid map field id")
		}
	default:
		panic("getDictionaryBuilder: unsupported array type `" + b.Type().Name() + "`")
	}
}

// collectDictionaries collects recursively all dictionary fields in the schema and returns a list of them.
func collectDictionaries(prefix string, ids []int, field *arrow.Field, dictionaries []*dictionaryField) []*dictionaryField {
	switch t := field.Type.(type) {
	case *arrow.DictionaryType:
		dictionaries = append(dictionaries, &dictionaryField{path: prefix, ids: ids, upperLimit: indexUpperLimit(t.IndexType), dictionary: field.Type.(*arrow.DictionaryType)})
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
