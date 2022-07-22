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

package rfield

import (
	"bytes"
	"fmt"
	"github.com/apache/arrow/go/v9/arrow"
	"sort"
)

type Value interface {
	Normalize()
	DataType() arrow.DataType
	ValueByPath(path []int) Value
	Compare(other Value) int

	AsI8() (*int8, error)
	AsI16() (*int16, error)
	AsI32() (*int32, error)
	AsI64() (*int64, error)
}

type CommonValue struct{}

func (cv *CommonValue) Normalize() {}

type Bool struct {
	CommonValue
	Value bool
}

func (v *Bool) DataType() arrow.DataType { return arrow.FixedWidthTypes.Boolean }
func (v *Bool) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *Bool) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*Bool).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value {
		return 1
	} else {
		return -1
	}
}
func (v *Bool) AsI8() (*int8, error) {
	value := int8(0)
	if v.Value {
		value = int8(1)
	}
	return &value, nil
}
func (v *Bool) AsI16() (*int16, error) {
	value := int16(0)
	if v.Value {
		value = int16(1)
	}
	return &value, nil
}
func (v *Bool) AsI32() (*int32, error) {
	value := int32(0)
	if v.Value {
		value = int32(1)
	}
	return &value, nil
}
func (v *Bool) AsI64() (*int64, error) {
	v64 := int64(0)
	if v.Value {
		v64 = int64(1)
	}
	return &v64, nil
}

type I8 struct {
	CommonValue
	Value int8
}

func (v *I8) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int8 }
func (v *I8) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *I8) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I8).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I8) AsI8() (*int8, error) {
	return &v.Value, nil
}
func (v *I8) AsI16() (*int16, error) {
	value := int16(v.Value)
	return &value, nil
}
func (v *I8) AsI32() (*int32, error) {
	value := int32(v.Value)
	return &value, nil
}
func (v *I8) AsI64() (*int64, error) {
	v64 := int64(v.Value)
	return &v64, nil
}

type I16 struct {
	CommonValue
	Value int16
}

func (v *I16) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int16 }
func (v *I16) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *I16) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I16).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I16) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert int16 to int8")
}
func (v *I16) AsI16() (*int16, error) {
	return &v.Value, nil
}
func (v *I16) AsI32() (*int32, error) {
	value := int32(v.Value)
	return &value, nil
}
func (v *I16) AsI64() (*int64, error) {
	v64 := int64(v.Value)
	return &v64, nil
}

type I32 struct {
	CommonValue
	Value int32
}

func (v *I32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int32 }
func (v *I32) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *I32) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I32).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I32) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert int32 to int8")
}
func (v *I32) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert int32 to int16")
}
func (v *I32) AsI32() (*int32, error) {
	return &v.Value, nil
}
func (v *I32) AsI64() (*int64, error) {
	v64 := int64(v.Value)
	return &v64, nil
}

type I64 struct {
	CommonValue
	Value int64
}

func (v *I64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int64 }
func (v *I64) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *I64) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I64).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I64) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert int64 to int8")
}
func (v *I64) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert int64 to int16")
}
func (v *I64) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert int64 to int32")
}
func (v *I64) AsI64() (*int64, error) {
	return &v.Value, nil
}

type U8 struct {
	CommonValue
	Value uint8
}

func (v *U8) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint8 }
func (v *U8) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *U8) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U8).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U8) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert uint8 to int8")
}
func (v *U8) AsI16() (*int16, error) {
	value := int16(v.Value)
	return &value, nil
}
func (v *U8) AsI32() (*int32, error) {
	value := int32(v.Value)
	return &value, nil
}
func (v *U8) AsI64() (*int64, error) {
	v64 := int64(v.Value)
	return &v64, nil
}

type U16 struct {
	CommonValue
	Value uint16
}

func (v *U16) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint16 }
func (v *U16) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *U16) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U16).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U16) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert uint16 to int8")
}
func (v *U16) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert uint16 to int16")
}
func (v *U16) AsI32() (*int32, error) {
	value := int32(v.Value)
	return &value, nil
}
func (v *U16) AsI64() (*int64, error) {
	v64 := int64(v.Value)
	return &v64, nil
}

type U32 struct {
	CommonValue
	Value uint32
}

func (v *U32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint32 }
func (v *U32) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *U32) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U32).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U32) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert uint32 to int8")
}
func (v *U32) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert uint32 to int16")
}
func (v *U32) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert uint32 to int32")
}
func (v *U32) AsI64() (*int64, error) {
	v64 := int64(v.Value)
	return &v64, nil
}

type U64 struct {
	CommonValue
	Value uint64
}

func (v *U64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint64 }
func (v *U64) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *U64) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U64).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U64) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert uint64 column to int8")
}
func (v *U64) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert uint64 column to int16")
}
func (v *U64) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert uint64 column to int32")
}
func (c *U64) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert uint64 column to int64")
}

type F32 struct {
	CommonValue
	Value float32
}

func (v *F32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Float32 }
func (v *F32) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *F32) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*F32).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *F32) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert f32 column to int8")
}
func (v *F32) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert f32 column to int16")
}
func (v *F32) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert f32 column to int32")
}
func (c *F32) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert f32 column to int64")
}

type F64 struct {
	CommonValue
	Value float64
}

func (v *F64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Float64 }
func (v *F64) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *F64) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*F64).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *F64) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int8")
}
func (v *F64) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int16")
}
func (v *F64) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int32")
}
func (c *F64) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int64")
}

type String struct {
	CommonValue
	Value string
}

func (v *String) DataType() arrow.DataType { return arrow.BinaryTypes.String }
func (v *String) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *String) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*String).Value
	if v.Value == otherValue {
		return 0
	} else if v.Value > otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *String) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert string column to int8")
}
func (v *String) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert string column to int16")
}
func (v *String) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert string column to int32")
}
func (c *String) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert string column to int64")
}

type Binary struct {
	CommonValue
	Value []byte
}

func (v *Binary) DataType() arrow.DataType { return arrow.BinaryTypes.Binary }
func (v *Binary) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return nil
}
func (v *Binary) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*Binary).Value
	return bytes.Compare(v.Value, otherValue)
}
func (v *Binary) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert binary column to int8")
}
func (v *Binary) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert binary column to int16")
}
func (v *Binary) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert binary column to int32")
}
func (c *Binary) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert binary column to int64")
}

type Struct struct {
	Fields []*Field
}

func (v *Struct) DataType() arrow.DataType {
	fields := make([]arrow.Field, 0, len(v.Fields))
	for _, field := range v.Fields {
		arrowField := arrow.Field{Name: field.Name, Type: field.Value.DataType(), Nullable: true, Metadata: arrow.Metadata{}}
		fields = append(fields, arrowField)
	}
	return arrow.StructOf(fields...)
}
func (v *Struct) Normalize() {
	// Sort all the fields by name
	sort.Sort(Fields(v.Fields))
	// Normalize recursively all the fields
	for _, field := range v.Fields {
		field.Normalize()
	}
}
func (v *Struct) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return v.Fields[path[0]].ValueByPath(path[1:])
}
func (v *Struct) Compare(_ Value) int {
	panic("struct comparison not implemented")
}
func (v *Struct) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert struct column to int8")
}
func (v *Struct) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert struct column to int16")
}
func (v *Struct) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert struct column to int32")
}
func (c *Struct) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert struct column to int64")
}

type List struct {
	etype  arrow.DataType
	Values []Value
}

func (v *List) DataType() arrow.DataType {
	return arrow.ListOf(v.EType())
}

func (v *List) EType() arrow.DataType {
	if v.etype == nil {
		v.etype = listDataType(v.Values)
	}
	return v.etype
}

func listDataType(values []Value) arrow.DataType {
	dataTypeSet := map[arrow.DataType]bool{}

	// Deduplicate data types
	for _, value := range values {
		dataType := value.DataType()
		if dataType.ID() != arrow.NULL {
			dataTypeSet[dataType] = true
		}
	}

	if len(dataTypeSet) > 0 {
		dataTypes := make([]arrow.DataType, 0, len(dataTypeSet))
		for dataType := range dataTypeSet {
			dataTypes = append(dataTypes, dataType)
		}
		return CoerceDataType(&dataTypes)
	} else {
		return arrow.Null
	}
}

func (v *List) Normalize() {
	// Normalize recursively all the value
	for _, value := range v.Values {
		value.Normalize()
	}
}
func (v *List) ValueByPath(path []int) Value {
	if path == nil || len(path) == 0 {
		return v
	}
	return v.Values[path[0]].ValueByPath(path[1:])
}
func (v *List) Compare(_ Value) int {
	panic("struct comparison not implemented")
}
func (v *List) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert list column to int8")
}
func (v *List) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert list column to int16")
}
func (v *List) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert list column to int32")
}
func (c *List) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert list column to int64")
}

// ToDo what about list mixing struct, uint, string, ... items?
