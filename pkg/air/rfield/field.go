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

package rfield

import (
	"strings"

	"github.com/apache/arrow/go/v9/arrow"
)

type Fields []*Field

// Sort interface
func (f Fields) Less(i, j int) bool {
	return f[i].Name < f[j].Name
}
func (f Fields) Len() int      { return len(f) }
func (f Fields) Swap(i, j int) { f[i], f[j] = f[j], f[i] }

// Field is a scalar or a composite named value.
type Field struct {
	Name  string
	Value Value
}

func NewField(name string, value Value) *Field {
	return &Field{
		Name:  name,
		Value: value,
	}
}

func NewBoolField(name string, value bool) *Field {
	return &Field{
		Name: name,
		Value: &Bool{
			Value: value,
		},
	}
}

func NewI8Field(name string, value int8) *Field {
	return &Field{
		Name: name,
		Value: &I8{
			Value: value,
		},
	}
}

func NewI16Field(name string, value int16) *Field {
	return &Field{
		Name: name,
		Value: &I16{
			Value: value,
		},
	}
}

func NewI32Field(name string, value int32) *Field {
	return &Field{
		Name: name,
		Value: &I32{
			Value: value,
		},
	}
}

func NewI64Field(name string, value int64) *Field {
	return &Field{
		Name: name,
		Value: &I64{
			Value: value,
		},
	}
}

func NewU8Field(name string, value uint8) *Field {
	return &Field{
		Name: name,
		Value: &U8{
			Value: value,
		},
	}
}

func NewU16Field(name string, value uint16) *Field {
	return &Field{
		Name: name,
		Value: &U16{
			Value: value,
		},
	}
}

func NewU32Field(name string, value uint32) *Field {
	return &Field{
		Name: name,
		Value: &U32{
			Value: value,
		},
	}
}

func NewU64Field(name string, value uint64) *Field {
	return &Field{
		Name: name,
		Value: &U64{
			Value: value,
		},
	}
}

func NewF32Field(name string, value float32) *Field {
	return &Field{
		Name: name,
		Value: &F32{
			Value: value,
		},
	}
}

func NewF64Field(name string, value float64) *Field {
	return &Field{
		Name: name,
		Value: &F64{
			Value: value,
		},
	}
}

func NewStringField(name string, value string) *Field {
	return &Field{
		Name: name,
		Value: &String{
			Value: value,
		},
	}
}

func NewBinaryField(name string, value []byte) *Field {
	return &Field{
		Name: name,
		Value: &Binary{
			Value: value,
		},
	}
}

func NewStructField(name string, value Struct) *Field {
	return &Field{
		Name:  name,
		Value: &value,
	}
}

func NewListField(name string, value List) *Field {
	return &Field{
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

func (f *Field) StringPath(path []int) string {
	if len(path) == 0 {
		return f.Name
	} else {
		subPath := f.Value.StringPath(path)
		if subPath != "" {
			return f.Name + "." + subPath
		} else {
			return f.Name
		}
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
		sig.WriteString(DataTypeSignature(v.EType()))
		sig.WriteString("]")
	default:
		panic("unknown field value type")
	}
}
