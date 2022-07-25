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

package air

import (
	"sort"
	"strings"

	"otel-arrow-adapter/pkg/air/rfield"
)

type Records struct {
	records []*Record
	orderBy *OrderBy
}

// Sort interface
func (r *Records) Less(i, j int) bool {
	r1 := r.records[i]
	r2 := r.records[j]
	if r1.Compare(r2, r.orderBy.FieldPaths) < 0 {
		return true
	} else {
		return false
	}
}
func (r *Records) Len() int      { return len(r.records) }
func (r *Records) Swap(i, j int) { r.records[i], r.records[j] = r.records[j], r.records[i] }

// Record is a collection of fields (scalar our composite fields).
type Record struct {
	fields []*rfield.Field
}

func NewRecord() *Record {
	return &Record{}
}

func NewRecordFromFields(fields []*rfield.Field) *Record {
	return &Record{fields: fields}
}

// Normalize normalizes the field names and values.
func (r Record) Normalize() {
	sort.Sort(rfield.Fields(r.fields))
	for _, f := range r.fields {
		f.Normalize()
	}
}

// SchemaId returns the canonical schema id of the record.
func (r Record) SchemaId() string {
	var sig strings.Builder
	for i, f := range r.fields {
		if i > 0 {
			sig.WriteByte(',')
		}
		f.WriteSignature(&sig)
	}
	return sig.String()
}

func (r *Record) FieldCount() int {
	return len(r.fields)
}

func (r *Record) AddField(f *rfield.Field) {
	r.fields = append(r.fields, f)
}

func (r *Record) GenericField(name string, value rfield.Value) {
	r.fields = append(r.fields, rfield.NewField(name, value))
}

func (r *Record) BoolField(name string, value bool) {
	r.fields = append(r.fields, rfield.NewBoolField(name, value))
}

func (r *Record) I8Field(name string, value int8) {
	r.fields = append(r.fields, rfield.NewI8Field(name, value))
}

func (r *Record) I16Field(name string, value int16) {
	r.fields = append(r.fields, rfield.NewI16Field(name, value))
}

func (r *Record) I32Field(name string, value int32) {
	r.fields = append(r.fields, rfield.NewI32Field(name, value))
}

func (r *Record) I64Field(name string, value int64) {
	r.fields = append(r.fields, rfield.NewI64Field(name, value))
}

func (r *Record) U8Field(name string, value uint8) {
	r.fields = append(r.fields, rfield.NewU8Field(name, value))
}

func (r *Record) U16Field(name string, value uint16) {
	r.fields = append(r.fields, rfield.NewU16Field(name, value))
}

func (r *Record) U32Field(name string, value uint32) {
	r.fields = append(r.fields, rfield.NewU32Field(name, value))
}

func (r *Record) U64Field(name string, value uint64) {
	r.fields = append(r.fields, rfield.NewU64Field(name, value))
}

func (r *Record) F32Field(name string, value float32) {
	r.fields = append(r.fields, rfield.NewF32Field(name, value))
}

func (r *Record) F64Field(name string, value float64) {
	r.fields = append(r.fields, rfield.NewF64Field(name, value))
}

func (r *Record) StringField(name string, value string) {
	r.fields = append(r.fields, rfield.NewStringField(name, value))
}

func (r *Record) BinaryField(name string, value []byte) {
	r.fields = append(r.fields, rfield.NewBinaryField(name, value))
}

func (r *Record) StructField(name string, value rfield.Struct) {
	r.fields = append(r.fields, rfield.NewStructField(name, value))
}

func (r *Record) ListField(name string, value rfield.List) {
	r.fields = append(r.fields, rfield.NewListField(name, value))
}

func (r *Record) ValueByPath(path []int) rfield.Value {
	if path == nil {
		return nil
	}
	if len(r.fields) > path[0] {
		return r.fields[path[0]].ValueByPath(path[1:])
	}
	return nil
}

// Compare compares two records based on an order by clause expressed as a collection of numerical path.
func (r *Record) Compare(other *Record, sortBy [][]int) int {
	for _, path := range sortBy {
		v := r.ValueByPath(path)
		otherV := other.ValueByPath(path)
		if v == nil || otherV == nil {
			panic("compare: invalid path")
		}

		if cmp := v.Compare(otherV); cmp != 0 {
			// Not equals
			return cmp
		}
	}
	return 0
}
