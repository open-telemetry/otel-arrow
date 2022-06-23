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

// DictIdGenerator defines a dictionary id generator.
type DictIdGenerator struct {
	id uint64
}

// FieldPath defines a field path.
type FieldPath struct {
	Current  uint64
	Children []FieldPath
}

type OrderBy struct {
	FieldPaths [][]uint64
}

type RecordList struct {
	Records []Record
}

// A RecordBatch builder.
// Must be fed with homogeneous records.
type RecordBatchBuilder struct {
	// The configuration of the builder.
	Config Config

	// The dictionary id generator.
	DictIdGen DictIdGenerator

	// The columns of the RecordBatch builder.
	Columns Columns

	// The path for each fields.
	FieldPaths []FieldPath

	// Optional order by clause
	OrderBy OrderBy

	// Non ordered records
	RecordList []RecordList

	// Flag to indicate if the builder has been optimized.
	Optimized bool
}
