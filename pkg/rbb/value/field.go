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

package value

import (
	"github.com/apache/arrow/go/arrow"
	"strings"
)

// Field is a scalar or a composite named value.
type Field struct {
	Name  string
	Value Value
}

func MakeBoolField(name string, value bool) Field {
	return Field{
		Name: name,
		Value: &Bool{
			Value: value,
		},
	}
}

func MakeI8Field(name string, value int8) Field {
	return Field{
		Name: name,
		Value: &I8{
			Value: value,
		},
	}
}

func MakeI16Field(name string, value int16) Field {
	return Field{
		Name: name,
		Value: &I16{
			Value: value,
		},
	}
}

func MakeI32Field(name string, value int32) Field {
	return Field{
		Name: name,
		Value: &I32{
			Value: value,
		},
	}
}

func MakeI64Field(name string, value int64) Field {
	return Field{
		Name: name,
		Value: &I64{
			Value: value,
		},
	}
}

func MakeU8Field(name string, value uint8) Field {
	return Field{
		Name: name,
		Value: &U8{
			Value: value,
		},
	}
}

func MakeU16Field(name string, value uint16) Field {
	return Field{
		Name: name,
		Value: &U16{
			Value: value,
		},
	}
}

func MakeU32Field(name string, value uint32) Field {
	return Field{
		Name: name,
		Value: &U32{
			Value: value,
		},
	}
}

func MakeU64Field(name string, value uint64) Field {
	return Field{
		Name: name,
		Value: &U64{
			Value: value,
		},
	}
}

func MakeF32Field(name string, value float32) Field {
	return Field{
		Name: name,
		Value: &F32{
			Value: value,
		},
	}
}

func MakeF64Field(name string, value float64) Field {
	return Field{
		Name: name,
		Value: &F64{
			Value: value,
		},
	}
}

func MakeStringField(name string, value string) Field {
	return Field{
		Name: name,
		Value: &String{
			Value: value,
		},
	}
}

func MakeBinaryField(name string, value []byte) Field {
	return Field{
		Name: name,
		Value: &Binary{
			Value: value,
		},
	}
}

func MakeStructField(name string, value Struct) Field {
	return Field{
		Name:  name,
		Value: &value,
	}
}

func MakeListField(name string, value List) Field {
	return Field{
		Name:  name,
		Value: &value,
	}
}

func (f *Field) ValueByPath(path []int) Value {
	if len(path) == 0 {
		return f.Value
	} else {
		return f.Value.ValueByPath(path)
	}
}

func (f *Field) DataType() arrow.DataType {
	return f.Value.DataType()
}

// Normalize normalizes the field name and value.
func (f *Field) Normalize() {
	f.Value.Normalize()
}

func (f *Field) WriteSignature(sig *strings.Builder) {
	sig.WriteString(f.Name)
	sig.WriteString(":")
	switch v := f.Value.(type) {
	case *Bool:
		sig.WriteString(BOOL_SIG)
	case *I8:
		sig.WriteString(I8_SIG)
	case *I16:
		sig.WriteString(I16_SIG)
	case *I32:
		sig.WriteString(I32_SIG)
	case *I64:
		sig.WriteString(I64_SIG)
	case *U8:
		sig.WriteString(U8_SIG)
	case *U16:
		sig.WriteString(U16_SIG)
	case *U32:
		sig.WriteString(U32_SIG)
	case *U64:
		sig.WriteString(U64_SIG)
	case *F32:
		sig.WriteString(F32_SIG)
	case *F64:
		sig.WriteString(F64_SIG)
	case *String:
		sig.WriteString(STRING_SIG)
	case *Binary:
		sig.WriteString(BINARY_SIG)
	case *Struct:
		sig.WriteString("{")
		for i, f := range v.Fields {
			if i > 0 {
				sig.WriteByte(',')
			}
			f.WriteSignature(sig)
		}
		sig.WriteString("}")
	case *List:
		sig.WriteString("[")
		sig.WriteString(DataTypeSignature(ListDataType(v.Values)))
		sig.WriteString("]")
	default:
		panic("unknown field value type")
	}
}
