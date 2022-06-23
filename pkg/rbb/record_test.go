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

package rbb

import (
	"github.com/google/go-cmp/cmp"
	"testing"
)

func TestRecordNormalize(t *testing.T) {
	record := Record{
		fields: []Field{
			{Name: "b", Value: &String{Value: ""}},
			{Name: "a", Value: &Struct{
				fields: []Field{
					{Name: "e", Value: &String{Value: ""}},
					{Name: "b", Value: &String{Value: ""}},
					{Name: "c", Value: &Struct{
						fields: []Field{
							{Name: "x", Value: &String{Value: ""}},
							{Name: "t", Value: &String{Value: ""}},
							{Name: "z", Value: &List{
								values: []Value{
									&I64{Value: 1},
									&I64{Value: 2},
								},
							}},
							{Name: "a", Value: &List{
								values: []Value{
									&Struct{
										fields: []Field{
											{Name: "f2_3_4_2", Value: &String{Value: "f2_3_4_2"}},
											{Name: "f2_3_4_1", Value: &String{Value: "f2_3_4_1"}},
										},
									},
								},
							}},
						},
					}},
				},
			}},
		},
	}
	record.Normalize()

	expected_record := Record{
		fields: []Field{
			{Name: "a", Value: &Struct{
				fields: []Field{
					{Name: "b", Value: &String{Value: ""}},
					{Name: "c", Value: &Struct{
						fields: []Field{
							{Name: "a", Value: &List{
								values: []Value{
									&Struct{
										fields: []Field{
											{Name: "f2_3_4_1", Value: &String{Value: "f2_3_4_1"}},
											{Name: "f2_3_4_2", Value: &String{Value: "f2_3_4_2"}},
										},
									},
								},
							}},
							{Name: "t", Value: &String{Value: ""}},
							{Name: "x", Value: &String{Value: ""}},
							{Name: "z", Value: &List{
								values: []Value{
									&I64{Value: 1},
									&I64{Value: 2},
								},
							}},
						},
					}},
					{Name: "e", Value: &String{Value: ""}},
				},
			}},
			{Name: "b", Value: &String{Value: ""}},
		},
	}

	if !cmp.Equal(record, expected_record, cmp.AllowUnexported(Record{}, Struct{}, List{})) {
		t.Errorf("Expected: %+v\nGot: %+v", expected_record, record)
	}
}

func TestRecordSchemaId(t *testing.T) {
	record := Record{
		fields: []Field{
			{Name: "b", Value: &String{Value: ""}},
			{Name: "a", Value: &Struct{
				fields: []Field{
					{Name: "e", Value: &String{Value: ""}},
					{Name: "b", Value: &String{Value: ""}},
					{Name: "c", Value: &Struct{
						fields: []Field{
							{Name: "y", Value: &List{
								values: []Value{
									&I8{Value: 1},
									&I64{Value: 2},
									&String{Value: "true"},
								},
							}},
							{Name: "x", Value: &List{
								values: []Value{
									&I8{Value: 1},
									&I64{Value: 2},
									&Bool{Value: true},
								},
							}},
							{Name: "t", Value: &String{Value: ""}},
							{Name: "z", Value: &List{
								values: []Value{
									&I8{Value: 1},
									&I64{Value: 2},
								},
							}},
							{Name: "a", Value: &List{
								values: []Value{
									&Struct{
										fields: []Field{
											{Name: "f2_3_4_2", Value: &I8{Value: 1}},
											{Name: "f2_3_4_1", Value: &I8{Value: 2}},
										},
									},
									&Struct{
										fields: []Field{
											{Name: "f2_3_4_3", Value: &String{Value: "f2_3_4_3"}},
											{Name: "f2_3_4_1", Value: &String{Value: "f2_3_4_1"}},
										},
									},
								},
							}},
						},
					}},
				},
			}},
		},
	}
	record.Normalize()
	id := record.SchemaId()
	expectedSchemaId := "a:{b:Str,c:{a:[{f2_3_4_1:Str,f2_3_4_2:I8,f2_3_4_3:Str}],t:Str,x:[I64],y:[Str],z:[I64]},e:Str},b:Str"
	if id != expectedSchemaId {
		t.Errorf("Expected: %s\nGot: %s", expectedSchemaId, id)
	}
}
