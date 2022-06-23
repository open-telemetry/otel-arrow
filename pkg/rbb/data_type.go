package rbb

import (
	"github.com/apache/arrow/go/arrow"
	"sort"
	"strings"
)

const BOOL_SIG = "Bol"
const U8_SIG = "U8"
const U16_SIG = "U16"
const U32_SIG = "U32"
const U64_SIG = "U64"
const I8_SIG = "I8"
const I16_SIG = "I16"
const I32_SIG = "I32"
const I64_SIG = "I64"
const F32_SIG = "F32"
const F64_SIG = "F64"
const BINARY_SIG = "Bin"
const STRING_SIG = "Str"

type NameType struct {
	Name string
	Type string
}

// DataTypeSignature returns the canonical arrow.DataType signature of the data type.
func DataTypeSignature(dataType arrow.DataType) string {
	switch dataType.ID() {
	case arrow.BOOL:
		return BOOL_SIG
	case arrow.UINT8:
		return U8_SIG
	case arrow.UINT16:
		return U16_SIG
	case arrow.UINT32:
		return U32_SIG
	case arrow.UINT64:
		return U64_SIG
	case arrow.INT8:
		return I8_SIG
	case arrow.INT16:
		return I16_SIG
	case arrow.INT32:
		return I32_SIG
	case arrow.INT64:
		return I64_SIG
	case arrow.FLOAT32:
		return F32_SIG
	case arrow.FLOAT64:
		return F64_SIG
	case arrow.STRING:
		return STRING_SIG
	case arrow.BINARY:
		return BINARY_SIG
	case arrow.LIST:
		return "[" + DataTypeSignature(dataType.(*arrow.ListType).Elem()) + "]"
	case arrow.STRUCT:
		var fields []NameType
		structDataType := dataType.(*arrow.StructType)
		for _, field := range structDataType.Fields() {
			fields = append(fields, NameType{
				Name: field.Name,
				Type: DataTypeSignature(field.Type),
			})
		}
		sort.Slice(fields, func(i, j int) bool {
			return fields[i].Name < fields[j].Name
		})
		fieldSigs := make([]string, 0, len(fields))
		for _, field := range fields {
			fieldSigs = append(fieldSigs, field.Name+":"+field.Type)
		}
		return "{" + strings.Join(fieldSigs, ",") + "}"
	default:
		panic("unknown data type '" + dataType.ID().String() + "'")
	}
}
