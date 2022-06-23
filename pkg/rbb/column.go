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

import "github.com/apache/arrow/go/arrow"

type BoolColumn struct {
	Name string
	Data []*bool
}

type I8Column struct {
	Name string
	Data []*int8
}

type I16Column struct {
	Name string
	Data []*int16
}

type I32Column struct {
	Name string
	Data []*int32
}

type I64Column struct {
	Name string
	Data []*int64
}

type U8Column struct {
	Name string
	Data []*uint8
}

type U16Column struct {
	Name string
	Data []*uint16
}

type U32Column struct {
	Name string
	Data []*uint32
}

type U64Column struct {
	Name string
	Data []*uint64
}

type F32Column struct {
	Name string
	Data []*float32
}

type F64Column struct {
	Name string
	Data []*float64
}

type StringColumn struct {
	Name string
	Data []*string
}

type BinaryColumn struct {
	Name string
	Data []*[]byte
}

type ListColumn struct {
	Name string
	Type arrow.DataType
	Data [][]Value
}

type StructColumn struct {
	Name    string
	Type    arrow.DataType
	Columns Columns
}

type Columns struct {
	BooleanColumns []BoolColumn

	I8Columns  []I8Column
	I16Columns []I16Column
	I32Columns []I32Column
	I64Columns []I64Column

	U8Columns  []U8Column
	U16Columns []U16Column
	U32Columns []U32Column
	U64Columns []U64Column

	F32Columns []F32Column
	F64Columns []F64Column

	StringColumns []StringColumn
	BinaryColumns []BinaryColumn

	ListColumns   []ListColumn
	StructColumns []StructColumn
}
