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

import (
	"fmt"

	"github.com/apache/arrow/go/v11/arrow/array"
)

// AdaptiveDictionaryBuilder is a wrapper around arrow builders that
// can be used to append values to a dictionary builder or a regular
// builder (string or binary) depending on the type of the builder.
type AdaptiveDictionaryBuilder struct {
	builder array.Builder
}

// AdaptiveDictionaryBuilderFrom creates a new AdaptiveDictionaryBuilder from an existing builder.
func AdaptiveDictionaryBuilderFrom(builder array.Builder) *AdaptiveDictionaryBuilder {
	return &AdaptiveDictionaryBuilder{builder: builder}
}

// AppendString appends a string value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendString(s string) error {
	switch b := b.builder.(type) {
	case *array.BinaryDictionaryBuilder:
		return b.AppendString(s)
	case *array.StringBuilder:
		b.Append(s)
		return nil
	default:
		return fmt.Errorf("AdaptiveDictionaryBuilder: unsupported builder type: %T", b)
	}
}

// AppendBinary appends a binary value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendBinary(v []byte) error {
	switch b := b.builder.(type) {
	case *array.BinaryDictionaryBuilder:
		return b.Append(v)
	case *array.FixedSizeBinaryDictionaryBuilder:
		return b.Append(v)
	case *array.FixedSizeBinaryBuilder:
		b.Append(v)
		return nil
	case *array.BinaryBuilder:
		b.Append(v)
		return nil
	default:
		return fmt.Errorf("AdaptiveDictionaryBuilder: unsupported builder type: %T", b)
	}
}

// AppendI32 appends a i32 value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendI32(v int32) error {
	switch b := b.builder.(type) {
	case *array.Int32DictionaryBuilder:
		return b.Append(v)
	case *array.Int32Builder:
		b.Append(v)
		return nil
	default:
		return fmt.Errorf("AdaptiveDictionaryBuilder: unsupported builder type: %T", b)
	}
}

// AppendNull appends a null value to the builder.
func (b *AdaptiveDictionaryBuilder) AppendNull() {
	b.builder.AppendNull()
}

// Release releases the underlying builder.
func (b *AdaptiveDictionaryBuilder) Release() {
	b.builder.Release()
}
