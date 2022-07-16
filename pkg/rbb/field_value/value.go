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

package field_value

import (
	"bytes"
	"github.com/apache/arrow/go/v9/arrow"
	"sort"
)

type Value interface {
	Normalize()
	DataType() arrow.DataType
	ValueByPath(path []int) Value
	Compare(other Value) int
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
	sort.Slice(v.Fields, func(i, j int) bool {
		return v.Fields[i].Name < v.Fields[j].Name
	})
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

type List struct {
	Values []Value
}

func (v *List) DataType() arrow.DataType {
	return arrow.ListOf(ListDataType(v.Values))
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

// ToDo what about list mixing struct, uint, string, ... items?
