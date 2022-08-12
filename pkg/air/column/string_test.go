/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package column

import (
	"fmt"
	"testing"

	"github.com/apache/arrow/go/v9/arrow"
	"github.com/apache/arrow/go/v9/arrow/array"
	"github.com/apache/arrow/go/v9/arrow/memory"
)

func TestDictionaryBuilder(t *testing.T) {
	allocator := memory.NewGoAllocator()
	dicoType := arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint8,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}
	dicoBuilder := array.NewDictionaryBuilder(allocator, &dicoType).(*array.BinaryDictionaryBuilder)

	dicoBuilder.AppendString("test3")
	dicoBuilder.AppendString("test4")
	dicoBuilder.AppendString("test1")
	dicoBuilder.AppendString("test3")
	dicoBuilder.AppendString("test2")

	dicoArray := dicoBuilder.NewArray()
	fmt.Printf("%v\n", dicoArray)
	dicoArray.Release()

	dicoBuilder.AppendString("test3")
	dicoBuilder.AppendString("test4")
	dicoBuilder.AppendString("test1")
	dicoBuilder.AppendString("test3")
	dicoBuilder.AppendString("test2")

	dicoArray = dicoBuilder.NewArray()
	fmt.Printf("%v\n", dicoArray)
	dicoArray.Release()

	dicoBuilder.AppendString("test1")
	dicoBuilder.AppendString("test3")
	dicoBuilder.AppendString("test2")

	dicoArray = dicoBuilder.NewArray()
	fmt.Printf("%v\n", dicoArray)
	dicoArray.Release()

	dicoBuilder.AppendString("test4")
	dicoBuilder.AppendString("test5")
	dicoBuilder.AppendString("test6")

	dicoArray = dicoBuilder.NewArray()
	fmt.Printf("%v\n", dicoArray)
	dicoArray.Release()

	dicoBuilder.AppendString("test4")
	dicoBuilder.AppendString("test5")
	dicoBuilder.AppendString("test6")

	indices, delta, err := dicoBuilder.NewDelta()
	fmt.Printf("%v\n", indices)
	fmt.Printf("%v\n", delta)
	fmt.Printf("%v\n", err)

	dicoBuilder.AppendString("test4")
	dicoBuilder.AppendString("test7")
	dicoBuilder.AppendString("test8")

	indices, delta, err = dicoBuilder.NewDelta()
	fmt.Printf("%v\n", indices)
	fmt.Printf("%v\n", delta)
	fmt.Printf("%v\n", err)

}
