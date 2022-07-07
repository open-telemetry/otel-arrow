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

// U8Column is a column of uint8 data.
type U8Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*uint8
}

// U16Column is a column of uint16 data.
type U16Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*uint16
}

// U32Column is a column of uint32 data.
type U32Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*uint32
}

// U64Column is a column of uint64 data.
type U64Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*uint64
}

// Clear clears the uint8 data in the column but keep the original memory buffer allocated.
func (c *U8Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the uint16 data in the column but keep the original memory buffer allocated.
func (c *U16Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the uint32 data in the column but keep the original memory buffer allocated.
func (c *U32Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the uint64 data in the column but keep the original memory buffer allocated.
func (c *U64Column) Clear() {
	c.Data = c.Data[:0]
}
