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

package column

// I8Column is a column of int8 data.
type I8Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*int8
}

// U8Column is a column of uint8 data.
type I16Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*int16
}

// I32Column is a column of int32 data.
type I32Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*int32
}

// I64Column is a column of int64 data.
type I64Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*int64
}

// Clear clears the int8 data in the column but keep the original memory buffer allocated.
func (c *I8Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the int16 data in the column but keep the original memory buffer allocated.
func (c *I16Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the int32 data in the column but keep the original memory buffer allocated.
func (c *I32Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the int64 data in the column but keep the original memory buffer allocated.
func (c *I64Column) Clear() {
	c.Data = c.Data[:0]
}
