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

import (
	"github.com/apache/arrow/go/v9/arrow"
	"otel-arrow-adapter/pkg/rbb/field_value"
)

// ListColumn is a column of list data.
type ListColumn struct {
	// Name of the column.
	Name string
	// Type of the list items.
	Type arrow.DataType
	// Data of the column.
	Data [][]field_value.Value
}

// Clear clears the list data in the column but keep the original memory buffer allocated.
func (c *ListColumn) Clear() {
	c.Data = c.Data[:0]
}
