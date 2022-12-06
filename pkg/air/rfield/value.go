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
	"encoding/binary"
	"fmt"
	"sort"
	"strconv"
	"strings"

	"github.com/apache/arrow/go/v10/arrow"

	"github.com/f5/otel-arrow-adapter/pkg/air/common"
)

type Value interface {
	Normalize()
	DataType() arrow.DataType
	ValueByPath(path []int) Value
	StringPath(path []int) string
	Compare(other Value) int
	WriteSignature(sig *strings.Builder)
	WriteData(sig *strings.Builder)
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

	AsString() (*string, error)
	AsBinary() ([]byte, error)
}

type CommonValue struct{}

func (cv *CommonValue) Normalize() {}

type Bool struct {
	CommonValue
	Value *bool
}

func NewBool(value bool) *Bool {
	return &Bool{
		Value: &value,
	}
}
func (v *Bool) DataType() arrow.DataType { return arrow.FixedWidthTypes.Boolean }
func (v *Bool) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *Bool) StringPath(_ []int) string {
	return ""
}
func (v *Bool) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*Bool).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value != nil && *v.Value {
		return 1
	} else {
		return -1
	}
}
func (v *Bool) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.BOOL_SIG)
}
func (v *Bool) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%v", *v.Value))
	}
}
func (v *Bool) AsBool() (*bool, error) {
	return v.Value, nil
}

func (v *Bool) AsU8() (*uint8, error) {
	value := uint8(0)
	if v.Value != nil && *v.Value {
		value = uint8(1)
	}
	return &value, nil
}
func (v *Bool) AsU16() (*uint16, error) {
	value := uint16(0)
	if v.Value != nil && *v.Value {
		value = uint16(1)
	}
	return &value, nil
}
func (v *Bool) AsU32() (*uint32, error) {
	value := uint32(0)
	if v.Value != nil && *v.Value {
		value = uint32(1)
	}
	return &value, nil
}
func (v *Bool) AsU64() (*uint64, error) {
	v64 := uint64(0)
	if v.Value != nil && *v.Value {
		v64 = uint64(1)
	}
	return &v64, nil
}
func (v *Bool) AsI8() (*int8, error) {
	value := int8(0)
	if v.Value != nil && *v.Value {
		value = int8(1)
	}
	return &value, nil
}
func (v *Bool) AsI16() (*int16, error) {
	value := int16(0)
	if v.Value != nil && *v.Value {
		value = int16(1)
	}
	return &value, nil
}
func (v *Bool) AsI32() (*int32, error) {
	value := int32(0)
	if v.Value != nil && *v.Value {
		value = int32(1)
	}
	return &value, nil
}
func (v *Bool) AsI64() (*int64, error) {
	v64 := int64(0)
	if v.Value != nil && *v.Value {
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
func (v *Bool) AsString() (*string, error) {
	if v.Value != nil {
		if *v.Value {
			val := "true"
			return &val, nil
		} else {
			val := "false"
			return &val, nil
		}
	} else {
		return nil, nil
	}
}
func (v *Bool) AsBinary() ([]byte, error) {
	if v.Value != nil {
		if *v.Value {
			return []byte("true"), nil
		} else {
			return []byte("false"), nil
		}
	} else {
		return nil, nil
	}
}

type I8 struct {
	CommonValue
	Value *int8
}

func NewI8(value int8) *I8 {
	return &I8{Value: &value}
}

func (v *I8) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int8 }
func (v *I8) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *I8) StringPath(_ []int) string {
	return ""
}
func (v *I8) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I8).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I8) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.I8_SIG)
}
func (v *I8) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
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
	return v.Value, nil
}
func (v *I8) AsI16() (*int16, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int16(*v.Value)
	return &value, nil
}
func (v *I8) AsI32() (*int32, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int32(*v.Value)
	return &value, nil
}
func (v *I8) AsI64() (*int64, error) {
	if v.Value == nil {
		return nil, nil
	}
	v64 := int64(*v.Value)
	return &v64, nil
}
func (v *I8) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I8) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
}
func (v *I8) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatInt(int64(*v.Value), 10)
	return &val, nil
}
func (v *I8) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type I16 struct {
	CommonValue
	Value *int16
}

func NewI16(value int16) *I16 {
	return &I16{Value: &value}
}
func (v *I16) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int16 }
func (v *I16) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *I16) StringPath(_ []int) string {
	return ""
}
func (v *I16) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I16).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I16) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.I16_SIG)
}
func (v *I16) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
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
	return v.Value, nil
}
func (v *I16) AsI32() (*int32, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int32(*v.Value)
	return &value, nil
}
func (v *I16) AsI64() (*int64, error) {
	if v.Value == nil {
		return nil, nil
	}
	v64 := int64(*v.Value)
	return &v64, nil
}
func (v *I16) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I16) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
}
func (v *I16) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatInt(int64(*v.Value), 10)
	return &val, nil
}
func (v *I16) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type I32 struct {
	CommonValue
	Value *int32
}

func NewI32(value int32) *I32 {
	return &I32{Value: &value}
}
func (v *I32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int32 }
func (v *I32) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *I32) StringPath(_ []int) string {
	return ""
}
func (v *I32) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I32).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I32) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.I32_SIG)
}
func (v *I32) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
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
	return v.Value, nil
}
func (v *I32) AsI64() (*int64, error) {
	if v.Value == nil {
		return nil, nil
	}
	v64 := int64(*v.Value)
	return &v64, nil
}
func (v *I32) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I32) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
}
func (v *I32) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatInt(int64(*v.Value), 10)
	return &val, nil
}
func (v *I32) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type I64 struct {
	CommonValue
	Value *int64
}

func NewI64(value int64) *I64 {
	return &I64{Value: &value}
}
func (v *I64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Int64 }
func (v *I64) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *I64) StringPath(_ []int) string {
	return ""
}
func (v *I64) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*I64).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *I64) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.I64_SIG)
}
func (v *I64) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
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
	return v.Value, nil
}
func (v *I64) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float32")
}
func (v *I64) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert signed integer to float64")
}
func (v *I64) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatInt(*v.Value, 10)
	return &val, nil
}
func (v *I64) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type U8 struct {
	CommonValue
	Value *uint8
}

func NewU8(value uint8) *U8 {
	return &U8{Value: &value}
}
func (v *U8) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint8 }
func (v *U8) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *U8) StringPath(_ []int) string {
	return ""
}
func (v *U8) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U8).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U8) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.U8_SIG)
}
func (v *U8) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
	}
}
func (v *U8) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to bool")
}
func (v *U8) AsU8() (*uint8, error) {
	return v.Value, nil
}
func (v *U8) AsU16() (*uint16, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := uint16(*v.Value)
	return &value, nil
}
func (v *U8) AsU32() (*uint32, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := uint32(*v.Value)
	return &value, nil
}
func (v *U8) AsU64() (*uint64, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := uint64(*v.Value)
	return &value, nil
}
func (v *U8) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert uint8 to int8")
}
func (v *U8) AsI16() (*int16, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int16(*v.Value)
	return &value, nil
}
func (v *U8) AsI32() (*int32, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int32(*v.Value)
	return &value, nil
}
func (v *U8) AsI64() (*int64, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int64(*v.Value)
	return &value, nil
}
func (v *U8) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U8) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
}
func (v *U8) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatUint(uint64(*v.Value), 10)
	return &val, nil
}
func (v *U8) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type U16 struct {
	CommonValue
	Value *uint16
}

func NewU16(value uint16) *U16 {
	return &U16{Value: &value}
}
func (v *U16) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint16 }
func (v *U16) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *U16) StringPath(_ []int) string {
	return ""
}
func (v *U16) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U16).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U16) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.U16_SIG)
}
func (v *U16) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
	}
}
func (v *U16) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to bool")
}
func (v *U16) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert uint16 to uint8")
}
func (v *U16) AsU16() (*uint16, error) {
	return v.Value, nil
}
func (v *U16) AsU32() (*uint32, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := uint32(*v.Value)
	return &value, nil
}
func (v *U16) AsU64() (*uint64, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := uint64(*v.Value)
	return &value, nil
}
func (v *U16) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert uint16 to int8")
}
func (v *U16) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert uint16 to int16")
}
func (v *U16) AsI32() (*int32, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := int32(*v.Value)
	return &value, nil
}
func (v *U16) AsI64() (*int64, error) {
	if v.Value == nil {
		return nil, nil
	}
	v64 := int64(*v.Value)
	return &v64, nil
}
func (v *U16) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U16) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
}
func (v *U16) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatUint(uint64(*v.Value), 10)
	return &val, nil
}
func (v *U16) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type U32 struct {
	CommonValue
	Value *uint32
}

func NewU32(value uint32) *U32 {
	return &U32{Value: &value}
}
func (v *U32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint32 }
func (v *U32) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *U32) StringPath(_ []int) string {
	return ""
}
func (v *U32) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U32).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U32) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.U32_SIG)
}
func (v *U32) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
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
	return v.Value, nil
}
func (v *U32) AsU64() (*uint64, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := uint64(*v.Value)
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
	if v.Value == nil {
		return nil, nil
	}
	v64 := int64(*v.Value)
	return &v64, nil
}
func (v *U32) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U32) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
}
func (v *U32) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatUint(uint64(*v.Value), 10)
	return &val, nil
}
func (v *U32) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type U64 struct {
	CommonValue
	Value *uint64
}

func NewU64(value uint64) *U64 {
	return &U64{Value: &value}
}
func (v *U64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Uint64 }
func (v *U64) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *U64) StringPath(_ []int) string {
	return ""
}
func (v *U64) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*U64).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *U64) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.U64_SIG)
}
func (v *U64) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%d", *v.Value))
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
	return v.Value, nil
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
func (v *U64) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert uint64 column to int64")
}
func (v *U64) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float32")
}
func (v *U64) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert unsigned integer to float64")
}
func (v *U64) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := strconv.FormatUint(*v.Value, 10)
	return &val, nil
}
func (v *U64) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type F32 struct {
	CommonValue
	Value *float32
}

func NewF32(value float32) *F32 {
	return &F32{Value: &value}
}
func (v *F32) DataType() arrow.DataType { return arrow.PrimitiveTypes.Float32 }
func (v *F32) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return v
	}
	return nil
}
func (v *F32) StringPath(_ []int) string {
	return ""
}
func (v *F32) Compare(other Value) int {
	if other == nil || other.DataType() != v.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*F32).Value
	if *v.Value == *otherValue {
		return 0
	} else if v.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *v.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (v *F32) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.F32_SIG)
}
func (v *F32) WriteData(sig *strings.Builder) {
	if v.Value != nil {
		sig.WriteString(fmt.Sprintf("%f", *v.Value))
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
func (v *F32) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert f32 column to int64")
}
func (v *F32) AsF32() (*float32, error) {
	return v.Value, nil
}
func (v *F32) AsF64() (*float64, error) {
	if v.Value == nil {
		return nil, nil
	}
	value := float64(*v.Value)
	return &value, nil
}
func (v *F32) AsString() (*string, error) {
	if v.Value == nil {
		return nil, nil
	}
	val := fmt.Sprintf("%f", *v.Value)
	return &val, nil
}
func (v *F32) AsBinary() ([]byte, error) {
	if v.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *v.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type F64 struct {
	CommonValue
	Value *float64
}

func NewF64(value float64) *F64 {
	return &F64{Value: &value}
}
func (f *F64) DataType() arrow.DataType { return arrow.PrimitiveTypes.Float64 }
func (f *F64) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return f
	}
	return nil
}
func (f *F64) StringPath(_ []int) string {
	return ""
}
func (f *F64) Compare(other Value) int {
	if other == nil || other.DataType() != f.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*F64).Value
	if *f.Value == *otherValue {
		return 0
	} else if f.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *f.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (f *F64) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.F64_SIG)
}
func (f *F64) WriteData(sig *strings.Builder) {
	if f.Value != nil {
		sig.WriteString(fmt.Sprintf("%f", *f.Value))
	}
}
func (f *F64) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert float64 to bool")
}
func (f *F64) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint8")
}
func (f *F64) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint16")
}
func (f *F64) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint16")
}
func (f *F64) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert f64 to uint16")
}
func (f *F64) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int8")
}
func (f *F64) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int16")
}
func (f *F64) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int32")
}
func (f *F64) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert f64 column to int64")
}
func (f *F64) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert f64 to float32")
}
func (f *F64) AsF64() (*float64, error) {
	return f.Value, nil
}
func (f *F64) AsString() (*string, error) {
	if f.Value == nil {
		return nil, nil
	}
	val := fmt.Sprintf("%f", *f.Value)
	return &val, nil
}
func (f *F64) AsBinary() ([]byte, error) {
	if f.Value == nil {
		return nil, nil
	}
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, *f.Value)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

type String struct {
	CommonValue
	Value *string
}

func NewString(value string) *String {
	return &String{
		Value: &value,
	}
}
func (s *String) DataType() arrow.DataType { return arrow.BinaryTypes.String }
func (s *String) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return s
	}
	return nil
}
func (s *String) StringPath(_ []int) string {
	return ""
}
func (s *String) Compare(other Value) int {
	if other == nil || other.DataType() != s.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*String).Value
	if *s.Value == *otherValue {
		return 0
	} else if s.Value == nil {
		return -1
	} else if otherValue == nil {
		return 1
	} else if *s.Value > *otherValue {
		return 1
	} else {
		return -1
	}
}
func (s *String) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.STRING_SIG)
}
func (s *String) WriteData(sig *strings.Builder) {
	if s.Value != nil {
		sig.WriteString(*s.Value)
	}
}
func (s *String) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert string to bool")
}
func (s *String) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert string to uint8")
}
func (s *String) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert string to uint16")
}
func (s *String) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert string to uint16")
}
func (s *String) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert string to uint16")
}
func (s *String) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert string column to int8")
}
func (s *String) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert string column to int16")
}
func (s *String) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert string column to int32")
}
func (s *String) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert string column to int64")
}
func (s *String) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert string to float32")
}
func (s *String) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert string to float64")
}
func (s *String) AsString() (*string, error) {
	return s.Value, nil
}
func (s *String) AsBinary() ([]byte, error) {
	if s.Value == nil {
		return nil, nil
	} else {
		return []byte(*s.Value), nil
	}
}

type Binary struct {
	CommonValue
	Value []byte
}

func NewBinary(value []byte) *Binary {
	return &Binary{
		Value: value,
	}
}
func (b *Binary) DataType() arrow.DataType { return arrow.BinaryTypes.Binary }
func (b *Binary) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return b
	}
	return nil
}
func (b *Binary) StringPath(_ []int) string {
	return ""
}
func (b *Binary) Compare(other Value) int {
	if other == nil || other.DataType() != b.DataType() {
		panic("invalid comparison")
	}
	otherValue := other.(*Binary).Value
	return bytes.Compare(b.Value, otherValue)
}
func (b *Binary) WriteSignature(sig *strings.Builder) {
	sig.WriteString(common.BINARY_SIG)
}
func (b *Binary) WriteData(sig *strings.Builder) {
	if b.Value != nil {
		sig.WriteString(fmt.Sprintf("%x", b.Value))
	}
}
func (b *Binary) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert binary to bool")
}
func (b *Binary) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert binary to uint8")
}
func (b *Binary) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert binary to uint16")
}
func (b *Binary) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert binary to uint16")
}
func (b *Binary) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert binary to uint16")
}
func (b *Binary) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert binary column to int8")
}
func (b *Binary) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert binary column to int16")
}
func (b *Binary) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert binary column to int32")
}
func (b *Binary) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert binary column to int64")
}
func (b *Binary) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert binary to float32")
}
func (b *Binary) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert binary to float64")
}
func (b *Binary) AsString() (*string, error) {
	val := string(b.Value)
	return &val, nil
}
func (b *Binary) AsBinary() ([]byte, error) {
	return b.Value, nil
}

type Struct struct {
	Fields []*Field
}

func NewStruct(fields []*Field) *Struct {
	return &Struct{
		Fields: fields,
	}
}
func (st *Struct) DataType() arrow.DataType {
	fields := make([]arrow.Field, 0, len(st.Fields))
	for _, field := range st.Fields {
		fieldMetadata := field.Metadata()
		if fieldMetadata == nil {
			arrowField := arrow.Field{Name: field.Name, Type: field.Value.DataType(), Nullable: true, Metadata: arrow.Metadata{}}
			fields = append(fields, arrowField)
		} else {
			arrowField := arrow.Field{Name: field.Name, Type: field.Value.DataType(), Nullable: true, Metadata: arrow.NewMetadata(fieldMetadata.Keys, fieldMetadata.Values)}
			fields = append(fields, arrowField)
		}
	}
	return arrow.StructOf(fields...)
}
func (st *Struct) Normalize() {
	// Sort all the fields by name
	sort.Sort(Fields(st.Fields))
	// Normalize recursively all the fields
	for _, field := range st.Fields {
		field.Normalize()
	}
}
func (st *Struct) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return st
	}
	return st.Fields[path[0]].ValueByPath(path[1:])
}
func (st *Struct) StringPath(path []int) string {
	if len(path) == 0 {
		return ""
	}
	return st.Fields[path[0]].StringPath(path[1:])
}
func (st *Struct) Compare(_ Value) int {
	panic("struct comparison not implemented")
}
func (st *Struct) WriteSignature(sig *strings.Builder) {
	sig.WriteString("{")
	for i, f := range st.Fields {
		if i > 0 {
			sig.WriteByte(',')
		}
		f.WriteSigType(sig)
	}
	sig.WriteString("}")
}
func (st *Struct) WriteData(sig *strings.Builder) {
	sig.WriteString("{")
	for i, f := range st.Fields {
		if i > 0 {
			sig.WriteByte(',')
		}
		f.Value.WriteData(sig)
	}
	sig.WriteString("}")
}
func (st *Struct) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert struct to bool")
}
func (st *Struct) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert struct to uint8")
}
func (st *Struct) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert struct to uint16")
}
func (st *Struct) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert struct to uint16")
}
func (st *Struct) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert struct to uint16")
}
func (st *Struct) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert struct column to int8")
}
func (st *Struct) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert struct column to int16")
}
func (st *Struct) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert struct column to int32")
}
func (st *Struct) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert struct column to int64")
}
func (st *Struct) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert struct to float32")
}
func (st *Struct) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert struct to float64")
}
func (st *Struct) AsString() (*string, error) {
	return nil, fmt.Errorf("cannot convert struct to string")
}
func (st *Struct) AsBinary() ([]byte, error) {
	return nil, fmt.Errorf("cannot convert struct to binary")
}

type List struct {
	etype  arrow.DataType
	Values []Value
}

func UnsafeNewList(etype arrow.DataType, values []Value) *List {
	return &List{
		etype:  etype,
		Values: values,
	}
}
func (l *List) DataType() arrow.DataType {
	return arrow.ListOf(l.EType())
}

func (l *List) EType() arrow.DataType {
	if l.etype == nil {
		l.etype = listDataType(l.Values)
	}
	return l.etype
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
		return CoerceDataType(dataTypes)
	} else {
		return arrow.Null
	}
}

func (l *List) Normalize() {
	// Normalize recursively all the value
	for _, value := range l.Values {
		value.Normalize()
	}
}
func (l *List) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return l
	}
	return l.Values[path[0]].ValueByPath(path[1:])
}
func (l *List) StringPath(path []int) string {
	if len(path) == 0 {
		return ""
	}
	subPath := l.Values[path[0]].StringPath(path[1:])
	if subPath != "" {
		return "[]" + subPath
	} else {
		return "[]"
	}
}
func (l *List) Compare(_ Value) int {
	panic("struct comparison not implemented")
}
func (l *List) WriteSignature(sig *strings.Builder) {
	sig.WriteByte('[')
	eType := l.EType()
	WriteDataTypeSignature(eType, sig)
	sig.WriteByte(']')
}
func (l *List) WriteData(sig *strings.Builder) {
	sig.WriteString("[")
	for i := 0; i < len(l.Values); i++ {
		if i > 0 {
			sig.WriteString(", ")
		}
		l.Values[i].WriteData(sig)
	}
	sig.WriteString("]")
}
func (l *List) AsBool() (*bool, error) {
	return nil, fmt.Errorf("cannot convert list to bool")
}
func (l *List) AsU8() (*uint8, error) {
	return nil, fmt.Errorf("cannot convert list to uint8")
}
func (l *List) AsU16() (*uint16, error) {
	return nil, fmt.Errorf("cannot convert list to uint16")
}
func (l *List) AsU32() (*uint32, error) {
	return nil, fmt.Errorf("cannot convert list to uint16")
}
func (l *List) AsU64() (*uint64, error) {
	return nil, fmt.Errorf("cannot convert list to uint16")
}
func (l *List) AsI8() (*int8, error) {
	return nil, fmt.Errorf("cannot convert list column to int8")
}
func (l *List) AsI16() (*int16, error) {
	return nil, fmt.Errorf("cannot convert list column to int16")
}
func (l *List) AsI32() (*int32, error) {
	return nil, fmt.Errorf("cannot convert list column to int32")
}
func (l *List) AsI64() (*int64, error) {
	return nil, fmt.Errorf("cannot convert list column to int64")
}
func (l *List) AsF32() (*float32, error) {
	return nil, fmt.Errorf("cannot convert list to float32")
}
func (l *List) AsF64() (*float64, error) {
	return nil, fmt.Errorf("cannot convert list to float64")
}
func (l *List) AsString() (*string, error) {
	return nil, fmt.Errorf("cannot convert list to string")
}
func (l *List) AsBinary() ([]byte, error) {
	return nil, fmt.Errorf("cannot convert list to binary")
}

// ToDo what about list mixing struct, uint, string, ... items?
