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
	cfg    config        // configuration
	schema *arrow.Schema // current schema

	// list of all dictionary fields
	dictionaries map[string]*dictionaryField

	// map of dictionary fields that have overflowed (used for test purpose)
	// map = path -> dictionary index type
	dictionariesWithOverflow map[string]string
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
	// path of the dictionary field in the adaptive schema
	DictPath string
	// old dictionary type
	oldDict *arrow.DictionaryType
	// new dictionary type (promoted to a larger index type or string/binary)
	// or nil if the dictionary field has to be replaced by a string or binary.
	newDict *arrow.DictionaryType
	// new upper limit of the dictionary index
	newUpperLimit uint64
}

type config struct {
	initIndexSize  uint64
	limitIndexSize uint64
}

// Option is a function that configures the AdaptiveSchema.
type Option func(*config)

// NewAdaptiveSchema creates a new AdaptiveSchema from an [arrow.Schema]
// and a list of options.
func NewAdaptiveSchema(schema *arrow.Schema, options ...Option) *AdaptiveSchema {
	cfg := config{
		initIndexSize:  math.MaxUint16, // default to uint16
		limitIndexSize: math.MaxUint16, // default to uint16
	}
	dictionaries := make(map[string]*dictionaryField)

	for _, opt := range options {
		opt(&cfg)
	}

	schema = initSchema(schema, &cfg)

	fields := schema.Fields()
	for i := 0; i < len(fields); i++ {
		ids := []int{i}
		collectDictionaries(fields[i].Name, ids, &fields[i], &dictionaries)
	}
	return &AdaptiveSchema{cfg: cfg, schema: schema, dictionaries: dictionaries, dictionariesWithOverflow: make(map[string]string)}
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
func (m *AdaptiveSchema) Analyze(record arrow.Record) (overflowDetected bool, updates []SchemaUpdate) {
	arrays := record.Columns()
	overflowDetected = false

	for dictPath, d := range m.dictionaries {
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
			updates = append(updates, SchemaUpdate{
				DictPath:      dictPath,
				oldDict:       d.dictionary,
				newDict:       newDict,
				newUpperLimit: newUpperLimit,
			})
			if newDict == nil {
				m.dictionariesWithOverflow[d.path] = d.dictionary.ValueType.Name()
			} else {
				m.dictionariesWithOverflow[d.path] = newDict.IndexType.Name()
			}
		}
	}
	return
}

// UpdateSchema updates the schema with the provided updates.
func (m *AdaptiveSchema) UpdateSchema(updates []SchemaUpdate) {
	m.rebuildSchema(updates)

	// update dictionaries based on the updates
	for _, u := range updates {
		m.dictionaries[u.DictPath].upperLimit = u.newUpperLimit
		m.dictionaries[u.DictPath].dictionary = u.newDict
		if u.newDict == nil {
			prevDict := m.dictionaries[u.DictPath].init
			if prevDict != nil {
				prevDict.Release()
				m.dictionaries[u.DictPath].init = nil
			}
		}
	}

	// remove dictionary fields that have been replaced by string/binary
	for path, dict := range m.dictionaries {
		if dict.init == nil {
			delete(m.dictionaries, path)
		}
	}
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

// DictionariesWithOverflow returns a map of dictionary fields that have overflowed and the
// corresponding last promoted type.
func (m *AdaptiveSchema) DictionariesWithOverflow() map[string]string {
	// TODO find a less "intrusive" way to test which dictionaries have overflowed, consider how to remove test-specific functionality from the code
	return m.dictionariesWithOverflow
}

func WithDictInitIndexSize(size uint64) Option {
	return func(cfg *config) {
		cfg.initIndexSize = size
	}
}

func WithDictLimitIndexSize(size uint64) Option {
	return func(cfg *config) {
		cfg.limitIndexSize = size
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

	if upperLimit > m.cfg.limitIndexSize {
		dictType = nil
	}
	return
}

func initSchema(schema *arrow.Schema, cfg *config) *arrow.Schema {
	var indexType arrow.DataType
	switch {
	case cfg.initIndexSize == 0:
		indexType = nil
	case cfg.initIndexSize == math.MaxUint8:
		indexType = arrow.PrimitiveTypes.Uint8
	case cfg.initIndexSize == math.MaxUint16:
		indexType = arrow.PrimitiveTypes.Uint16
	case cfg.initIndexSize == math.MaxUint32:
		indexType = arrow.PrimitiveTypes.Uint32
	case cfg.initIndexSize == math.MaxUint64:
		indexType = arrow.PrimitiveTypes.Uint64
	default:
		panic("initSchema: unsupported initial index size")
	}

	oldFields := schema.Fields()
	newFields := make([]arrow.Field, len(oldFields))
	for i := 0; i < len(oldFields); i++ {
		newFields[i] = initField(&oldFields[i], indexType)
	}

	metadata := schema.Metadata()
	return arrow.NewSchema(newFields, &metadata)
}

func (m *AdaptiveSchema) rebuildSchema(updates []SchemaUpdate) {
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

func initField(f *arrow.Field, indexType arrow.DataType) arrow.Field {
	switch t := f.Type.(type) {
	case *arrow.DictionaryType:
		if indexType == nil {
			return arrow.Field{Name: f.Name, Type: t.ValueType, Nullable: f.Nullable, Metadata: f.Metadata}
		} else {
			dictType := &arrow.DictionaryType{
				IndexType: indexType,
				ValueType: t.ValueType,
				Ordered:   t.Ordered,
			}
			return arrow.Field{Name: f.Name, Type: dictType, Nullable: f.Nullable, Metadata: f.Metadata}
		}
	case *arrow.StructType:
		oldFields := t.Fields()
		newFields := make([]arrow.Field, len(oldFields))
		for i := 0; i < len(oldFields); i++ {
			newFields[i] = initField(&oldFields[i], indexType)
		}
		return arrow.Field{Name: f.Name, Type: arrow.StructOf(newFields...), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.ListType:
		elemField := t.ElemField()
		newField := initField(&elemField, indexType)
		return arrow.Field{Name: f.Name, Type: arrow.ListOf(newField.Type), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.SparseUnionType:
		oldFields := t.Fields()
		newFields := make([]arrow.Field, len(oldFields))
		for i := 0; i < len(oldFields); i++ {
			newFields[i] = initField(&oldFields[i], indexType)
		}
		return arrow.Field{Name: f.Name, Type: arrow.SparseUnionOf(newFields, t.TypeCodes()), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.DenseUnionType:
		oldFields := t.Fields()
		newFields := make([]arrow.Field, len(oldFields))
		for i := 0; i < len(oldFields); i++ {
			newFields[i] = initField(&oldFields[i], indexType)
		}
		return arrow.Field{Name: f.Name, Type: arrow.DenseUnionOf(newFields, t.TypeCodes()), Nullable: f.Nullable, Metadata: f.Metadata}
	case *arrow.MapType:
		keyField := t.KeyField()
		newKeyField := initField(&keyField, indexType)
		valueField := t.ItemField()
		newValueField := initField(&valueField, indexType)
		return arrow.Field{Name: f.Name, Type: arrow.MapOf(newKeyField.Type, newValueField.Type), Nullable: f.Nullable, Metadata: f.Metadata}
	default:
		return *f
	}
}

func updateField(f *arrow.Field, dictMap map[*arrow.DictionaryType]*arrow.DictionaryType) arrow.Field {
	switch t := f.Type.(type) {
	case *arrow.DictionaryType:
		if newDict, ok := dictMap[t]; ok {
			if newDict != nil {
				return arrow.Field{Name: f.Name, Type: newDict, Nullable: f.Nullable, Metadata: f.Metadata}
			} else {
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
func collectDictionaries(prefix string, ids []int, field *arrow.Field, dictionaries *map[string]*dictionaryField) {
	switch t := field.Type.(type) {
	case *arrow.DictionaryType:
		(*dictionaries)[prefix] = &dictionaryField{path: prefix, ids: ids, upperLimit: indexUpperLimit(t.IndexType), dictionary: field.Type.(*arrow.DictionaryType)}
	case *arrow.StructType:
		fields := t.Fields()
		for i := 0; i < len(fields); i++ {
			childIds := make([]int, len(ids)+1)
			copy(childIds, ids)
			childIds[len(ids)] = i
			collectDictionaries(prefix+"."+fields[i].Name, childIds, &fields[i], dictionaries)
		}
	case *arrow.ListType:
		field := t.ElemField()
		collectDictionaries(prefix, ids, &field, dictionaries)
	case *arrow.SparseUnionType:
		fields := t.Fields()
		for i := 0; i < len(fields); i++ {
			childIds := make([]int, len(ids)+1)
			copy(childIds, ids)
			childIds[len(ids)] = i
			collectDictionaries(prefix+"."+fields[i].Name, childIds, &fields[i], dictionaries)
		}
	case *arrow.DenseUnionType:
		fields := t.Fields()
		for i := 0; i < len(fields); i++ {
			childIds := make([]int, len(ids)+1)
			copy(childIds, ids)
			childIds[len(ids)] = i
			collectDictionaries(prefix+"."+fields[i].Name, childIds, &fields[i], dictionaries)
		}
	case *arrow.MapType:
		childIds := make([]int, len(ids)+1)
		copy(childIds, ids)
		childIds[len(ids)] = 0
		keyField := t.KeyField()
		collectDictionaries(prefix+".key", childIds, &keyField, dictionaries)

		childIds = make([]int, len(ids)+1)
		copy(childIds, ids)
		childIds[len(ids)] = 1
		itemField := t.ItemField()
		collectDictionaries(prefix+".value", childIds, &itemField, dictionaries)
	}
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

// DictionaryOverflowError is returned when the cardinality of a dictionary (or several)
// exceeds the maximum allowed value.
//
// This error is returned by the TracesBuilder.Build method. This error is retryable.
type DictionaryOverflowError struct {
	FieldNames []string
}

func (e *DictionaryOverflowError) Error() string {
	return fmt.Sprintf("dictionary overflow for fields: %v", e.FieldNames)
}
