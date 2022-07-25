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

	AsBool() (*bool, error)

	AsU8() (*uint8, error)
	AsU16() (*uint16, error)
	AsU32() (*uint32, error)
	AsU64() (*uint64, error)

	AsI8() (*int8, error)
	AsI16() (*int16, error)
	AsI32() (*int32, error)
	AsI64() (*int64, error)

	AsF32() (*float32, error)
	AsF64() (*float64, error)
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
func (v *Bool) AsBool() (*bool, error) {
	return &v.Value, nil
}

func (v *Bool) AsU8() (*uint8, error) {
	value := uint8(0)
	if v.Value {
		value = uint8(1)
	}
	return &value, nil
}
func (v *Bool) AsU16() (*uint16, error) {
	value := uint16(0)
	if v.Value {
		value = uint16(1)
	}
	return &value, nil
}
func (v *Bool) AsU32() (*uint32, error) {
	value := uint32(0)
	if v.Value {
		value = uint32(1)
	}
	return &value, nil
}
func (v *Bool) AsU64() (*uint64, error) {
	v64 := uint64(0)
	if v.Value {
		v64 = uint64(1)
	}
	return &v64, nil
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
func (v *Bool) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert bool to float32")
}
func (v *Bool) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert bool to float64")
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
func (v *I8) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert signed integer to bool")
}
func (v *I8) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I8) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I8) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I8) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
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
func (v *I8) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I8) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
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
func (v *I16) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert signed integer to bool")
}
func (v *I16) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I16) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I16) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I16) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
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
func (v *I16) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I16) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
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
func (v *I32) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert signed integer to bool")
}
func (v *I32) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I32) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I32) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I32) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
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
func (v *I32) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I32) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
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
func (v *I64) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert signed integer to bool")
}
func (v *I64) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I64) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I64) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
}
func (v *I64) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to unsigned integer")
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
func (v *I64) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I64) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
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
func (v *U8) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to bool")
}
func (v *U8) AsU8() (*uint8, error) {
	return &v.Value, nil
}
func (v *U8) AsU16() (*uint16, error) {
	value := uint16(v.Value)
	return &value, nil
}
func (v *U8) AsU32() (*uint32, error) {
	value := uint32(v.Value)
	return &value, nil
}
func (v *U8) AsU64() (*uint64, error) {
	value := uint64(v.Value)
	return &value, nil
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
	value := int64(v.Value)
	return &value, nil
}
func (v *U8) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U8) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
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
func (v *U16) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to bool")
}
func (v *U16) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert uint16 to uint8")
}
func (v *U16) AsU16() (*uint16, error) {
	return &v.Value, nil
}
func (v *U16) AsU32() (*uint32, error) {
	value := uint32(v.Value)
	return &value, nil
}
func (v *U16) AsU64() (*uint64, error) {
	value := uint64(v.Value)
	return &value, nil
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
func (v *U16) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U16) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
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
func (v *U32) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to bool")
}
func (v *U32) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert uint32 to uint8")
}
func (v *U32) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert uint32 to uint16")
}
func (v *U32) AsU32() (*uint32, error) {
	value := uint32(v.Value)
	return &value, nil
}
func (v *U32) AsU64() (*uint64, error) {
	value := uint64(v.Value)
	return &value, nil
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
func (v *U32) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U32) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
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
func (v *U64) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to bool")
}
func (v *U64) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert uint64 to uint8")
}
func (v *U64) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert uint64 to uint16")
}
func (v *U64) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert uint64 to uint16")
}
func (v *U64) AsU64() (*uint64, error) {
	value := uint64(v.Value)
	return &value, nil
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
func (v *U64) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U64) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
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
func (v *F32) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert float32 to bool")
}
func (v *F32) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert f32 to uint8")
}
func (v *F32) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert f32 to uint16")
}
func (v *F32) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert f32 to uint16")
}
func (v *F32) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert f32 to uint16")
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
func (v *F32) AsF32() (*float32, error) {
	return &v.Value, nil
}
func (v *F32) AsF64() (*float64, error) {
	value := float64(v.Value)
	return &value, nil
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
func (v *F64) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert float64 to bool")
}
func (v *F64) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint8")
}
func (v *F64) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint16")
}
func (v *F64) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint16")
}
func (v *F64) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint16")
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
func (v *F64) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert f64 to float32")
}
func (v *F64) AsF64() (*float64, error) {
	return &v.Value, nil
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
func (v *String) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert string to bool")
}
func (v *String) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert string to uint8")
}
func (v *String) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert string to uint16")
}
func (v *String) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert string to uint16")
}
func (v *String) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert string to uint16")
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
func (v *String) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert string to float32")
}
func (v *String) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert string to float64")
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
func (v *Binary) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert binary to bool")
}
func (v *Binary) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert binary to uint8")
}
func (v *Binary) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert binary to uint16")
}
func (v *Binary) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert binary to uint16")
}
func (v *Binary) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert binary to uint16")
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
func (v *Binary) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert binary to float32")
}
func (v *Binary) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert binary to float64")
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
func (v *Struct) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert struct to bool")
}
func (v *Struct) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert struct to uint8")
}
func (v *Struct) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert struct to uint16")
}
func (v *Struct) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert struct to uint16")
}
func (v *Struct) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert struct to uint16")
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
func (v *Struct) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert struct to float32")
}
func (v *Struct) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert struct to float64")
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
	dataTypeSet := map[string]arrow.DataType{}

	// Deduplicate data types
	for _, value := range values {
		dataType := value.DataType()
		if dataType.ID() != arrow.NULL {
			dataTypeSet[DataTypeSignature(dataType)] = dataType
		}
	}

	if len(dataTypeSet) > 0 {
		dataTypes := make([]arrow.DataType, 0, len(dataTypeSet))
		for _, dataType := range dataTypeSet {
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
func (v *List) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert list to bool")
}
func (v *List) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert list to uint8")
}
func (v *List) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert list to uint16")
}
func (v *List) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert list to uint16")
}
func (v *List) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert list to uint16")
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
func (v *List) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert list to float32")
}
func (v *List) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert list to float64")
}

// ToDo what about list mixing struct, uint, string, ... items?
