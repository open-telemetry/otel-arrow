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

package rbb_test

import (
	"github.com/google/go-cmp/cmp"
	"otel-arrow-adapter/pkg/rbb"
	"otel-arrow-adapter/pkg/rbb/value"
	"testing"
)

func TestValue(t *testing.T) {
	t.Parallel()

	record := rbb.NewRecord()
	record.StringField("b", "b")
	record.StructField("a", value.Struct{
		Fields: []value.Field{
			{Name: "e1", Value: &value.String{Value: "e1"}},
			{Name: "b1", Value: &value.String{Value: "b1"}},
			{Name: "c1", Value: &value.Struct{
				Fields: []value.Field{
					{Name: "x", Value: &value.String{Value: "x"}},
					{Name: "t", Value: &value.String{Value: "t"}},
					{Name: "z", Value: &value.List{
						Values: []value.Value{
							&value.I64{Value: 1},
							&value.I64{Value: 2},
						},
					}},
					{Name: "a", Value: &value.List{
						Values: []value.Value{
							&value.Struct{
								Fields: []value.Field{
									{Name: "f2_3_4_2", Value: &value.String{Value: "f2_3_4_2"}},
									{Name: "f2_3_4_1", Value: &value.String{Value: "f2_3_4_1"}},
								},
							},
						},
					}},
				},
			}},
		},
	})
	record.Normalize()

	v := record.ValueByPath([]int{0, 0}) // field "b"
	if v.(*value.String).Value != "b1" {
		t.Errorf("expected the value of field \"a.b1\" to be \"b1\", got %v", v)
	}

	v = record.ValueByPath([]int{0, 1, 0, 0, 0}) // field "a.c1.a.f2_3_4_1"
	if v.(*value.String).Value != "f2_3_4_1" {
		t.Errorf("expected the value of field \"a.c1.a.f2_3_4_1\" to be \"f2_3_4_1\", got %v", v)
	}

	v = record.ValueByPath([]int{0, 1, 1}) // field "a.c1.t"
	if v.(*value.String).Value != "t" {
		t.Errorf("expected the value of field \"a.c1.t\" to be \"t\", got %v", v)
	}

	v = record.ValueByPath([]int{0, 1, 2}) // field "a.c1.x"
	if v.(*value.String).Value != "x" {
		t.Errorf("expected the value of field \"a.c1.x\" to be \"x\", got %v", v)
	}

	v = record.ValueByPath([]int{0, 1, 3, 0}) // field "a.c1.z[0]"
	if v.(*value.I64).Value != 1 {
		t.Errorf("expected the value of field \"a.c1.z[0]\" to be \"1\", got %v", v)
	}

	v = record.ValueByPath([]int{0, 1, 3, 1}) // field "a.c1.z[1]"
	if v.(*value.I64).Value != 2 {
		t.Errorf("expected the value of field \"a.c1.z[1]\" to be \"2\", got %v", v)
	}

	v = record.ValueByPath([]int{0, 2}) // field "a.e1"
	if v.(*value.String).Value != "e1" {
		t.Errorf("expected the value of field \"a.e1\" to be \"e1\", got %v", v)
	}

	v = record.ValueByPath([]int{1}) // field "b"
	if v.(*value.String).Value != "b" {
		t.Errorf("expected the value of field \"b\" to be \"b\", got %v", v)
	}
}

func TestRecordNormalize(t *testing.T) {
	t.Parallel()

	record := rbb.NewRecord()
	record.StringField("b", "")
	record.StructField("a", value.Struct{
		Fields: []value.Field{
			{Name: "e", Value: &value.String{Value: ""}},
			{Name: "b", Value: &value.String{Value: ""}},
			{Name: "c", Value: &value.Struct{
				Fields: []value.Field{
					{Name: "x", Value: &value.String{Value: ""}},
					{Name: "t", Value: &value.String{Value: ""}},
					{Name: "z", Value: &value.List{
						Values: []value.Value{
							&value.I64{Value: 1},
							&value.I64{Value: 2},
						},
					}},
					{Name: "a", Value: &value.List{
						Values: []value.Value{
							&value.Struct{
								Fields: []value.Field{
									{Name: "f2_3_4_2", Value: &value.String{Value: "f2_3_4_2"}},
									{Name: "f2_3_4_1", Value: &value.String{Value: "f2_3_4_1"}},
								},
							},
						},
					}},
				},
			}},
		},
	})
	record.Normalize()

	expected_record := rbb.NewRecord()
	expected_record.StructField("a", value.Struct{
		Fields: []value.Field{
			{Name: "b", Value: &value.String{Value: ""}},
			{Name: "c", Value: &value.Struct{
				Fields: []value.Field{
					{Name: "a", Value: &value.List{
						Values: []value.Value{
							&value.Struct{
								Fields: []value.Field{
									{Name: "f2_3_4_1", Value: &value.String{Value: "f2_3_4_1"}},
									{Name: "f2_3_4_2", Value: &value.String{Value: "f2_3_4_2"}},
								},
							},
						},
					}},
					{Name: "t", Value: &value.String{Value: ""}},
					{Name: "x", Value: &value.String{Value: ""}},
					{Name: "z", Value: &value.List{
						Values: []value.Value{
							&value.I64{Value: 1},
							&value.I64{Value: 2},
						},
					}},
				},
			}},
			{Name: "e", Value: &value.String{Value: ""}},
		},
	})
	expected_record.StringField("b", "")

	if !cmp.Equal(record, expected_record, cmp.AllowUnexported(rbb.Record{}, value.Struct{}, value.List{})) {
		t.Errorf("Expected: %+v\nGot: %+v", expected_record, record)
	}
}

func TestRecordSchemaId(t *testing.T) {
	t.Parallel()

	record := rbb.NewRecord()
	record.StringField("b", "")
	record.StructField("a", value.Struct{
		Fields: []value.Field{
			{Name: "e", Value: &value.String{Value: ""}},
			{Name: "b", Value: &value.String{Value: ""}},
			{Name: "c", Value: &value.Struct{
				Fields: []value.Field{
					{Name: "y", Value: &value.List{
						Values: []value.Value{
							&value.I8{Value: 1},
							&value.I64{Value: 2},
							&value.String{Value: "true"},
						},
					}},
					{Name: "x", Value: &value.List{
						Values: []value.Value{
							&value.I8{Value: 1},
							&value.I64{Value: 2},
							&value.Bool{Value: true},
						},
					}},
					{Name: "t", Value: &value.String{Value: ""}},
					{Name: "z", Value: &value.List{
						Values: []value.Value{
							&value.I8{Value: 1},
							&value.I64{Value: 2},
						},
					}},
					{Name: "a", Value: &value.List{
						Values: []value.Value{
							&value.Struct{
								Fields: []value.Field{
									{Name: "f2_3_4_2", Value: &value.I8{Value: 1}},
									{Name: "f2_3_4_1", Value: &value.I8{Value: 2}},
								},
							},
							&value.Struct{
								Fields: []value.Field{
									{Name: "f2_3_4_3", Value: &value.String{Value: "f2_3_4_3"}},
									{Name: "f2_3_4_1", Value: &value.String{Value: "f2_3_4_1"}},
								},
							},
						},
					}},
				},
			}},
		},
	})

	record.Normalize()
	id := record.SchemaId()
	expectedSchemaId := "a:{b:Str,c:{a:[{f2_3_4_1:Str,f2_3_4_2:I8,f2_3_4_3:Str}],t:Str,x:[I64],y:[Str],z:[I64]},e:Str},b:Str"
	if id != expectedSchemaId {
		t.Errorf("Expected: %s\nGot: %s", expectedSchemaId, id)
	}
}
