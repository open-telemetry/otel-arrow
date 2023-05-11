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

package tmo

import (
	"bytes"
	"sort"
	"strings"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
	"github.com/apache/arrow/go/v12/arrow/memory"
	"github.com/axiomhq/hyperloglog"
	"go.opentelemetry.io/collector/pdata/pcommon"

	arrow2 "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type (
	DynAttrsBuilder struct {
		mem         memory.Allocator
		payloadType *arrow2.PayloadType

		newColumn         bool
		schemaID          string
		schemaUpdateCount int
		counter           int // todo remove

		parentGroups   map[string][]int
		parentIDColumn *ParentIDColumn

		colIdx  map[string]int
		columns []AttrColumn

		builder *array.RecordBuilder
	}

	ParentIDColumn struct {
		colName string
		values  []uint32
		builder *array.Uint32Builder
	}

	AttrColumn interface {
		ColName() string
		ColType() arrow.DataType
		Append(parentGroup string, v pcommon.Value)
		AppendNull(parentGroup string)
		Len() int
		SetBuilder(builder array.Builder)
		Build(rowIndices []int) error
		Cardinality(parentGroup string) int
		Compare(i, j int) int
		Reset()
	}

	BoolAttrCard struct {
		nullCount  int
		trueCount  int
		falseCount int
	}

	BoolAttrColumn struct {
		colName string
		values  []*bool
		builder *array.BooleanBuilder
		card    map[string]*BoolAttrCard
	}

	IntAttrCard struct {
		nullCount int
		card      map[int64]bool
	}

	IntAttrColumn struct {
		colName string
		values  []*int64
		builder array.Builder
		card    map[string]*IntAttrCard
	}

	DoubleAttrCard struct {
		nullCount int
		card      map[float64]bool
	}

	DoubleAttrColumn struct {
		colName string
		values  []*float64
		builder *array.Float64Builder
		card    map[string]*DoubleAttrCard
	}

	StringAttrCard struct {
		nullCount int
		card      map[string]bool
	}

	StringAttrColumn struct {
		colName string
		values  []*string
		builder array.Builder
		card    map[string]*StringAttrCard
	}

	BinaryAttrCard struct {
		nullCount int
		card      *hyperloglog.Sketch
	}

	BinaryAttrColumn struct {
		colName string
		values  [][]byte
		builder array.Builder
		card    map[string]*BinaryAttrCard
	}

	CborAttrColumn struct {
		colName string
		values  [][]byte
		builder array.Builder
		card    map[string]*BinaryAttrCard
	}

	Row struct {
		idx int
	}
)

func NewDynAttrsBuilder(payloadType *arrow2.PayloadType, mem memory.Allocator) *DynAttrsBuilder {
	return &DynAttrsBuilder{
		mem:          mem,
		newColumn:    true,
		payloadType:  payloadType,
		parentGroups: make(map[string][]int),
		parentIDColumn: &ParentIDColumn{
			colName: constants.ParentID,
			values:  make([]uint32, 0),
		},
		colIdx: make(map[string]int),
	}
}

func (b *DynAttrsBuilder) Append(parentName string, parentID uint32, attrs pcommon.Map) error {
	if attrs.Len() == 0 {
		return nil
	}

	currRow := len(b.parentIDColumn.values)

	// Append all the attributes to their respective columns
	addedCount := 0
	attrs.Range(func(k string, v pcommon.Value) bool {
		if v.Type() == pcommon.ValueTypeEmpty {
			return true
		}
		name := colName(k, v)
		colIdx, ok := b.colIdx[name]
		if !ok {
			colIdx = len(b.columns)
			b.colIdx[name] = colIdx
			b.columns = append(b.columns, createColumn(name, v, currRow))
			b.newColumn = true
		}
		col := b.columns[colIdx]
		col.Append(parentName, v)
		addedCount++
		return true
	})
	if addedCount == 0 {
		return nil
	}

	b.parentIDColumn.values = append(b.parentIDColumn.values, parentID)

	// Append nils to columns that don't have a value for this row
	for _, col := range b.columns {
		if col.Len() < len(b.parentIDColumn.values) {
			col.AppendNull(parentName)
			addedCount++
		}
	}

	parentGroup, found := b.parentGroups[parentName]
	if !found {
		parentGroup = make([]int, 0)
		b.parentGroups[parentName] = parentGroup
	}
	b.parentGroups[parentName] = append(b.parentGroups[parentName], currRow)

	return nil
}

func (b *DynAttrsBuilder) SchemaUpdateCount() int {
	return b.schemaUpdateCount
}

func (b *DynAttrsBuilder) IsEmpty() bool {
	return len(b.parentIDColumn.values) == 0
}

func (b *DynAttrsBuilder) Build() (arrow.Record, error) {
	if b.newColumn {
		b.sortColumns()
		b.createBuilder()
		b.updateSchemaID()
		b.newColumn = false
		b.schemaUpdateCount++
	}

	_, sortIdx := b.sortData()

	b.parentIDColumn.Build(sortIdx)

	for _, col := range b.columns {
		err := col.Build(sortIdx)
		if err != nil {
			return nil, err
		}
	}

	record := b.builder.NewRecord()
	//if b.counter > 3 {
	//	panic("stop")
	//}
	//arrowutils.PrintRecord(parentGroups, record)
	//b.counter++

	b.Reset()

	return record, nil
}

func (b *DynAttrsBuilder) SchemaID() string {
	return b.schemaID
}

func (b *DynAttrsBuilder) PayloadType() *arrow2.PayloadType {
	return b.payloadType
}

func (b *DynAttrsBuilder) Reset() {
	b.parentGroups = make(map[string][]int)
	b.parentIDColumn.Reset()
	for _, col := range b.columns {
		col.Reset()
	}
}

// Release releases the memory allocated by the builder.
func (b *DynAttrsBuilder) Release() {
	if b.builder != nil {
		b.builder.Release()
		b.builder = nil
	}
}

func (b *DynAttrsBuilder) sortColumns() {
	sort.Slice(b.columns, func(i, j int) bool {
		return b.columns[i].ColName() < b.columns[j].ColName()
	})

	for i, col := range b.columns {
		b.colIdx[col.ColName()] = i
	}
}

func (b *DynAttrsBuilder) sortData() (parentGroups []string, rows []int) {
	type ColCard struct {
		ColIdx int
		Card   int
	}

	parentGroups = make([]string, 0, len(b.parentIDColumn.values))
	rows = make([]int, 0, len(b.parentIDColumn.values))

	sortedParentGroups := make([]string, 0, len(b.parentGroups))
	for parentGroup := range b.parentGroups {
		sortedParentGroups = append(sortedParentGroups, parentGroup)
	}
	sort.Strings(sortedParentGroups)

	for _, parentGroup := range sortedParentGroups {
		rowIndices := b.parentGroups[parentGroup]
		colCards := make([]ColCard, 0, len(b.columns))
		for i, col := range b.columns {
			card := col.Cardinality(parentGroup)
			colType := col.ColType()
			if card > 1 && colType != arrow.FixedWidthTypes.Boolean {
				colCards = append(colCards, ColCard{
					ColIdx: i,
					Card:   card,
				})
			}
		}
		sort.Slice(colCards, func(i, j int) bool {
			return colCards[i].Card < colCards[j].Card
		})

		//println()
		//fmt.Printf("%s %s-> sort order: ", b.payloadType.PayloadType().String(), parentGroup)
		//for _, colCard := range colCards {
		//	colIdx := colCard.ColIdx
		//	col := b.columns[colIdx]
		//	print(col.ColName(), " ", col.Cardinality(parentGroup), ",")
		//}
		//println("\n")
		//println(b.SchemaID())
		//println()

		sort.Slice(rowIndices, func(i, j int) bool {
			for _, colCard := range colCards {
				colIdx := colCard.ColIdx
				cmp := b.Compare(rowIndices[i], rowIndices[j], colIdx)
				if cmp == 0 {
					continue
				}
				return cmp < 0
			}
			return false
		})
		for i := 0; i < len(rowIndices); i++ {
			parentGroups = append(parentGroups, parentGroup)
		}
		rows = append(rows, rowIndices...)
	}

	return
}

func (b *DynAttrsBuilder) Compare(rowI, rowJ, colIdx int) int {
	col := b.columns[colIdx]
	return col.Compare(rowI, rowJ)
}

func (b *DynAttrsBuilder) createBuilder() {
	if b.builder != nil {
		b.builder.Release()
	}

	fields := make([]arrow.Field, len(b.columns)+1)
	fields[0] = arrow.Field{Name: constants.ParentID, Type: arrow.PrimitiveTypes.Uint32}

	for i, col := range b.columns {
		fields[i+1] = arrow.Field{
			Name: col.ColName(),
			Type: col.ColType(),
		}
	}

	b.builder = array.NewRecordBuilder(b.mem, arrow.NewSchema(fields, nil))

	b.parentIDColumn.builder = b.builder.Field(0).(*array.Uint32Builder)
	for i, builder := range b.builder.Fields()[1:] {
		b.columns[i].SetBuilder(builder)
	}
}

func (b *DynAttrsBuilder) updateSchemaID() {
	var buf bytes.Buffer
	buf.WriteString("struct{")
	buf.WriteString(constants.ParentID)
	buf.WriteString(":u32")
	for _, col := range b.columns {
		buf.WriteString(",")
		buf.WriteString(col.ColName())
		buf.WriteString(":")
		buf.WriteString(col.ColType().String())
	}
	buf.WriteString("}")
	b.schemaID = buf.String()
}

func colName(k string, v pcommon.Value) string {
	switch v.Type() {
	case pcommon.ValueTypeBool:
		return k + "_bool"
	case pcommon.ValueTypeInt:
		return k + "_i64"
	case pcommon.ValueTypeDouble:
		return k + "_f64"
	case pcommon.ValueTypeStr:
		return k + "_str"
	case pcommon.ValueTypeBytes:
		return k + "_bytes"
	case pcommon.ValueTypeMap:
		return k + "_cbor"
	case pcommon.ValueTypeSlice:
		return k + "_cbor"
	default:
		panic("unknown value type")
	}
}

func createColumn(k string, v pcommon.Value, initLen int) AttrColumn {
	switch v.Type() {
	case pcommon.ValueTypeBool:
		return &BoolAttrColumn{
			colName: k,
			values:  make([]*bool, initLen),
			card:    make(map[string]*BoolAttrCard),
		}
	case pcommon.ValueTypeInt:
		return &IntAttrColumn{
			colName: k,
			values:  make([]*int64, initLen),
			card:    make(map[string]*IntAttrCard),
		}
	case pcommon.ValueTypeDouble:
		return &DoubleAttrColumn{
			colName: k,
			values:  make([]*float64, initLen),
			card:    make(map[string]*DoubleAttrCard),
		}
	case pcommon.ValueTypeStr:
		return &StringAttrColumn{
			colName: k,
			values:  make([]*string, initLen),
			card:    make(map[string]*StringAttrCard),
		}
	case pcommon.ValueTypeBytes:
		return &BinaryAttrColumn{
			colName: k,
			values:  make([][]byte, initLen),
			card:    make(map[string]*BinaryAttrCard),
		}
	case pcommon.ValueTypeMap:
		return &CborAttrColumn{
			colName: k,
			values:  make([][]byte, initLen),
			card:    make(map[string]*BinaryAttrCard),
		}
	case pcommon.ValueTypeSlice:
		return &CborAttrColumn{
			colName: k,
			values:  make([][]byte, initLen),
			card:    make(map[string]*BinaryAttrCard),
		}
	default:
		panic("unknown value type")
	}
}

func (c *ParentIDColumn) Build(rowIndices []int) {
	for _, row := range rowIndices {
		parentID := c.values[row]
		c.builder.Append(parentID)
	}
}

func (c *ParentIDColumn) Reset() {
	c.values = c.values[:0]
}

func (c *BoolAttrColumn) ColName() string {
	return c.colName
}

func (c *BoolAttrColumn) ColType() arrow.DataType {
	return arrow.FixedWidthTypes.Boolean
}

func (c *BoolAttrColumn) Append(parentGroup string, v pcommon.Value) {
	val := v.Bool()
	c.values = append(c.values, &val)
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &BoolAttrCard{}
		c.card[parentGroup] = parentGroupCard
	}
	if val {
		parentGroupCard.trueCount++
	} else {
		parentGroupCard.falseCount++
	}
}

func (c *BoolAttrColumn) Cardinality(parentGroup string) int {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		return 0
	}
	card := 0
	if parentGroupCard.trueCount > 0 {
		card++
	}
	if parentGroupCard.falseCount > 0 {
		card++
	}
	if parentGroupCard.nullCount > 0 {
		card++
	}
	return card
}

func (c *BoolAttrColumn) Len() int {
	return len(c.values)
}

func (c *BoolAttrColumn) AppendNull(parentGroup string) {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &BoolAttrCard{}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.nullCount++
	c.values = append(c.values, nil)
}

func (c *BoolAttrColumn) SetBuilder(builder array.Builder) {
	c.builder = builder.(*array.BooleanBuilder)
}

func (c *BoolAttrColumn) Build(rowIndices []int) error {
	for _, row := range rowIndices {
		if c.values[row] == nil {
			c.builder.AppendNull()
		} else {
			c.builder.Append(*c.values[row])
		}
	}
	return nil
}

func (c *BoolAttrColumn) Reset() {
	c.values = c.values[:0]
	c.card = make(map[string]*BoolAttrCard)
}

func (c *BoolAttrColumn) Compare(i, j int) int {
	if c.values[i] == nil {
		if c.values[j] == nil {
			return 0
		}
		return -1
	}
	if c.values[j] == nil {
		return 1
	}
	if *c.values[i] == *c.values[j] {
		return 0
	}
	if *c.values[i] {
		return 1
	}
	return -1
}

func (c *IntAttrColumn) ColName() string {
	return c.colName
}

func (c *IntAttrColumn) ColType() arrow.DataType {
	dt := arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.PrimitiveTypes.Int64,
	}
	return &dt
}

func (c *IntAttrColumn) Append(parentGroup string, v pcommon.Value) {
	val := v.Int()
	c.values = append(c.values, &val)
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &IntAttrCard{
			card: make(map[int64]bool),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.card[val] = true
}

func (c *IntAttrColumn) Cardinality(parentGroup string) int {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		return 0
	}
	if parentGroupCard.nullCount > 0 {
		return len(parentGroupCard.card) + 1
	} else {
		return len(parentGroupCard.card)
	}
}

func (c *IntAttrColumn) Len() int {
	return len(c.values)
}

func (c *IntAttrColumn) AppendNull(parentGroup string) {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &IntAttrCard{
			card: make(map[int64]bool),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.nullCount++
	c.values = append(c.values, nil)
}

func (c *IntAttrColumn) SetBuilder(builder array.Builder) {
	c.builder = builder
}

func (c *IntAttrColumn) Build(rowIndices []int) error {
	switch b := c.builder.(type) {
	case *array.Int64Builder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				b.Append(*c.values[row])
			}
		}
	case *array.Int64DictionaryBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				err := b.Append(*c.values[row])
				if err != nil {
					return err
				}
			}
		}
	default:
		panic("invalid int64 builder type")
	}

	return nil
}

func (c *IntAttrColumn) Reset() {
	c.values = c.values[:0]
	c.card = make(map[string]*IntAttrCard)
}

func (c *IntAttrColumn) Compare(i, j int) int {
	if c.values[i] == nil {
		if c.values[j] == nil {
			return 0
		}
		return -1
	}
	if c.values[j] == nil {
		return 1
	}
	return int(*c.values[i] - *c.values[j])
}

func (c *DoubleAttrColumn) ColName() string {
	return c.colName
}

func (c *DoubleAttrColumn) ColType() arrow.DataType {
	return arrow.PrimitiveTypes.Float64
}

func (c *DoubleAttrColumn) Append(parentGroup string, v pcommon.Value) {
	val := v.Double()
	c.values = append(c.values, &val)
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &DoubleAttrCard{
			card: make(map[float64]bool),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.card[val] = true
}

func (c *DoubleAttrColumn) Cardinality(parentGroup string) int {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		return 0
	}
	if parentGroupCard.nullCount > 0 {
		return len(parentGroupCard.card) + 1
	} else {
		return len(parentGroupCard.card)
	}
}

func (c *DoubleAttrColumn) Len() int {
	return len(c.values)
}

func (c *DoubleAttrColumn) AppendNull(parentGroup string) {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &DoubleAttrCard{
			card: make(map[float64]bool),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.nullCount++
	c.values = append(c.values, nil)
}

func (c *DoubleAttrColumn) SetBuilder(builder array.Builder) {
	c.builder = builder.(*array.Float64Builder)
}

func (c *DoubleAttrColumn) Build(rowIndices []int) error {
	for _, row := range rowIndices {
		if c.values[row] == nil {
			c.builder.AppendNull()
		} else {
			c.builder.Append(*c.values[row])
		}
	}
	return nil
}

func (c *DoubleAttrColumn) Reset() {
	c.values = c.values[:0]
	c.card = make(map[string]*DoubleAttrCard)
}

func (c *DoubleAttrColumn) Compare(i, j int) int {
	if c.values[i] == nil {
		if c.values[j] == nil {
			return 0
		}
		return -1
	}
	if c.values[j] == nil {
		return 1
	}
	if *c.values[i] == *c.values[j] {
		return 0
	}
	if *c.values[i] > *c.values[j] {
		return 1
	}
	return -1
}

func (c *StringAttrColumn) ColName() string {
	return c.colName
}

func (c *StringAttrColumn) ColType() arrow.DataType {
	dt := arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.String,
	}
	return &dt
}

func (c *StringAttrColumn) Append(parentGroup string, v pcommon.Value) {
	val := v.Str()
	c.values = append(c.values, &val)
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &StringAttrCard{
			card: make(map[string]bool),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.card[val] = true
}

func (c *StringAttrColumn) Cardinality(parentGroup string) int {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		return 0
	}
	if parentGroupCard.nullCount > 0 {
		return len(parentGroupCard.card) + 1
	} else {
		return len(parentGroupCard.card)
	}
}

func (c *StringAttrColumn) Len() int {
	return len(c.values)
}

func (c *StringAttrColumn) AppendNull(parentGroup string) {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &StringAttrCard{
			card: make(map[string]bool),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.nullCount++
	c.values = append(c.values, nil)
}

func (c *StringAttrColumn) SetBuilder(builder array.Builder) {
	c.builder = builder
}

func (c *StringAttrColumn) Build(rowIndices []int) error {
	switch b := c.builder.(type) {
	case *array.StringBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				b.Append(*c.values[row])
			}
		}
	case *array.BinaryDictionaryBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				err := b.AppendString(*c.values[row])
				if err != nil {
					return err
				}
			}
		}
	default:
		panic("invalid string builder type")
	}
	return nil
}

func (c *StringAttrColumn) Reset() {
	c.values = c.values[:0]
	c.card = make(map[string]*StringAttrCard)
}

func (c *StringAttrColumn) Compare(i, j int) int {
	if c.values[i] == nil {
		if c.values[j] == nil {
			return 0
		}
		return -1
	}
	if c.values[j] == nil {
		return 1
	}
	return strings.Compare(*c.values[i], *c.values[j])
}

func (c *BinaryAttrColumn) ColName() string {
	return c.colName
}

func (c *BinaryAttrColumn) ColType() arrow.DataType {
	dt := arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.Binary,
	}
	return &dt
}

func (c *BinaryAttrColumn) Append(parentGroup string, v pcommon.Value) {
	val := v.Bytes().AsRaw()
	c.values = append(c.values, val)
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &BinaryAttrCard{
			card: hyperloglog.New16(),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.card.Insert(val)
}

func (c *BinaryAttrColumn) Cardinality(parentGroup string) int {
	estimator, found := c.card[parentGroup]
	if !found {
		return 0
	}
	card := int(estimator.card.Estimate())
	if estimator.nullCount == 0 {
		return card
	} else {
		return card + 1
	}
}

func (c *BinaryAttrColumn) Len() int {
	return len(c.values)
}

func (c *BinaryAttrColumn) AppendNull(parentGroup string) {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &BinaryAttrCard{
			card: hyperloglog.New16(),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.nullCount++
	c.values = append(c.values, nil)
}

func (c *BinaryAttrColumn) SetBuilder(builder array.Builder) {
	c.builder = builder
}

func (c *BinaryAttrColumn) Build(rowIndices []int) error {
	switch b := c.builder.(type) {
	case *array.BinaryBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				b.Append(c.values[row])
			}
		}
	case *array.BinaryDictionaryBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				err := b.Append(c.values[row])
				if err != nil {
					return err
				}
			}
		}
	default:
		panic("invalid binary builder type")
	}
	return nil
}

func (c *BinaryAttrColumn) Reset() {
	c.values = c.values[:0]
	c.card = make(map[string]*BinaryAttrCard)
}

func (c *BinaryAttrColumn) Compare(i, j int) int {
	if c.values[i] == nil {
		if c.values[j] == nil {
			return 0
		}
		return -1
	}
	if c.values[j] == nil {
		return 1
	}
	return bytes.Compare(c.values[i], c.values[j])
}

func (c *CborAttrColumn) ColName() string {
	return c.colName
}

func (c *CborAttrColumn) ColType() arrow.DataType {
	dt := arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.Binary,
	}
	return &dt
}

func (c *CborAttrColumn) Append(parentGroup string, v pcommon.Value) {
	panic("implement me")
}

func (c *CborAttrColumn) Cardinality(parentGroup string) int {
	estimator, found := c.card[parentGroup]
	if !found {
		return 0
	}
	if estimator.nullCount == 0 {
		return int(estimator.card.Estimate())
	}
	return int(estimator.card.Estimate()) + 1
}

func (c *CborAttrColumn) Len() int {
	return len(c.values)
}

func (c *CborAttrColumn) AppendNull(parentGroup string) {
	parentGroupCard, found := c.card[parentGroup]
	if !found {
		parentGroupCard = &BinaryAttrCard{
			card: hyperloglog.New16(),
		}
		c.card[parentGroup] = parentGroupCard
	}
	parentGroupCard.nullCount++
	c.values = append(c.values, nil)
}

func (c *CborAttrColumn) SetBuilder(builder array.Builder) {
	c.builder = builder.(*array.BinaryBuilder)
}

func (c *CborAttrColumn) Build(rowIndices []int) error {
	switch b := c.builder.(type) {
	case *array.BinaryBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				b.Append(c.values[row])
			}
		}
	case *array.BinaryDictionaryBuilder:
		for _, row := range rowIndices {
			if c.values[row] == nil {
				b.AppendNull()
			} else {
				err := b.Append(c.values[row])
				if err != nil {
					return err
				}
			}
		}
	default:
		panic("invalid cbor builder type")
	}
	return nil
}

func (c *CborAttrColumn) Reset() {
	c.values = c.values[:0]
	c.card = make(map[string]*BinaryAttrCard)
}

func (c *CborAttrColumn) Compare(i, j int) int {
	if c.values[i] == nil {
		if c.values[j] == nil {
			return 0
		}
		return -1
	}
	if c.values[j] == nil {
		return 1
	}
	return bytes.Compare(c.values[i], c.values[j])
}
