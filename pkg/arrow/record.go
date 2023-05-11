/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package arrow

import (
	"fmt"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"
)

// PrintRecord prints the contents of an Arrow record to stdout.
func PrintRecord(record arrow.Record) {
	println()
	schema := record.Schema()
	for _, field := range schema.Fields() {
		name := field.Name
		if len(name) > 15 {
			name = name[:15]
		}
		fmt.Printf("%15s", name)
		print(" |")
	}
	println()

	println("#rows:", record.NumRows())
	for row := 0; row < int(record.NumRows()); row++ {
		for col := 0; col < int(record.NumCols()); col++ {
			column := record.Column(col)
			if column.IsNull(row) {
				fmt.Printf("%15s", "NULL")
				print(" |")
				continue
			}

			switch c := column.(type) {
			case *array.Boolean:
				fmt.Printf("%15v", c.Value(row))
			case *array.Uint32:
				fmt.Printf("%15v", c.Value(row))
			case *array.Int64:
				fmt.Printf("%15v", c.Value(row))
			case *array.String:
				str := c.Value(row)
				if len(str) > 15 {
					str = str[:15]
				}
				fmt.Printf("%15v", str)
			case *array.Float64:
				fmt.Printf("%15v", c.Value(row))
			case *array.Binary:
				bin := c.Value(row)
				if len(bin) > 15 {
					bin = bin[:15]
				}
				fmt.Printf("%15v", bin)
			case *array.Dictionary:
				switch arr := c.Dictionary().(type) {
				case *array.Int64:
					print(arr.Value(c.GetValueIndex(row)))
				case *array.String:
					str := arr.Value(c.GetValueIndex(row))
					if len(str) > 15 {
						str = str[:15]
					}
					fmt.Printf("%15v", str)
				case *array.Binary:
					bin := arr.Value(c.GetValueIndex(row))
					if len(bin) > 15 {
						bin = bin[:15]
					}
					fmt.Printf("%15v", bin)
				}
			}
			print(" |")
		}
		println()
	}
}
