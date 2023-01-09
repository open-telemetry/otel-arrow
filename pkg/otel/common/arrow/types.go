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

package arrow

import "github.com/apache/arrow/go/v11/arrow"

var (
	DefaultDictString = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.String,
		Ordered:   false,
	}

	DefaultDictBinary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: arrow.BinaryTypes.Binary,
		Ordered:   false,
	}

	DefaultDictFixed16Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: &arrow.FixedSizeBinaryType{ByteWidth: 16},
		Ordered:   false,
	}

	DefaultDictFixed8Binary = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint16,
		ValueType: &arrow.FixedSizeBinaryType{ByteWidth: 8},
		Ordered:   false,
	}

	DefaultDictInt32 = &arrow.DictionaryType{
		IndexType: arrow.PrimitiveTypes.Uint8,
		ValueType: arrow.PrimitiveTypes.Int32,
		Ordered:   false,
	}
)
