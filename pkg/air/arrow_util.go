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

package air

import (
	"github.com/apache/arrow/go/v9/arrow"

	"otel-arrow-adapter/pkg/air/common"
)

func SchemaToId(schema *arrow.Schema) string {
	schemaId := ""
	for i := range schema.Fields() {
		field := &schema.Fields()[i]
		if i != 0 {
			schemaId += ","
		}
		schemaId += FieldToId(field)
	}
	return schemaId
}

func FieldToId(field *arrow.Field) string {
	return field.Name + ":" + DataTypeToId(field.Type)
}

func DataTypeToId(dt arrow.DataType) string {
	id := ""
	switch t := dt.(type) {
	case *arrow.BooleanType:
		id += common.BOOL_SIG
	case *arrow.Int8Type:
		id += common.I8_SIG
	case *arrow.Int16Type:
		id += common.I16_SIG
	case *arrow.Int32Type:
		id += common.I32_SIG
	case *arrow.Int64Type:
		id += common.I64_SIG
	case *arrow.Uint8Type:
		id += common.U8_SIG
	case *arrow.Uint16Type:
		id += common.U16_SIG
	case *arrow.Uint32Type:
		id += common.U32_SIG
	case *arrow.Uint64Type:
		id += common.U64_SIG
	case *arrow.Float32Type:
		id += common.F32_SIG
	case *arrow.Float64Type:
		id += common.F64_SIG
	case *arrow.StringType:
		id += common.STRING_SIG
	case *arrow.BinaryType:
		id += common.BINARY_SIG
	case *arrow.StructType:
		id += "{"
		for i, f := range t.Fields() {
			if i > 0 {
				id += ","
			}
			id += FieldToId(&f)
		}
		id += "}"
	case *arrow.ListType:
		id += "["
		elemField := t.ElemField()
		id += FieldToId(&elemField)
		id += "]"
	case *arrow.DictionaryType:
		id += "Dico<"
		id += DataTypeToId(t.IndexType)
		id += ","
		id += DataTypeToId(t.ValueType)
		id += ">"
	default:
		panic("unsupported data type")
	}

	return id
}
