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

	"github.com/apache/arrow/go/v11/arrow"
)

type Fields []*Field

// Sort interface
func (f Fields) Less(i, j int) bool {
	return f[i].Name < f[j].Name
}
func (f Fields) Len() int      { return len(f) }
func (f Fields) Swap(i, j int) { f[i], f[j] = f[j], f[i] }

type Metadata struct {
	Keys   []string
	Values []string
}

// Field is a scalar or a composite named value.
type Field struct {
	Name     string
	Value    Value
	metadata *Metadata
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
			Value: &value,
		},
	}
}

func NewI8Field(name string, value int8) *Field {
	return &Field{
		Name: name,
		Value: &I8{
			Value: &value,
		},
	}
}

func NewI16Field(name string, value int16) *Field {
	return &Field{
		Name: name,
		Value: &I16{
			Value: &value,
		},
	}
}

func NewI32Field(name string, value int32) *Field {
	return &Field{
		Name: name,
		Value: &I32{
			Value: &value,
		},
	}
}

func NewI64Field(name string, value int64) *Field {
	return &Field{
		Name: name,
		Value: &I64{
			Value: &value,
		},
	}
}

func NewU8Field(name string, value uint8) *Field {
	return &Field{
		Name: name,
		Value: &U8{
			Value: &value,
		},
	}
}

func NewU16Field(name string, value uint16) *Field {
	return &Field{
		Name: name,
		Value: &U16{
			Value: &value,
		},
	}
}

func NewU32Field(name string, value uint32) *Field {
	return &Field{
		Name: name,
		Value: &U32{
			Value: &value,
		},
	}
}

func NewU64Field(name string, value uint64) *Field {
	return &Field{
		Name: name,
		Value: &U64{
			Value: &value,
		},
	}
}

func NewF32Field(name string, value float32) *Field {
	return &Field{
		Name: name,
		Value: &F32{
			Value: &value,
		},
	}
}

func NewF64Field(name string, value float64) *Field {
	return &Field{
		Name: name,
		Value: &F64{
			Value: &value,
		},
	}
}

func NewStringField(name string, value string) *Field {
	return &Field{
		Name: name,
		Value: &String{
			Value: &value,
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

func NewNullFieldFromDataType(name string, dt arrow.DataType) *Field {
	switch t := dt.(type) {
	case *arrow.BooleanType:
		return &Field{
			Name: name,
			Value: &Bool{
				Value: nil,
			},
		}
	case *arrow.Int8Type:
		return &Field{
			Name: name,
			Value: &I8{
				Value: nil,
			},
		}
	case *arrow.Int16Type:
		return &Field{
			Name: name,
			Value: &I16{
				Value: nil,
			},
		}
	case *arrow.Int32Type:
		return &Field{
			Name: name,
			Value: &I32{
				Value: nil,
			},
		}
	case *arrow.Int64Type:
		return &Field{
			Name: name,
			Value: &I64{
				Value: nil,
			},
		}
	case *arrow.Uint8Type:
		return &Field{
			Name: name,
			Value: &U8{
				Value: nil,
			},
		}
	case *arrow.Uint16Type:
		return &Field{
			Name: name,
			Value: &U16{
				Value: nil,
			},
		}
	case *arrow.Uint32Type:
		return &Field{
			Name: name,
			Value: &U32{
				Value: nil,
			},
		}
	case *arrow.Uint64Type:
		return &Field{
			Name: name,
			Value: &U64{
				Value: nil,
			},
		}
	case *arrow.Float32Type:
		return &Field{
			Name: name,
			Value: &F32{
				Value: nil,
			},
		}
	case *arrow.Float64Type:
		return &Field{
			Name: name,
			Value: &F64{
				Value: nil,
			},
		}
	case *arrow.StringType:
		return &Field{
			Name: name,
			Value: &String{
				Value: nil,
			},
		}
	case *arrow.BinaryType:
		return &Field{
			Name: name,
			Value: &Binary{
				Value: nil,
			},
		}
	case *arrow.StructType:
		return &Field{
			Name: name,
			Value: &Struct{
				Fields: nil,
			},
		}
	case *arrow.ListType:
		return NewListField(name, List{})
	case *arrow.DictionaryType:
		switch t.ValueType.(type) {
		case *arrow.StringType:
			return NewStringField(name, "")
		case *arrow.BinaryType:
			return NewBinaryField(name, []byte{})
		default:
			panic("unsupported dictionary value type")
		}
	default:
		panic("unsupported type")
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

func (f *Field) Metadata() *Metadata {
	return f.metadata
}

func (f *Field) AddMetadata(key string, value string) {
	if f.metadata == nil {
		f.metadata = &Metadata{
			Keys:   []string{key},
			Values: []string{value},
		}
	} else {
		// Insertion sort (naive implementation as we don't expect many keys)
		// Metadata must be sorted by keys to be able to build a canonical signature (see WriteSigType).
		i := 0
		for ; i < len(f.metadata.Keys); i++ {
			if f.metadata.Keys[i] > key {
				break
			}
		}
		f.metadata.Keys = append(f.metadata.Keys, "")
		f.metadata.Values = append(f.metadata.Values, "")
		copy(f.metadata.Keys[i+1:], f.metadata.Keys[i:])
		copy(f.metadata.Values[i+1:], f.metadata.Values[i:])
		f.metadata.Keys[i] = key
		f.metadata.Values[i] = value
	}
}

// Normalize normalizes the field name and value.
func (f *Field) Normalize() {
	f.Value.Normalize()
}

// WriteSigType writes the field signature type to the given writer.
func (f *Field) WriteSigType(sig *strings.Builder) {
	sig.WriteString(f.Name)
	sig.WriteString(":")
	f.Value.WriteSignature(sig)

	if f.metadata != nil {
		sig.WriteString("<")
		for i, key := range f.metadata.Keys {
			if i > 0 {
				sig.WriteByte(',')
			}
			sig.WriteString(key)
			sig.WriteByte('=')
			sig.WriteString(f.metadata.Values[i])
		}
		sig.WriteString(">")
	}
}

// WriteSig writes the field signature (type + data) to the given writer.
// Important note: the field is supposed to be normalized before calling this method.
func (f *Field) WriteSig(sig *strings.Builder) {
	sig.WriteString(f.Name)
	sig.WriteString(":")
	f.Value.WriteSignature(sig)

	if f.metadata != nil {
		sig.WriteString("<")
		for i, key := range f.metadata.Keys {
			if i > 0 {
				sig.WriteByte(',')
			}
			sig.WriteString(key)
			sig.WriteByte('=')
			sig.WriteString(f.metadata.Values[i])
		}
		sig.WriteString(">")
	}
	sig.WriteString("=")
	f.Value.WriteData(sig)
}
