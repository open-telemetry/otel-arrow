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

package value_test

import (
	"testing"

	"github.com/apache/arrow/go/v9/arrow/memory"

	value2 "otel-arrow-adapter/pkg/air/column"
	"otel-arrow-adapter/pkg/air/config"
)

func TestStringColumn(t *testing.T) {
	t.Parallel()

	dictionaryConfig := config.DictionaryConfig{
		MinRowCount:           10,
		MaxCard:               3,
		MaxCardRatio:          0.5,
		MaxSortedDictionaries: 5,
	}
	allocator := memory.NewGoAllocator()

	sc := value2.NewStringColumn(allocator, "test", &dictionaryConfig, []int{1}, 1)
	if sc.Name() != "test" {
		t.Errorf("Expected column name to be 'test', got %s", sc.Name())
	}

	// Push 5 strings + 1 nil string to the column
	value := "test1" // len = 5
	sc.Push(&value)
	value = "test2" // len = 5
	sc.Push(&value)
	value = "test2" // len = 5
	sc.Push(&value)
	value = "test1" // len = 5
	sc.Push(&value)
	sc.Push(nil)    // len = 0
	value = "test3" // len = 5
	sc.Push(&value)

	if sc.Len() != 6 {
		t.Errorf("Expected column length to be 6, got %d", sc.Len())
	}

	if sc.DictionaryLen() != 3 {
		t.Errorf("Expected dictionary length to be 3, got %d", sc.DictionaryLen())
	}

	if sc.AvgValueLength() != (5*5.0)/6.0 {
		t.Errorf("Expected average value length to be 5.0, got %f", sc.AvgValueLength())
	}

	// dictionary card > max card ==> no dictionary
	value = "test4" // len = 5
	sc.Push(&value)
	if sc.DictionaryLen() != 0 {
		t.Errorf("Expected dictionary length to be 0, got %d", sc.DictionaryLen())
	}
}
