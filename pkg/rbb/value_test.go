// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package rbb

import (
	"testing"
)

func TestNormalize(t *testing.T) {
	// Test normalize on a scalar value
	s1 := Bool{
		Value: true,
	}
	s1.Normalize()
	if s1.Value != true {
		t.Errorf("Expected true, got %v", s1.Value)
	}

	// Test normalize on a struct value
	s2 := Struct{
		fields: []Field{
			{
				Name:  "c",
				Value: &Bool{Value: false},
			},
			{
				Name:  "a",
				Value: &I64{Value: 123},
			},
			{
				Name:  "b",
				Value: &String{Value: "text"},
			},
		},
	}
	s2.Normalize()
	if s2.fields[0].Name != "a" {
		t.Errorf("Expected a, got %v", s2.fields[0].Name)
	}
	if s2.fields[1].Name != "b" {
		t.Errorf("Expected b, got %v", s2.fields[1].Name)
	}
	if s2.fields[2].Name != "c" {
		t.Errorf("Expected c, got %v", s2.fields[2].Name)
	}

	// Test normalize on a list value
	s3 := List{
		values: []Value{
			&Bool{Value: true},
			&I64{Value: 1},
			&Struct{
				fields: []Field{
					{
						Name:  "c",
						Value: &Bool{Value: false},
					},
					{
						Name:  "a",
						Value: &I64{Value: 123},
					},
					{
						Name:  "b",
						Value: &String{Value: "text"},
					},
				},
			},
		},
	}
	s3.Normalize()
	if s3.values[2].(*Struct).fields[0].Name != "a" {
		t.Errorf("Expected a, got %v", s2.fields[0].Name)
	}
	if s3.values[2].(*Struct).fields[1].Name != "b" {
		t.Errorf("Expected b, got %v", s2.fields[1].Name)
	}
	if s3.values[2].(*Struct).fields[2].Name != "c" {
		t.Errorf("Expected c, got %v", s2.fields[2].Name)
	}
}
