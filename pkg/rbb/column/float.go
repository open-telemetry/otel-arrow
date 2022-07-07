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

// F32Column is a column of float32 data.
type F32Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*float32
}

// F64Column is a column of float64 data.
type F64Column struct {
	// Name of the column.
	Name string
	// Data of the column.
	Data []*float64
}

// Clear clears the f32 data in the column but keep the original memory buffer allocated.
func (c *F32Column) Clear() {
	c.Data = c.Data[:0]
}

// Clear clears the f64 data in the column but keep the original memory buffer allocated.
func (c *F64Column) Clear() {
	c.Data = c.Data[:0]
}
