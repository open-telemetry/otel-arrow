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

// This package contains functions for printing Arrow records to stdout.
// This is mostly used for debugging purposes.

import (
	"fmt"
	"math"
	"strings"

	"github.com/apache/arrow/go/v12/arrow"
	"github.com/apache/arrow/go/v12/arrow/array"

	"github.com/open-telemetry/otel-arrow/pkg/werror"
)

const (
	StrCode    int8 = 0
	I64Code    int8 = 1
	F64Code    int8 = 2
	BoolCode   int8 = 3
	BinaryCode int8 = 4
	CborCode   int8 = 5

	MaxColSize = 20
	MaxStrSize = "%20s"
	MaxValSize = "%20v"
	MaxBinSize = "%20x"
)

// PrintRecord prints the contents of an Arrow record to stdout.
func PrintRecord(name string, record arrow.Record, maxRows, countPrints, maxPrints int) {
	println()

	if record.NumRows() > int64(maxRows) {
		fmt.Printf("Record %q -> #rows: %d/%d, prints: %d/%d\n", name, maxRows, record.NumRows(), countPrints, maxPrints)
	} else {
		fmt.Printf("Record %q -> #rows: %d, prints: %d/%d\n", name, record.NumRows(), countPrints, maxPrints)
	}

	schema := record.Schema()
	colNames := schemaColNames(schema)

	for i := 0; i < len(colNames); i++ {
		print(strings.Repeat("-", MaxColSize), "-+")
	}
	println()

	for _, colName := range colNames {
		if len(colName) > MaxColSize {
			colName = colName[:MaxColSize]
		}
		fmt.Printf(MaxStrSize, colName)
		print(" |")
	}
	println()

	for i := 0; i < len(colNames); i++ {
		print(strings.Repeat("-", MaxColSize), "-+")
	}
	println()

	rows := int(math.Min(500.0, float64(record.NumRows())))
	for row := 0; row < rows; row++ {
		values := recordColValues(record, row)
		for _, value := range values {
			if len(value) > MaxColSize {
				value = value[:MaxColSize]
			}
			fmt.Printf(MaxStrSize, value)
			print(" |")
		}
		println()
	}
}

func schemaColNames(schema *arrow.Schema) []string {
	var names []string
	for _, field := range schema.Fields() {
		names = append(names, fieldColNames("", &field)...)
	}
	return names
}

func fieldColNames(path string, field *arrow.Field) []string {
	path = path + field.Name

	st, isStruct := field.Type.(*arrow.StructType)
	if isStruct {
		var names []string
		path = path + "."
		for _, structField := range st.Fields() {
			names = append(names, fieldColNames(path, &structField)...)
		}
		return names
	}

	return []string{path}
}

func recordColValues(record arrow.Record, row int) []string {
	var values []string

	for col := 0; col < int(record.NumCols()); col++ {
		arr := record.Column(col)
		values = append(values, arrayColValues(arr, row)...)
	}
	return values
}

func arrayColValues(arr arrow.Array, row int) []string {
	if arr.IsNull(row) {
		return []string{fmt.Sprintf(MaxStrSize, "NULL")}
	}

	switch c := arr.(type) {
	case *array.Boolean:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	// uints
	case *array.Uint8:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Uint16:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Uint32:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Uint64:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	// ints
	case *array.Int8:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Int16:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Int32:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Int64:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	// floats
	case *array.Float32:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Float64:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}

	case *array.String:
		str := c.Value(row)
		if len(str) > MaxColSize {
			str = str[:MaxColSize]
		}
		return []string{fmt.Sprintf(MaxValSize, str)}
	case *array.Binary:
		bin := c.Value(row)
		if len(bin) > MaxColSize {
			bin = bin[:MaxColSize]
		}
		return []string{fmt.Sprintf(MaxValSize, bin)}
	case *array.FixedSizeBinary:
		bin := c.Value(row)
		if len(bin) > MaxColSize {
			bin = bin[:MaxColSize]
		}
		return []string{fmt.Sprintf(MaxBinSize, bin)}
	case *array.Timestamp:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Duration:
		return []string{fmt.Sprintf(MaxValSize, c.Value(row))}
	case *array.Dictionary:
		switch arr := c.Dictionary().(type) {
		case *array.Int32:
			return []string{fmt.Sprintf("%d", arr.Value(c.GetValueIndex(row)))}
		case *array.Int64:
			return []string{fmt.Sprintf("%d", arr.Value(c.GetValueIndex(row)))}
		case *array.Uint32:
			return []string{fmt.Sprintf("%d", arr.Value(c.GetValueIndex(row)))}
		case *array.String:
			str := arr.Value(c.GetValueIndex(row))
			if len(str) > MaxColSize {
				str = str[:MaxColSize]
			}
			return []string{fmt.Sprintf(MaxValSize, str)}
		case *array.Binary:
			bin := arr.Value(c.GetValueIndex(row))
			if len(bin) > MaxColSize {
				bin = bin[:MaxColSize]
			}
			return []string{fmt.Sprintf(MaxValSize, bin)}
		case *array.Duration:
			return []string{fmt.Sprintf(MaxValSize, arr.Value(c.GetValueIndex(row)))}
		case *array.FixedSizeBinary:
			bin := arr.Value(c.GetValueIndex(row))
			if len(bin) > MaxColSize {
				bin = bin[:MaxColSize]
			}
			return []string{fmt.Sprintf(MaxBinSize, bin)}
		default:
			panic(fmt.Sprintf("unsupported dictionary type %T", arr))
		}
	case *array.Struct:
		var values []string
		for i := 0; i < c.NumField(); i++ {
			values = append(values, arrayColValues(c.Field(i), row)...)
		}
		return values
	case *array.SparseUnion:
		return []string{sparseUnionValue(c, row)}
	case *array.List:
		return []string{"List not supported"}
	default:
		panic(fmt.Sprintf("unsupported array type %T", arr))
	}
	return []string{}
}

func sparseUnionValue(union *array.SparseUnion, row int) string {
	tcode := union.TypeCode(row)
	fieldID := union.ChildID(row)

	switch tcode {
	case StrCode:
		strArr := union.Field(fieldID)
		if strArr.IsNull(row) {
			return ""
		}

		switch arr := strArr.(type) {
		case *array.String:
			return arr.Value(row)
		case *array.Dictionary:
			return arr.Dictionary().(*array.String).Value(arr.GetValueIndex(row))
		default:
			panic(fmt.Sprintf("unsupported array type %T", arr))
		}
	case I64Code:
		i64Arr := union.Field(fieldID)
		val, err := i64FromArray(i64Arr, row)
		if err != nil {
			panic(err)
		}
		return fmt.Sprintf("%d", val)
	case F64Code:
		f64Arr := union.Field(fieldID)
		val := f64Arr.(*array.Float64).Value(row)
		return fmt.Sprintf("%f", val)
	case BoolCode:
		boolArr := union.Field(fieldID)
		val := boolArr.(*array.Boolean).Value(row)
		return fmt.Sprintf("%t", val)
	case BinaryCode:
		binArr := union.Field(fieldID)
		val, err := binaryFromArray(binArr, row)
		if err != nil {
			panic(err)
		}
		return fmt.Sprintf("%x", val)
	case CborCode:
		panic("cbor not supported")
	default:
		panic(fmt.Sprintf("unsupported type code %d", tcode))
	}
}

func i64FromArray(arr arrow.Array, row int) (int64, error) {
	if arr == nil {
		return 0, nil
	} else {
		switch arr := arr.(type) {
		case *array.Int64:
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return arr.Value(row), nil
			}
		case *array.Dictionary:
			i64Arr := arr.Dictionary().(*array.Int64)
			if arr.IsNull(row) {
				return 0, nil
			} else {
				return i64Arr.Value(arr.GetValueIndex(row)), nil
			}
		default:
			return 0, werror.WrapWithMsg(ErrInvalidArrayType, "not an int64 array")
		}
	}
}

func binaryFromArray(arr arrow.Array, row int) ([]byte, error) {
	if arr == nil {
		return nil, nil
	} else {
		if arr.IsNull(row) {
			return nil, nil
		}

		switch arr := arr.(type) {
		case *array.Binary:
			return arr.Value(row), nil
		case *array.Dictionary:
			return arr.Dictionary().(*array.Binary).Value(arr.GetValueIndex(row)), nil
		default:
			return nil, werror.WrapWithMsg(ErrInvalidArrayType, "not a binary array")
		}
	}
}
